use serde::{Deserialize, Serialize};
use semver::{Version, VersionReq};
use uuid::Uuid;

use crate::metadata::{AgentMetadata, Capability};

/// Declarative specification of a tool dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Canonical tool name (matches `Tool::name()`).
    pub name: String,
    /// Semantic version requirement (e.g. "^1.2.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_req: Option<VersionReq>,
}

/// A portable, signed-off package that fully describes a deployable agent.
///
/// The bundle is meant to be serialized to JSON or CBOR and later restored by
/// the runtime (or CLI) to create & register an agent plus all required tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBundle {
    /// Static metadata about the agent implementation.
    pub metadata: AgentMetadata,
    /// Declared tool dependencies.
    #[serde(default)]
    pub tools: Vec<ToolSpec>,
    /// Arbitrary agent-specific configuration (opaque to the runtime).
    #[serde(default)]
    pub config: serde_json::Value,
}

impl AgentBundle {
    /// Create a minimal agent bundle with no tools and default version "0.1.0".
    pub fn new(name: &str, description: &str, capabilities: Capability) -> Self {
        Self {
            metadata: AgentMetadata {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: description.to_string(),
                version: Version::new(0, 1, 0),
                capabilities,
            },
            tools: Vec::new(),
            config: serde_json::Value::Null,
        }
    }

    /// Serialize the bundle to a pretty-printed JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> { serde_json::to_string_pretty(self) }

    /// Restore an `AgentBundle` from JSON.
    pub fn from_json<S: AsRef<str>>(s: S) -> serde_json::Result<Self> {
        serde_json::from_str(s.as_ref())
    }
} 