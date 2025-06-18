//! Toka Agents – default agent implementations
//!
//! This crate provides out-of-the-box agents (e.g. `SymbolicAgent`) that can be compiled
//! into `toka-runtime` when the `agents-core` feature is enabled.  The implementation is
//! intentionally free of heavy external dependencies so the runtime can remain lean.

pub mod symbolic;
pub mod prelude;

pub use symbolic::{SymbolicAgent, Observation, Belief};
pub use prelude::*;

// -----------------------------------------------------------------------------
//  Minimal local EventBus + AgentEvent stubs
// -----------------------------------------------------------------------------
// These are temporary placeholders until the unified `toka-events` crate is
// integrated with the runtime.  They let the agent implementation compile in
// isolation.  Downstream code can choose to ignore them or adapt as needed.

use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Created { agent_id: String, agent_type: String, timestamp: u64 },
    BeliefUpdated { agent_id: String, belief_key: String, probability: f64, timestamp: u64 },
    ActionTriggered { agent_id: String, action: String, timestamp: u64 },
    PlanGenerated { agent_id: String, plan: String, timestamp: u64 },
    ObservationProcessed { agent_id: String, observation_key: String, timestamp: u64 },
}

#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
    subscribers: Arc<RwLock<Vec<broadcast::Sender<AgentEvent>>>>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender, subscribers: Arc::new(RwLock::new(Vec::new())) }
    }

    pub async fn emit_agent_event(&self, ev: AgentEvent, _source: &str) -> Result<()> {
        let _ = self.sender.send(ev.clone());
        let subs = self.subscribers.read().await;
        for sub in subs.iter() {
            let _ = sub.send(ev.clone());
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        let rx = self.sender.subscribe();
        let mut subs = self.subscribers.write().await;
        subs.push(self.sender.clone());
        rx
    }
}

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
}

// -----------------------------------------------------------------------------
// Blanket implementation for `SymbolicAgent`
// -----------------------------------------------------------------------------

#[async_trait]
impl Agent for SymbolicAgent {
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
            let obs = crate::Observation { key: event_type.to_string(), evidence_strength: 1.2, supports: true };
            self.observe(obs).await?;
        }
        Ok(())
    }
} 