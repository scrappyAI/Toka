use crate::Event;
use anyhow::Result;
use async_trait::async_trait;

/// Convenience alias used by subscribe.
pub type Subscriber = tokio::sync::broadcast::Receiver<Event>;

/// Errors that a dispatcher implementation may return.
#[derive(thiserror::Error, Debug)]
pub enum EventError {
    /// Publishing failed (channel closed, queue full, etc.).
    #[error("Dispatch failed: {0}")]
    Dispatch(String),

    /// Generic storage / persistence failure.
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Abstraction over an event delivery mechanism.
#[async_trait]
pub trait EventDispatcher: Send + Sync {
    /// Broadcast an event to all subscribers.
    async fn publish(&self, event: Event) -> Result<(), EventError>;

    /// Subscribe to the stream of events (hot observable).
    /// Each subscriber receives **all** subsequent events (no backfill).
    async fn subscribe(&self) -> Result<Subscriber, EventError>;
}
