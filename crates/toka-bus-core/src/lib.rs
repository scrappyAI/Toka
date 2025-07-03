#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-bus-core** – Core event bus abstraction for Toka OS.
//!
//! This crate provides the fundamental event bus traits and types used throughout
//! the Toka ecosystem. It sits at the deterministic core layer and provides
//! lightweight, in-memory event broadcasting with no persistence or I/O concerns.
//!
//! The bus abstraction allows different components to communicate via typed events
//! while maintaining loose coupling and testability.

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use toka_types::{EntityId, TaskSpec, AgentSpec};

//─────────────────────────────
//  Core kernel events
//─────────────────────────────

/// Typed kernel event enumeration emitted by the kernel after a successful
/// state transition. Each variant mirrors one opcode family from
/// `toka_types::Operation`.
///
/// These events represent the canonical "what happened" notifications that
/// flow through the system after kernel operations complete successfully.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum KernelEvent {
    /// Agent was assigned a new task
    TaskScheduled {
        /// The agent that received the task
        agent: EntityId,
        /// The task specification
        task: TaskSpec,
    },
    /// New sub-agent was spawned
    AgentSpawned {
        /// The parent agent that spawned the child
        parent: EntityId,
        /// The agent specification
        spec: AgentSpec,
    },
    /// Agent emitted observation data
    ObservationEmitted {
        /// The agent that made the observation
        agent: EntityId,
        /// The observation data
        data: Vec<u8>,
    },
}

//─────────────────────────────
//  Event bus trait
//─────────────────────────────

/// Core event bus abstraction for publishing and subscribing to kernel events.
///
/// The bus provides a simple publish-subscribe mechanism that allows different
/// components to communicate asynchronously while maintaining loose coupling.
/// All implementations must be thread-safe and support multiple subscribers.
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers.
    ///
    /// This operation should complete quickly and not block the caller.
    /// If subscribers are slow or unavailable, the bus may drop events
    /// to maintain system responsiveness.
    fn publish(&self, event: &KernelEvent) -> Result<()>;

    /// Subscribe to the live event stream.
    ///
    /// Returns a receiver that will receive copies of all events published
    /// after the subscription was created. Subscribers that fall behind
    /// may miss events if the bus buffer overflows.
    fn subscribe(&self) -> broadcast::Receiver<KernelEvent>;
}

//─────────────────────────────
//  In-memory bus implementation
//─────────────────────────────

/// Simple in-memory, broadcast-only event bus using Tokio channels.
///
/// This implementation uses a ring buffer to store recent events and broadcasts
/// them to all active subscribers. It provides good performance for scenarios
/// where events don't need persistence.
#[derive(Debug, Clone)]
pub struct InMemoryBus {
    tx: Arc<broadcast::Sender<KernelEvent>>,
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl InMemoryBus {
    /// Create a new in-memory bus with the specified ring buffer capacity.
    ///
    /// The capacity determines how many events can be buffered for slow
    /// subscribers before older events are dropped.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }

    /// Get the current number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl EventBus for InMemoryBus {
    fn publish(&self, event: &KernelEvent) -> Result<()> {
        // Ignore lagging receiver errors - subscribers must handle missed events
        let _ = self.tx.send(event.clone());
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<KernelEvent> {
        self.tx.subscribe()
    }
}

//─────────────────────────────
//  Error types
//─────────────────────────────

/// Errors that can occur during bus operations.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    /// Event could not be published
    #[error("failed to publish event: {0}")]
    PublishFailed(String),
    /// Subscription failed
    #[error("failed to create subscription: {0}")]
    SubscriptionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::RecvError;

    #[tokio::test]
    async fn test_in_memory_bus_basic_flow() {
        let bus = InMemoryBus::new(16);
        let mut rx = bus.subscribe();

        let event = KernelEvent::TaskScheduled {
            agent: EntityId(123),
            task: TaskSpec {
                description: "test task".to_string(),
            },
        };

        // Publish event
        bus.publish(&event).unwrap();

        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = InMemoryBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        let event = KernelEvent::AgentSpawned {
            parent: EntityId(1),
            spec: AgentSpec {
                name: "test-agent".to_string(),
            },
        };

        bus.publish(&event).unwrap();

        // Both subscribers should receive the event
        assert_eq!(rx1.recv().await.unwrap(), event);
        assert_eq!(rx2.recv().await.unwrap(), event);
    }

    #[tokio::test]
    async fn test_buffer_overflow() {
        let bus = InMemoryBus::new(2); // Very small buffer
        let mut rx = bus.subscribe();

        // Fill buffer beyond capacity
        for i in 0..5 {
            let event = KernelEvent::ObservationEmitted {
                agent: EntityId(i as u128),
                data: vec![i as u8],
            };
            bus.publish(&event).unwrap();
        }

        // First few events should be lost due to buffer overflow
        match rx.recv().await {
            Ok(_) => {
                // Successfully received an event - continue receiving
                while let Ok(_) = rx.recv().await {
                    // Keep draining
                }
            }
            Err(RecvError::Lagged(_)) => {
                // Expected - some events were dropped
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}