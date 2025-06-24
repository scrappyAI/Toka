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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Blanket serialization bound used by the bus ecosystem.
/// Identical to the trait in `toka-vault` but re-defined here so the bus
/// crate stays independent.
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}
impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Minimal header metadata broadcast with every event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEventHeader {
    /// Unique event identifier (uuid v4).
    pub id: Uuid,
    /// Application-defined kind, e.g. `tool.invoked`.
    pub kind: String,
    /// Wall-clock timestamp when the event was published.
    pub timestamp: DateTime<Utc>,
}

/// Core abstraction – an async, multi-producer multi-consumer bus.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish a payload of type `P` under `kind` and receive the resulting
    /// header for reference / causal linking.
    async fn publish<P: EventPayload + ?Sized>(&self, payload: &P, kind: &str) -> Result<BusEventHeader>;

    /// Obtain a live subscription of event headers in **publish order**.
    fn subscribe(&self) -> broadcast::Receiver<BusEventHeader>;
}

/// Agent-centric event types commonly used by agents & runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// Agent's belief state has been updated
    BeliefUpdated { 
        /// The agent identifier
        agent_id: String, 
        /// The belief key that was updated
        belief_key: String, 
        /// The new probability value
        probability: f64, 
        /// Unix timestamp when the belief was updated
        timestamp: u64 
    },
    /// Agent has generated a new plan
    PlanGenerated { 
        /// The agent identifier
        agent_id: String, 
        /// The generated plan text
        plan: String, 
        /// Unix timestamp when the plan was generated
        timestamp: u64 
    },
    /// Agent has triggered an action
    ActionTriggered { 
        /// The agent identifier
        agent_id: String, 
        /// The action that was triggered
        action: String, 
        /// Unix timestamp when the action was triggered
        timestamp: u64 
    },
}

/// Tool lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEvent {
    /// Tool was invoked by a user
    Invoked { 
        /// Name of the tool that was invoked
        tool_name: String, 
        /// User who invoked the tool
        user_id: String, 
        /// Unix timestamp when the tool was invoked
        timestamp: u64 
    },
    /// Tool execution completed
    Completed { 
        /// Name of the tool that completed
        tool_name: String, 
        /// User who invoked the tool
        user_id: String, 
        /// Execution duration in milliseconds
        duration_ms: u64, 
        /// Whether the tool execution was successful
        success: bool, 
        /// Unix timestamp when the tool completed
        timestamp: u64 
    },
    /// Tool execution encountered an error
    Error { 
        /// Name of the tool that errored
        tool_name: String, 
        /// User who invoked the tool
        user_id: String, 
        /// Error message
        error: String, 
        /// Unix timestamp when the error occurred
        timestamp: u64 
    },
}

// ------------------------------------------------------------------
// Convenience extension trait – emitted events will use canonical kind strings
// ------------------------------------------------------------------

/// Extension trait providing convenience methods for emitting specific event types
#[async_trait]
pub trait EventBusExt: EventBus {
    /// Emit an agent event with canonical kind string formatting
    async fn emit_agent_event(&self, ev: AgentEvent, subject: &str) -> Result<BusEventHeader> {
        self.publish(&ev, &format!("agent.{subject}" )).await
    }

    /// Emit a tool event with canonical kind string formatting
    async fn emit_tool_event(&self, ev: ToolEvent, subject: &str) -> Result<BusEventHeader> {
        self.publish(&ev, &format!("tool.{subject}")).await
    }
}

impl<T: EventBus + ?Sized> EventBusExt for T {}

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