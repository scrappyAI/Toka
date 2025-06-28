//! Toka Agents – API crate
//!
//! This crate provides the **pure data types**, flags and minimal trait
//! contracts required to interact with agent implementations in the Toka
//! platform.  No heavy dependencies – only optional `serde`, `async` and
//! `uuid/semver` helpers behind feature flags – so the crate is suitable for
//! lightweight, embedded or `no_std` environments.
//!
//! # Feature Flags
//! * `serde` *(default)* – derive `Serialize`/`Deserialize` for all public
//!   structs and enable JSON helpers.
//! * `async` *(default)* – include the async `Agent` trait extension that
//!   depends on `async-trait`, `anyhow` and the `toka-memory-api` crate.
//! * `std`  *(default)* – link against the Rust standard library.  Disable to
//!   compile for `no_std` targets.
//!
//! # Layering
//! This crate lives in the **ApiLayer** as per the workspace rules.  It must
//! therefore remain free of heavyweight dependencies and implementation code.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "std")] extern crate std;
extern crate alloc;

use alloc::string::String;

#[cfg(feature = "serde")] use serde::{Serialize, Deserialize};
#[cfg(feature = "serde")] use serde_json;
#[cfg(feature = "serde")] use alloc::vec::Vec;

// -----------------------------------------------------------------------------
// Capability flags
// -----------------------------------------------------------------------------
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
bitflags::bitflags! {
    /// Capability flags declare which subsystems an agent may access.
    /// The flags can be combined using `|` and round-trip as their underlying
    /// bit representation when `serde` is enabled.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", serde(transparent))]
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

// -----------------------------------------------------------------------------
// Core data types
// -----------------------------------------------------------------------------

/// Bayesian belief state attached to a hypothesis key.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Belief {
    /// Current probability in the range \[0.0, 1.0].
    pub probability: f64,
    /// Last update timestamp (seconds since Unix epoch).
    pub last_updated: u64,
}

/// Single observation used for Bayesian updates.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    /// Arbitrary belief key e.g. `"sky_is_blue"`.
    pub key: String,
    /// Strength of evidence (likelihood ratio).  Must be `> 0`.
    pub evidence_strength: f64,
    /// `true` if the observation *supports* the hypothesis, `false` if it
    /// *contradicts* it.
    pub supports: bool,
}

// -----------------------------------------------------------------------------
// Metadata & packaging
// -----------------------------------------------------------------------------

#[cfg(feature = "serde")] use semver::Version;
#[cfg(feature = "serde")] use uuid::Uuid;

/// Static, immutable metadata describing an agent implementation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct AgentMetadata {
    /// Stable runtime identifier (uuid v4).
    pub id: Uuid,
    /// Human-readable label.
    pub name: String,
    /// One-sentence summary of the agent's purpose.
    pub description: String,
    /// Semantic version denoting implementation revision.
    pub version: Version,
    /// Advertised capability flags.  Defaults to empty.
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: Capability,
}

#[cfg(feature = "serde")] use semver::VersionReq;

/// Declarative specification of a tool dependency.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSpec {
    /// Canonical tool name (matches `Tool::name()`).
    pub name: String,
    /// Semantic version requirement (e.g. "^1.2.0").
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub version_req: Option<VersionReq>,
}

/// Portable, signed-off package that fully describes a deployable agent.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct AgentBundle {
    /// Static metadata describing the agent implementation.
    pub metadata: AgentMetadata,
    /// Declared tool dependencies.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tools: alloc::vec::Vec<ToolSpec>,
    /// Arbitrary agent-specific configuration (opaque to the runtime).
    #[cfg_attr(feature = "serde", serde(default))]
    pub config: serde_json::Value,
}

impl AgentBundle {
    /// Create a minimal agent bundle with no tools and default version
    /// `"0.1.0"`.
    #[cfg(feature = "serde")]
    pub fn new(name: &str, description: &str, capabilities: Capability) -> Self {
        Self {
            metadata: AgentMetadata {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: description.to_string(),
                version: Version::new(0, 1, 0),
                capabilities,
            },
            tools: alloc::vec::Vec::new(),
            config: serde_json::Value::Null,
        }
    }

    /// Serialize the bundle to a pretty-printed JSON string.
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Restore an `AgentBundle` from JSON.
    #[cfg(feature = "serde")]
    pub fn from_json<S: AsRef<str>>(s: S) -> serde_json::Result<Self> {
        serde_json::from_str(s.as_ref())
    }
}

// -----------------------------------------------------------------------------
// Agent trait contract (async optional)
// -----------------------------------------------------------------------------

#[cfg(feature = "async")]
mod agent_trait {
    use super::*;
    use toka_memory_api::MemoryAdapter;
    use async_trait::async_trait;
    use anyhow::Result;

    /// Core behaviour required by the Toka runtime.
    #[async_trait]
    pub trait Agent: Send + Sync {
        /// Human-readable name / identifier for logging & debugging.
        fn name(&self) -> &str;

        /// Consume an event emitted by the runtime or other agents.
        ///
        /// The default contract is *best-effort* processing – implementors
        /// should swallow non-fatal errors and report diagnostics rather than
        /// panic.  A returned `Err` will be bubbled up to the runtime manager
        /// for centralised handling.
        async fn process_event(&mut self, event_type: &str, event_data: &str) -> Result<()>;

        /// Persist internal state via provided adapter.
        async fn save_state(&self, adapter: &dyn MemoryAdapter) -> Result<()>;

        /// Restore internal state from adapter (if exists).
        async fn load_state(&mut self, adapter: &dyn MemoryAdapter) -> Result<()>;
    }

    // Re-export so consumers use `toka_agents_api::Agent` directly.
    pub use Agent as _;
}

#[cfg(feature = "async")]
pub use agent_trait::Agent; 