//! Toka Agents – default agent implementations
//!
//! This crate provides out-of-the-box agents (e.g. `SymbolicAgent`) that can be compiled
//! into `toka-runtime` when the `agents-core` feature is enabled.  The implementation is
//! intentionally free of heavy external dependencies so the runtime can remain lean.

pub mod agent;
pub mod bundle;
pub mod metadata;
pub mod prelude;
pub mod reasoning;
pub mod system;

pub use agent::{BaseAgent, Belief, Observation, SymbolicAgent};
pub use bundle::{AgentBundle, ToolSpec};
pub use metadata::{AgentMetadata, Capability};
pub use prelude::*;
pub use reasoning::{AgentContext, NoOpReasoner, ReasonOutcome, ReasoningEngine};
pub use system::{SystemAgent, SystemAgentKind};

pub use toka_events::{AgentEvent, EventBus};

use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use toka_security_vault::MemoryAdapter;

// -----------------------------------------------------------------------------
//  Minimal local EventBus + AgentEvent stubs
// -----------------------------------------------------------------------------
// (removed; use toka_events)

/// Core Agent behaviour required by `toka-runtime`.
///
/// This minimal trait purposefully keeps the surface area lean so that
/// downstream runtimes can embed any kind of agent implementation without
/// incurring heavy dep-graph costs.
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

// -----------------------------------------------------------------------------
// Blanket implementation for `SymbolicAgent`
// -----------------------------------------------------------------------------

#[async_trait]
impl Agent for BaseAgent {
    fn name(&self) -> &str {
        &self.id
    }

    async fn process_event(&mut self, event_type: &str, _event_data: &str) -> Result<()> {
        // Extremely naive handling for now – symbolic agents simply log the
        // event class and update an internal counter when the event matches
        // their ID.  A real implementation would parse JSON, update beliefs,
        // etc.

        if event_type.contains(&self.id) {
            // Self-referential event – treat as positive observation.
            let obs = crate::Observation {
                key: event_type.to_string(),
                evidence_strength: 1.2,
                supports: true,
            };
            self.observe(obs).await?;
        }
        Ok(())
    }

    async fn save_state(&self, adapter: &dyn MemoryAdapter) -> Result<()> {
        let key = format!("agent:{}", self.id);
        let json = serde_json::to_string(self)?;
        adapter.save_json(&key, &json)
    }

    async fn load_state(&mut self, adapter: &dyn MemoryAdapter) -> Result<()> {
        let key = format!("agent:{}", self.id);
        if let Some(json) = adapter.load_json(&key)? {
            if let Ok(saved) = serde_json::from_str::<BaseAgent>(&json) {
                self.context = saved.context;
                self.reasoner = saved.reasoner;
            }
        }
        Ok(())
    }
}
