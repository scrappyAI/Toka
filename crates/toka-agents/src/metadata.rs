use serde::{Deserialize, Serialize};
use uuid::Uuid;
use semver::Version;
use bitflags::bitflags;

bitflags! {
    /// Capability flags declare which subsystems an agent may access.
    /// Marked with `serde` helper so flags round-trip as underlying bits.
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Capability: u32 {
        /// Agent is allowed to invoke registered tools via `ToolRegistry`.
        const TOOL_USE  = 0b0001;
        /// Agent may access a Vault implementation for secrets/state.
        const VAULT     = 0b0010;
        /// Agent maintains long-term memory through a `MemoryAdapter`.
        const MEMORY    = 0b0100;
        /// Agent owns a `ReasoningEngine` and may perform cognitive work.
        const REASONING = 0b1000;
    }
}

/// Static, immutable metadata describing an agent instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMetadata {
    /// Stable runtime identifier (uuid v4).
    pub id: Uuid,
    /// Human-readable label.
    pub name: String,
    /// One-sentence summary of the agent's purpose.
    pub description: String,
    /// Semantic version denoting implementation rev.
    pub version: Version,
    /// Advertised capability flags. Defaults to empty.
    #[serde(default)]
    pub capabilities: Capability,
} 