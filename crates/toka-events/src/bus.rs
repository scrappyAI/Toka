//! Lightweight, in-memory event **bus** used by the kernel for the v0.1
//! milestone.  It provides a simple broadcast channel so any subsystem can
//! subscribe to live `KernelEvent`s.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use anyhow::Result;
use std::sync::Arc;

use toka_types::{EntityId, Hash256, PubKey, Capability, HandlerRef};

/// Typed kernel event enumeration emitted by the kernel after a successful
/// state transition.  Each variant mirrors one opcode family from
/// `toka-types::Operation`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    EntityCreated { template: Hash256, id: EntityId },
    EntityDeleted { id: EntityId },
    CapabilityGranted { to: PubKey, cap: Capability },
    CapabilityRevoked { cap_id: Hash256 },
    BatchSubmitted { count: usize },
    EventEmitted { topic: String, data: Vec<u8> },
    HandlerRegistered { range_start: u8, range_end: u8, entry: HandlerRef },
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