//! Tool loading helpers – register tools via on-disk manifest files.
//!
//! The main entry-point is [`register_from_manifest_file`](crate::ToolRegistry::register_from_manifest_file).
//! It parses `ToolManifest` JSON (YAML/TOML support can be added later),
//! validates it, and instantiates an appropriate `Tool` implementation
//! based on the first supported transport.
//!
//! ## WASM Transport
//! When the manifest lists a `Transport::Wasm { path }`, the module is
//! executed inside Wasmtime (**optional** – behind the `wasm_loader` feature).
//! The guest MUST export an `execute` function with the signature:
//!
//! ```text
//! // Pseudo-signature (C ABI):
//! //   const char* execute(const char* json_params_ptr, size_t len, size_t* out_len);
//! // Returns a UTF-8, heap-allocated JSON string (caller frees via `free`).
//! ```
//!
//! Simpler host-dependent conventions (e.g. WASI + stdout) can be added later.
//! For the first iteration we assume a basic JSON-in/JSON-out function.
//!
//! ## Safety
//! • Manifests are validated via `ToolManifest::validate` (incl. JSON-Schema check).
//! • WASM modules run in a sandbox; WASI imports are disabled for now.
//! • Invalid or unsupported transports yield an `anyhow::Error`.

use crate::{manifest::*, Tool, ToolParams, ToolResult, ToolMetadata, ToolRegistry};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
use tokio::time::{timeout, Duration};

impl ToolRegistry {
    /// Load, validate and register a tool described by `manifest_path`.
    ///
    /// * The manifest must be JSON for now.
    /// * Only the first transport is honoured; others are ignored.
    ///
    /// Returns an error on validation failure or unsupported transport.
    pub async fn register_from_manifest_file<P: AsRef<Path>>(&self, manifest_path: P) -> Result<()> {
        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Reading manifest at {}", manifest_path.as_ref().display()))?;

        let manifest: ToolManifest = serde_json::from_str(&raw)
            .with_context(|| format!("{}: invalid JSON", manifest_path.as_ref().display()))?;

        manifest.validate()?;

        // Pick the **first transport we know how to handle** instead of
        // blindly taking index 0. This prevents accidentally using an
        // insecure or unsupported option when the manifest lists multiple
        // transports.
        let supported_transport = manifest.transports.iter().find(|t| match t {
            Transport::Wasm { .. } => cfg!(feature = "wasm_loader"),
            Transport::JsonRpcHttp { .. } => cfg!(feature = "http_client"),
            Transport::JsonRpcStdio { .. } => cfg!(feature = "stdio_transport"),
            // Additional transports go here as they become available.
            _ => false,
        });

        let transport = supported_transport
            .ok_or_else(|| anyhow!("None of the declared transports are supported in this build"))?;

        match transport {
            Transport::Wasm { path } => {
                #[cfg(feature = "wasm_loader")]
                {
                    let tool = WasmTool::new(manifest.clone(), path.clone())?;
                    self.register_tool(Arc::new(tool)).await
                }
                #[cfg(not(feature = "wasm_loader"))]
                {
                    anyhow::bail!("WASM transport requires the `wasm_loader` feature");
                }
            }
            Transport::InProcess => anyhow::bail!("InProcess transport cannot be loaded dynamically"),
            Transport::JsonRpcHttp { endpoint } => {
                #[cfg(feature = "http_client")]
                {
                    let tool = JsonRpcHttpTool::new(manifest.clone(), endpoint.clone())?;
                    self.register_tool(Arc::new(tool)).await
                }
                #[cfg(not(feature = "http_client"))]
                {
                    anyhow::bail!("JsonRpcHttp transport requires the `http_client` feature");
                }
            }
            Transport::JsonRpcStdio { exec } => {
                #[cfg(feature = "stdio_transport")]
                {
                    let tool = JsonRpcStdioTool::new(manifest.clone(), exec.clone());
                    self.register_tool(Arc::new(tool)).await
                }
                #[cfg(not(feature = "stdio_transport"))]
                {
                    anyhow::bail!("JsonRpcStdio transport requires the `stdio_transport` feature");
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// WASM-based tool shim (feature gated)
// -------------------------------------------------------------------------------------------------

#[cfg(feature = "wasm_loader")]
mod wasm {
    use super::*;
    use wasmtime::{Engine, Module, Store, Linker, Instance};

    /// Maximum number of bytes a WASM tool is allowed to return in a single
    /// call. 1 MiB should be plenty for structured JSON output while
    /// preventing accidental or malicious memory exhaustion.
    const MAX_WASM_OUTPUT_BYTES: usize = 1 * 1024 * 1024; // 1 MiB

    /// Simple WASM-hosted tool wrapper.
    pub struct WasmTool {
        manifest: ToolManifest,
        engine: Engine,
        module: std::sync::Arc<Module>,
        param_validator: Option<std::sync::Arc<jsonschema::JSONSchema>>, // Pre-compiled schema for runtime checks
    }

    impl WasmTool {
        pub fn new(manifest: ToolManifest, module_path: String) -> Result<Self> {
            // Basic validation: file exists
            if !Path::new(&module_path).exists() {
                anyhow::bail!("WASM module not found: {}", module_path);
            }

            // Use default Wasmtime config for now (no WASI, no I/O).
            let engine = Engine::default();

            // Compile module **once** at registration time – this is CPU
            // intensive so we avoid doing it on every invocation.
            let module = Module::from_file(&engine, &module_path)
                .with_context(|| format!("Compiling WASM module at {}", module_path))?;

            // Pre-compile the JSON-Schema (if any) for fast param validation.
            let param_validator = if let Some(schema) = &manifest.input_schema {
                let doc: serde_json::Value = serde_json::from_str(&schema.0)?;
                Some(std::sync::Arc::new(jsonschema::JSONSchema::compile(&doc)?))
            } else { None };

            Ok(Self {
                manifest,
                engine,
                module: std::sync::Arc::new(module),
                param_validator,
            })
        }

        /// Blocking helper – instantiate the module & invoke `execute`.
        fn run_module(&self, json_in: &str) -> Result<String> {
            // Instantiate the pre-compiled module.
            let mut store = Store::new(&self.engine, ());
            let linker = Linker::new(&self.engine);
            let instance = linker.instantiate(&mut store, &*self.module)?;

            // Fetch exported `execute` function
            let execute_func = instance.get_typed_func::<(i32, i32), (i32, i32), _>(&mut store, "execute")
                .context("`execute` export with (ptr, len) -> (ptr, len) signature not found")?;

            // Locate guest memory
            let memory = instance.get_memory(&mut store, "memory")
                .context("`memory` export not found")?;

            // Copy input JSON into guest memory
            let in_bytes = json_in.as_bytes();
            let len = in_bytes.len() as i32;
            let ptr = self.alloc_in_guest(&mut store, &instance, len)?;
            memory.write(&mut store, ptr as usize, in_bytes)?;

            // Call execute
            let (out_ptr, out_len) = execute_func.call(&mut store, (ptr, len))?;

            // Enforce length limits **before** allocating the buffer.
            if (out_len as usize) > MAX_WASM_OUTPUT_BYTES {
                anyhow::bail!("WASM tool returned {out_len} bytes which exceeds the hard limit of {MAX_WASM_OUTPUT_BYTES} bytes");
            }

            // Read output JSON
            let mut out_buf = vec![0u8; out_len as usize];
            memory.read(&mut store, out_ptr as usize, &mut out_buf)?;

            let json_out = String::from_utf8(out_buf)?;
            Ok(json_out)
        }

        /// Very naive guest allocator integration – looks for an exported `alloc`.
        fn alloc_in_guest(&self, store: &mut Store<()>, instance: &Instance, len: i32) -> Result<i32> {
            let alloc = instance.get_typed_func::<i32, i32, _>(store, "alloc")
                .context("`alloc` export not found (needed to copy params)")?;
            Ok(alloc.call(store, len)?)
        }
    }

    #[async_trait]
    impl Tool for WasmTool {
        fn name(&self) -> &str {
            &self.manifest.id
        }
        fn description(&self) -> &str {
            &self.manifest.description
        }
        fn version(&self) -> &str {
            &self.manifest.version
        }

        async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
            // Serialize params first. The tool is only allowed to receive the
            // `args` object per the schema – we forward the full struct for
            // backwards compatibility.
            let json_in = serde_json::to_string(params)?;

            // We run the heavy Wasmtime work in a blocking task, but pass only
            // cheap, reference-counted handles – not `self` by move – to avoid
            // duplicating state.
            let engine = self.engine.clone();
            let module = self.module.clone();
            let json_in_owned = json_in.clone();
            let run = move || {
                // Local shim replicating `run_module`, borrowing the moved
                // engine & module.
                let mut store = Store::new(&engine, ());
                let linker = Linker::new(&engine);
                let instance = linker.instantiate(&mut store, &module)?;

                let execute_func = instance.get_typed_func::<(i32, i32), (i32, i32), _>(&mut store, "execute")
                    .context("`execute` export with (ptr, len) -> (ptr, len) signature not found")?;

                let memory = instance.get_memory(&mut store, "memory")
                    .context("`memory` export not found")?;

                let in_bytes = json_in_owned.as_bytes();
                let len = in_bytes.len() as i32;
                // Re-use helper to allocate – we need an instance-specific call.
                let alloc = instance.get_typed_func::<i32, i32, _>(&mut store, "alloc")
                    .context("`alloc` export not found (needed to copy params)")?;
                let ptr = alloc.call(&mut store, len)?;
                memory.write(&mut store, ptr as usize, in_bytes)?;

                let (out_ptr, out_len) = execute_func.call(&mut store, (ptr, len))?;

                if (out_len as usize) > MAX_WASM_OUTPUT_BYTES {
                    anyhow::bail!("WASM tool returned {out_len} bytes which exceeds the hard limit of {MAX_WASM_OUTPUT_BYTES} bytes");
                }

                let mut out_buf = vec![0u8; out_len as usize];
                memory.read(&mut store, out_ptr as usize, &mut out_buf)?;
                Ok::<_, anyhow::Error>(String::from_utf8(out_buf)?)
            };

            let json_out = tokio::task::spawn_blocking(run).await??;

            Ok(ToolResult {
                success: true,
                output: json_out,
                metadata: ToolMetadata {
                    execution_time_ms: None, // populated by registry
                    tool_version: self.manifest.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                },
            })
        }

        fn validate_params(&self, params: &ToolParams) -> Result<()> {
            if let Some(validator) = &self.param_validator {
                let value = serde_json::to_value(&params.args)?;
                validator
                    .validate(&value)
                    .map_err(|e| anyhow!("parameter validation failed: {e}"))?;
            }
            Ok(())
        }
    }

    pub use WasmTool;
}

#[cfg(feature = "wasm_loader")]
use wasm::WasmTool;

#[cfg(feature = "http_client")]
mod http_tool {
    use super::*;
    use reqwest::Client;
    use async_trait::async_trait;

    const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
    const MAX_BODY_BYTES: usize = 1 * 1024 * 1024; // 1 MiB

    pub struct JsonRpcHttpTool {
        manifest: ToolManifest,
        endpoint: String,
        client: Client,
        param_validator: Option<std::sync::Arc<jsonschema::JSONSchema>>,
    }

    impl JsonRpcHttpTool {
        pub fn new(manifest: ToolManifest, endpoint: String) -> anyhow::Result<Self> {
            // Basic URL sanity already checked in manifest::validate, but re-check here defensively.
            let _ = url::Url::parse(&endpoint)?;
            let client = Client::builder()
                .timeout(HTTP_TIMEOUT)
                .build()?;

            // Compile param schema once.
            let param_validator = if let Some(schema) = &manifest.input_schema {
                let doc: serde_json::Value = serde_json::from_str(&schema.0)?;
                Some(std::sync::Arc::new(jsonschema::JSONSchema::compile(&doc)?))
            } else { None };

            Ok(Self { manifest, endpoint, client, param_validator })
        }
    }

    #[async_trait]
    impl Tool for JsonRpcHttpTool {
        fn name(&self) -> &str { &self.manifest.id }
        fn description(&self) -> &str { &self.manifest.description }
        fn version(&self) -> &str { &self.manifest.version }

        async fn execute(&self, params: &ToolParams) -> anyhow::Result<ToolResult> {
            // Validate params if schema present.
            if let Some(v) = &self.param_validator {
                let value = serde_json::to_value(&params.args)?;
                v.validate(&value).map_err(|e| anyhow!("parameter validation failed: {e}"))?;
            }

            // Build JSON-RPC request.
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": self.manifest.capability,
                "id": 1,
                "params": params.args,
            });

            // POST with timeout.
            let fut = self.client.post(&self.endpoint).json(&request).send();
            let resp = timeout(HTTP_TIMEOUT, fut).await??;

            // Enforce content-length / size.
            let bytes = resp.bytes().await?;
            if bytes.len() > MAX_BODY_BYTES {
                anyhow::bail!("HTTP response too large: {} bytes (limit {})", bytes.len(), MAX_BODY_BYTES);
            }

            let reply: serde_json::Value = serde_json::from_slice(&bytes)?;
            let output = reply.to_string();

            Ok(ToolResult {
                success: true,
                output,
                metadata: ToolMetadata {
                    execution_time_ms: None,
                    tool_version: self.manifest.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                },
            })
        }

        fn validate_params(&self, _params: &ToolParams) -> anyhow::Result<()> { Ok(()) }
    }

    pub use JsonRpcHttpTool;
}

#[cfg(feature = "http_client")]
use http_tool::JsonRpcHttpTool;

#[cfg(feature = "stdio_transport")]
mod stdio_tool {
    use super::*;
    use async_trait::async_trait;
    use tokio::process::Command;
    use tokio::io::AsyncReadExt;
    use tokio::time::{timeout, Duration};

    const STDIO_TIMEOUT: Duration = Duration::from_secs(10);
    const MAX_STDOUT_BYTES: usize = 1 * 1024 * 1024; // 1 MiB

    pub struct JsonRpcStdioTool {
        manifest: ToolManifest,
        exec: String,
        param_validator: Option<std::sync::Arc<jsonschema::JSONSchema>>,
    }

    impl JsonRpcStdioTool {
        pub fn new(manifest: ToolManifest, exec: String) -> Self {
            let param_validator = if let Some(schema) = &manifest.input_schema {
                if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&schema.0) {
                    Some(std::sync::Arc::new(jsonschema::JSONSchema::compile(&doc).unwrap()))
                } else { None }
            } else { None };
            Self { manifest, exec, param_validator }
        }
    }

    #[async_trait]
    impl Tool for JsonRpcStdioTool {
        fn name(&self) -> &str { &self.manifest.id }
        fn description(&self) -> &str { &self.manifest.description }
        fn version(&self) -> &str { &self.manifest.version }

        async fn execute(&self, params: &ToolParams) -> anyhow::Result<ToolResult> {
            // Validate params.
            if let Some(v) = &self.param_validator {
                let value = serde_json::to_value(&params.args)?;
                v.validate(&value).map_err(|e| anyhow!("parameter validation failed: {e}"))?;
            }

            // Build JSON-RPC request line.
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": self.manifest.capability,
                "id": 1,
                "params": params.args,
            })
            .to_string();

            let mut child = Command::new(&self.exec)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;

            // Write request to stdin.
            if let Some(mut stdin) = child.stdin.take() {
                tokio::io::AsyncWriteExt::write_all(&mut stdin, request.as_bytes()).await?;
            }

            // Wait with timeout.
            let status = timeout(STDIO_TIMEOUT, child.wait()).await??;
            if !status.success() {
                // Capture stderr for context.
                let mut stderr_buf = Vec::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_end(&mut stderr_buf).await;
                }
                let stderr_str = String::from_utf8_lossy(&stderr_buf);
                anyhow::bail!("Subprocess returned {}: {}", status, stderr_str);
            }

            // Read stdout with size cap.
            let mut stdout_buf = Vec::new();
            if let Some(mut stdout) = child.stdout.take() {
                timeout(STDIO_TIMEOUT, stdout.read_to_end(&mut stdout_buf)).await??;
            }
            if stdout_buf.len() > MAX_STDOUT_BYTES {
                anyhow::bail!("Subprocess output too large: {} bytes (limit {})", stdout_buf.len(), MAX_STDOUT_BYTES);
            }

            let output = String::from_utf8(stdout_buf)?;

            Ok(ToolResult {
                success: true,
                output,
                metadata: ToolMetadata {
                    execution_time_ms: None,
                    tool_version: self.manifest.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                },
            })
        }

        fn validate_params(&self, _params: &ToolParams) -> anyhow::Result<()> { Ok(()) }
    }

    pub use JsonRpcStdioTool;
}

#[cfg(feature = "stdio_transport")]
use stdio_tool::JsonRpcStdioTool; 