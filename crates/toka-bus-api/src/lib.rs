//! `toka-bus-api` — minimal, **no_std-friendly** trait contracts and data types
//! for the Toka **in-process event bus**.
//!
//! This crate purposefully ships **zero** heavy dependencies; consumers can
//! enable the following *opt-in* features:
//!
//! | Feature | Enables | Notes |
//! |---------|---------|-------|
//! | `serde-support` | `serde::{Serialize, Deserialize}` impls on all public   
//!                    types | Keeps default build lean if you only work with    
//!                    opaque headers. |
//! | `async` | • `async_trait` for the runtime traits  
//!           | • `anyhow` for ergonomic `Result` types | Activates the async   
//!           APIs; implies you are in a `tokio` (or compatible) env. |
//! | `tokio` | `tokio::sync::broadcast::Receiver` in the `EventBus` API | Pulled
//!            in automatically by `async` but can be enabled standalone. |
//!
//! ## Quick Example
//! ```rust,ignore
//! use toka_bus_api::{prelude::*, AgentEvent};
//! use anyhow::Result;
//!
//! // Dummy implementation which just logs.
//! struct LoggerBus;
//! #[async_trait::async_trait]
//! impl EventBus for LoggerBus {
//!     async fn publish<P: EventPayload + ?Sized>(&self,
//!         _payload: &P,
//!         kind: &str) -> Result<BusEventHeader> {
//!         println!("published {kind}");
//!         Ok(BusEventHeader::new(kind))
//!     }
//!     fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BusEventHeader> {
//!         let (tx, rx) = tokio::sync::broadcast::channel(8);
//!         let _ = tx; // tx would be stored and used by publish
//!         rx
//!     }
//! }
//! ```
//!
//! See `toka-bus` for the default in-memory implementation.

#![no_std]
#![forbid(unsafe_code)]
#![allow(missing_docs)]

extern crate alloc;
use alloc::boxed::Box;

#[cfg(feature = "serde-support")] use serde::{Serialize, Deserialize};
#[cfg(feature = "async")] use async_trait::async_trait;
#[cfg(feature = "async")] use anyhow::Result;
#[cfg(feature = "serde-support")] use chrono::{DateTime, Utc};
#[cfg(feature = "serde-support")] use uuid::Uuid;

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

impl BusEventHeader {
    /// Convenience constructor when payload isn't required.
    #[cfg(all(feature = "serde-support", feature = "async"))]
    pub fn new(kind: &str) -> Self {
        Self { id: Uuid::new_v4(), kind: alloc::string::String::from(kind), timestamp: Utc::now() }
    }
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
    //! Convenient glob-import for the most common items:
    //! `use toka_bus_api::prelude::*;`
    pub use super::{BusEventHeader, AgentEvent, ToolEvent, EventPayload};
    #[cfg(feature = "async")]
    pub use super::{EventBus, EventBusExt};
}