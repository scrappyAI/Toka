//! Tool Manifest / Specification
//!
//! Provides versioned, serialisable data-structures that describe a tool's
//! public contract.  Stored as JSON (or embedded YAML/TOML) and compatible with
//! existing ecosystems:
//!   • JSON-RPC 2.0 (method names map to `capability`)
//!   • Google A2A / App Actions (`action_id`) – experimental
//!   • MCP (Multipurpose Control Protocol) – transport enumeration
//!
//! The manifest is *stable*-ish: breaking changes bump `SCHEMA_VERSION`.

use serde::{Deserialize, Serialize};

/// Current schema version – increment **major** on breaking changes.
pub const SCHEMA_VERSION: &str = "1.1";

/// Supported higher-level protocol mapping (MCP / A2A) so external frameworks
/// can automatically translate the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum ProtocolMapping {
    /// Model Context Protocol (Anthropic) function mapping
    /// – `function_name` becomes the MCP `call` field.
    Mcp {
        /// JSON-RPC method name advertised via MCP (often same as capability).
        function_name: String,
        /// Target MCP version (semver).  Default = "1".
        #[serde(default = "default_mcp_version", skip_serializing_if = "is_default_mcp_version")]
        version: String,
    },
    /// Google Agent-to-Agent protocol action mapping.
    A2a {
        /// The `action` identifier as defined by A2A spec.
        action: String,
        /// Optional A2A spec version.
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
}

fn default_mcp_version() -> String { "1".into() }
fn is_default_mcp_version(v: &String) -> bool { v == "1" }

/// Where & how the tool can be invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Transport {
    /// JSON-RPC 2.0 over HTTP(S).  `endpoint` must be absolute URI.
    JsonRpcHttp { endpoint: String },
    /// JSON-RPC 2.0 over stdio (command-line programs).
    JsonRpcStdio { exec: String },
    /// In-process Rust struct implementing the [`Tool`](crate::Tool) trait.
    InProcess,
    /// WebAssembly module exposing `execute` function.
    Wasm { path: String },
}

/// Side-effect characteristics used for audit & scheduling policy.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideEffect {
    #[default]
    None,
    ReadOnly,
    Idempotent,
    External,   // network or fs writes
    Privileged, // requires elevated authz/sandbox
}

/// Input or output schema description.
/// For now this is opaque JSON Schema (draft-07) represented as a raw string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema(pub String);

/// Top-level manifest object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// Hard-coded identifier (`crate::name` or full reverse-DNS).
    pub id: String,
    /// Human name.
    pub name: String,
    /// Tool semantic version.
    pub version: String,
    /// Short description.
    pub description: String,

    /// Declared capability (maps to JSON-RPC method when using that transport).
    pub capability: String,
    /// Side-effects classification.
    #[serde(default)]
    pub side_effect: SideEffect,

    /// JSON Schema for parameters (nullable when none).
    pub input_schema: Option<Schema>,
    /// JSON Schema for successful result.
    pub output_schema: Option<Schema>,

    /// Transport options – at least one.
    pub transports: Vec<Transport>,

    /// Optional mapping for Google App Actions / A2A.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,

    /// Schema version for forward/backward compat.
    #[serde(default = "schema_version")]
    pub manifest_version: String,

    /// Supported external protocol mappings (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protocols: Vec<ProtocolMapping>,

    /// Arbitrary extension metadata for future or domain-specific keys.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub metadata: std::collections::BTreeMap<String, String>,
}

fn schema_version() -> String {
    SCHEMA_VERSION.to_string()
}

impl ToolManifest {
    /// Perform static validation of the manifest.
    ///
    /// This **does not** validate JSON Schema payloads – only structural
    /// rules enforced by Toka:
    ///  * `id`, `name`, `version`, `capability` non-empty
    ///  * at least one transport
    ///  * when `Transport::JsonRpcHttp` endpoint must be valid URL
    ///  * protocols list may be empty but mapping consistency is checked when present.
    pub fn validate(&self) -> anyhow::Result<()> {
        use anyhow::{anyhow, Context};
        if self.id.trim().is_empty() {
            return Err(anyhow!("manifest.id must not be empty"));
        }
        if self.name.trim().is_empty() {
            return Err(anyhow!("manifest.name must not be empty"));
        }
        if self.version.trim().is_empty() {
            return Err(anyhow!("manifest.version must not be empty"));
        }
        if self.capability.trim().is_empty() {
            return Err(anyhow!("manifest.capability must not be empty"));
        }
        if self.transports.is_empty() {
            return Err(anyhow!("at least one transport must be specified"));
        }

        for t in &self.transports {
            if let Transport::JsonRpcHttp { endpoint } = t {
                let url = url::Url::parse(endpoint)
                    .with_context(|| format!("invalid JsonRpcHttp endpoint: {}", endpoint))?;
                if url.scheme() != "http" && url.scheme() != "https" {
                    return Err(anyhow!("JsonRpcHttp endpoint must be http(s)"));
                }
            }
        }

        // Ensure at most one mapping per protocol kind
        let mut seen_mcp = false;
        let mut seen_a2a = false;
        for p in &self.protocols {
            match p {
                ProtocolMapping::Mcp { .. } => {
                    if seen_mcp {
                        return Err(anyhow!("duplicate MCP mapping"));
                    }
                    seen_mcp = true;
                }
                ProtocolMapping::A2a { .. } => {
                    if seen_a2a {
                        return Err(anyhow!("duplicate A2A mapping"));
                    }
                    seen_a2a = true;
                }
            }
        }

        // Deep-validate JSON Schemas (draft-07) when supplied.
        ensure_schema_compiles(&self.input_schema, "input")?;
        ensure_schema_compiles(&self.output_schema, "output")?;

        Ok(())
    }
}

/// Compile-time size limit (bytes) applied to every embedded JSON Schema.
const MAX_SCHEMA_BYTES: usize = 65_536; // 64 KiB

// Feature-gated constants controlling optional behaviour.

// Remote `$ref`s are **rejected** unless the `allow_remote_refs` feature is
// enabled.  This prevents unbounded network fetches during manifest loading
// and aligns with the HardenSecuritySurface guidelines.
#[cfg(feature = "allow_remote_refs")]
const ALLOW_REMOTE_REFS: bool = true;
#[cfg(not(feature = "allow_remote_refs"))]
const ALLOW_REMOTE_REFS: bool = false;

/// Ensures that an optional schema string compiles under JSON-Schema draft-07.
///
/// Security & UX guarantees:
///  • Rejects oversized schemas (> 64 KiB) to mitigate DoS vectors.
///  • Denies remote `$ref` unless `allow_remote_refs` feature is active.
///  • Provides precise error context for tool authors.
///  • Optional LRU-style cache behind the `schema_cache` feature to amortise
///    compile overhead across manifests.
fn ensure_schema_compiles(opt: &Option<Schema>, which: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    use jsonschema::Draft;

    let raw = match opt {
        Some(s) => &s.0,
        None => return Ok(()),
    };

    if raw.len() > MAX_SCHEMA_BYTES {
        anyhow::bail!("{which} schema exceeds {MAX_SCHEMA_BYTES} bytes ({} bytes)", raw.len());
    }

    // Fast JSON parse – we need a `serde_json::Value` anyway for the compiler.
    let doc: serde_json::Value = serde_json::from_str(raw)
        .with_context(|| format!("{which} schema: invalid JSON"))?;

    // Shallow scan for remote `$ref` before expensive compilation to avoid
    // network IO surprises.  We only need to walk objects & arrays.
    if !ALLOW_REMOTE_REFS && contains_remote_ref(&doc) {
        anyhow::bail!(
            "{which} schema: remote $ref URLs are disabled – enable the \
             `allow_remote_refs` feature to override"
        );
    }

    // --------------------------- Optional cache ---------------------------
    #[cfg(feature = "schema_cache")]
    {
        use dashmap::DashMap;
        use once_cell::sync::Lazy;
        use std::hash::{Hash, Hasher};
        use std::sync::Arc;

        static SCHEMA_CACHE: Lazy<DashMap<u64, Arc<jsonschema::JSONSchema>>> =
            Lazy::new(DashMap::new);

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        raw.hash(&mut hasher);
        let key = hasher.finish();

        if SCHEMA_CACHE.contains_key(&key) {
            return Ok(());
        }

        let compiled = jsonschema::JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&doc)
            .with_context(|| format!("{which} schema: invalid draft-07"))?;

        SCHEMA_CACHE.insert(key, Arc::new(compiled));
        return Ok(());
    }

    // --------------------------- No-cache path ----------------------------
    #[cfg(not(feature = "schema_cache"))]
    {
        jsonschema::JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&doc)
            .with_context(|| format!("{which} schema: invalid draft-07"))?;
        Ok(())
    }
}

/// Recursively checks whether a JSON value contains a remote `$ref`.
fn contains_remote_ref(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(s)) = map.get("$ref") {
                if s.starts_with("http://") || s.starts_with("https://") {
                    return true;
                }
            }
            map.values().any(contains_remote_ref)
        }
        serde_json::Value::Array(arr) => arr.iter().any(contains_remote_ref),
        _ => false,
    }
} 