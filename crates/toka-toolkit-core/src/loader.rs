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

        let transport = manifest
            .transports
            .get(0)
            .ok_or_else(|| anyhow!("manifest.transports must not be empty"))?;

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
            Transport::JsonRpcHttp { .. } | Transport::JsonRpcStdio { .. } => {
                anyhow::bail!("Transport {:?} not yet supported", transport);
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
    use wasmtime::{Engine, Module, Store, Caller, Linker, Func, Memory, Config, Instance};

    /// Simple WASM-hosted tool wrapper.
    pub struct WasmTool {
        manifest: ToolManifest,
        module_path: String,
        engine: Engine,
    }

    impl WasmTool {
        pub fn new(manifest: ToolManifest, module_path: String) -> Result<Self> {
            // Basic validation: file exists
            if !Path::new(&module_path).exists() {
                anyhow::bail!("WASM module not found: {}", module_path);
            }

            // Use default Wasmtime config for now (no WASI, no I/O).
            let engine = Engine::default();
            Ok(Self { manifest, module_path, engine })
        }

        /// Blocking helper – compile & instantiate the module, invoke `execute`.
        fn run_module(&self, json_in: &str) -> Result<String> {
            // Compile module (could be cached per module_path in future).
            let module = Module::from_file(&self.engine, &self.module_path)
                .with_context(|| format!("Compiling WASM module at {}", self.module_path))?;
            let mut store = Store::new(&self.engine, ());
            let linker = Linker::new(&self.engine);
            let instance = linker.instantiate(&mut store, &module)?;

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
            let json_in = serde_json::to_string(params)?;
            let json_out = tokio::task::spawn_blocking(move || self.run_module(&json_in))
                .await??;

            Ok(ToolResult {
                success: true,
                output: json_out,
                metadata: ToolMetadata {
                    execution_time_ms: 0, // updated by caller
                    tool_version: self.manifest.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                },
            })
        }

        fn validate_params(&self, _params: &ToolParams) -> Result<()> {
            // For now we rely on WASM guest to validate – registry has already validated against schema.
            Ok(())
        }
    }

    pub use WasmTool;
}

#[cfg(feature = "wasm_loader")]
use wasm::WasmTool; 