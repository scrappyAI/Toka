//! Bus API crate – defines the contracts and core types for Toka's in-process
//! event bus.  Keep this crate lean so it can be consumed by binaries,
//! libraries, or even WASM targets without dragging in Tokio.
//!
//! # Features
//! * `serde-support` – enables `serde::{Serialize, Deserialize}` for all public
//!   types.
//! * `async` – pulls in `async_trait` + `anyhow` and exposes the async traits.
//!
//! All features are opt-in – use `default-features = false` for strict `no_std`
//! targets.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;
use alloc::boxed::Box;
use alloc::format;

#[cfg(feature = "serde-support")] use serde::{Serialize, Deserialize};
#[cfg(feature = "async")] use async_trait::async_trait;
#[cfg(feature = "async")] use anyhow::Result;
#[cfg(feature = "serde-support")] use chrono::{DateTime, Utc};
#[cfg(feature = "serde-support")] use uuid::Uuid;
#[cfg(feature = "async")] use tokio::sync::broadcast;

/// Blanket serialization bound required for bus payloads.
#[cfg(feature = "serde-support")]
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}
#[cfg(feature = "serde-support")]
impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Header metadata broadcast with every bus event.
#[cfg(feature = "serde-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEventHeader {
    /// Unique event identifier (UUID v4).
    pub id: Uuid,
    /// Application-defined kind string, e.g. `tool.invoked`.
    pub kind: alloc::string::String,
    /// Wall-clock timestamp when the event was published.
    pub timestamp: DateTime<Utc>,
}

/// Core abstraction – an async, multi-producer multi-consumer bus.
#[cfg(feature = "async")]
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish a payload and receive the resulting [`BusEventHeader`].
    async fn publish<P: EventPayload + ?Sized>(&self, payload: &P, kind: &str) -> Result<BusEventHeader>;

    /// Obtain a live subscription of event headers in **publish order**.
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BusEventHeader>;
}

// -------------------------------------------------------------------------------------------------
// Optional higher-level events used by agents & tools (kept here for convenience)
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "serde-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// Agent's belief state was updated.
    BeliefUpdated { agent_id: alloc::string::String, belief_key: alloc::string::String, probability: f64, timestamp: u64 },
    /// Agent generated a new plan.
    PlanGenerated  { agent_id: alloc::string::String, plan: alloc::string::String, timestamp: u64 },
    /// Agent triggered an action.
    ActionTriggered { agent_id: alloc::string::String, action: alloc::string::String, timestamp: u64 },
}

#[cfg(feature = "serde-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEvent {
    /// Tool invocation.
    Invoked   { tool_name: alloc::string::String, user_id: alloc::string::String, timestamp: u64 },
    /// Tool successful completion.
    Completed { tool_name: alloc::string::String, user_id: alloc::string::String, duration_ms: u64, success: bool, timestamp: u64 },
    /// Tool error.
    Error     { tool_name: alloc::string::String, user_id: alloc::string::String, error: alloc::string::String, timestamp: u64 },
}

/// Convenience helpers for publishing canonical events.
#[cfg(all(feature = "async", feature = "serde-support"))]
#[async_trait]
pub trait EventBusExt: EventBus {
    /// Emit an [`AgentEvent`].
    async fn emit_agent_event(&self, ev: AgentEvent, subject: &str) -> Result<BusEventHeader> {
        self.publish(&ev, &alloc::format!("agent.{subject}")).await
    }
    /// Emit a [`ToolEvent`].
    async fn emit_tool_event(&self, ev: ToolEvent, subject: &str) -> Result<BusEventHeader> {
        self.publish(&ev, &alloc::format!("tool.{subject}")).await
    }
}

#[cfg(all(feature = "async", feature = "serde-support"))]
impl<T: EventBus + ?Sized> EventBusExt for T {}

// -------------------------------------------------------------------------------------------------
// Prelude for ergonomic downstream imports
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "serde-support")]
pub mod prelude {
    pub use super::{BusEventHeader, AgentEvent, ToolEvent, EventPayload};
    #[cfg(feature = "async")]
    pub use super::{EventBus, EventBusExt};
}