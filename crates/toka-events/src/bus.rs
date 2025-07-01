use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use anyhow::Result;
use std::sync::Arc;

use toka_types::{EntityId, TaskSpec, AgentSpec, Role};

/// Typed kernel event enumeration (v0.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    /* — financial — */
    FundsTransferred { from: EntityId, to: EntityId, amount: u64 },
    AssetMinted      { asset: EntityId, to: EntityId, amount: u64 },
    AssetBurned      { asset: EntityId, from: EntityId, amount: u64 },

    /* — agent — */
    TaskScheduled  { agent: EntityId, task: TaskSpec },
    AgentSpawned   { parent: EntityId, spec: AgentSpec },
    ObservationEmitted { agent: EntityId, data: Vec<u8> },

    /* — user — */
    UserCreated { alias: String, id: EntityId },
    RoleAssigned { user: EntityId, role: Role },
}

/// Minimal event bus interface for the kernel and clients.
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers.
    fn publish(&self, event: &Event) -> Result<()>;

    /// Subscribe to the live event stream.
    fn subscribe(&self) -> broadcast::Receiver<Event>;
}

/// Simple in-memory, broadcast-only event bus using Tokio.
#[derive(Debug, Clone)]
pub struct InMemoryBus {
    tx: Arc<broadcast::Sender<Event>>,
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl InMemoryBus {
    /// Create a bus with the given ring-buffer `capacity`.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }
}

impl EventBus for InMemoryBus {
    fn publish(&self, event: &Event) -> Result<()> {
        // Ignore lagging error for now (subscriber must handle).
        let _ = self.tx.send(event.clone());
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}