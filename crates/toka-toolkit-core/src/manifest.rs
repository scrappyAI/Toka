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
pub const SCHEMA_VERSION: &str = "1.0";

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
}

fn schema_version() -> String {
    SCHEMA_VERSION.to_string()
} 