//! # Toka Bus
//!
//! Process-local, in-memory event propagation layer.
//! Other crates (agents, runtime, vault) interact only through this
//! abstraction rather than talking to each other directly.  In turn this
//! keeps the system loosely coupled and far easier for humans – and LLMs –
//! to comprehend.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use tokio::sync::broadcast;
use toka_bus_api::{BusEventHeader, EventPayload};
pub use toka_bus_api::{AgentEvent, ToolEvent, EventBus, EventBusExt};

// ------------------------------------------------------------------
// Default in-memory implementation – uses `tokio::broadcast`
// ------------------------------------------------------------------

const DEFAULT_BUFFER: usize = 1024;

/// In-process, non-persistent bus.
#[derive(Debug, Clone)]
pub struct MemoryBus {
    sender: broadcast::Sender<BusEventHeader>,
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self::new(DEFAULT_BUFFER)
    }
}

impl MemoryBus {
    /// Create a bus with custom ring-buffer capacity.
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    /// Legacy helper for tests – returns `Self::default()`.
    pub fn new_default() -> Self {
        Self::default()
    }

    /// Testing convenience – obtains a fresh receiver.
    pub fn get_receiver(&self) -> broadcast::Receiver<BusEventHeader> {
        self.subscribe()
    }
}

#[async_trait]
impl EventBus for MemoryBus {
    async fn publish<P: EventPayload + ?Sized>(&self, _payload: &P, kind: &str) -> Result<BusEventHeader> {
        // Note: payload is _not_ transported by the bus – only metadata.
        // Actual persistence happens in the Vault writer task.
        let header = BusEventHeader {
            id: Uuid::new_v4(),
            kind: kind.to_string(),
            timestamp: Utc::now(),
        };
        let _ = self.sender.send(header.clone());
        Ok(header)
    }

    fn subscribe(&self) -> broadcast::Receiver<BusEventHeader> {
        self.sender.subscribe()
    }
} 