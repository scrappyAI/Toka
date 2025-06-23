//! # Toka Ledger Agents – semantic extensions re-export
//!
//! This crate is now a thin wrapper around `toka-bus-persist` with the
//! `intent-cluster` feature enabled. All agent-aware event buses and intent
//! clustering utilities live in that canonical crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Re-export primitives and agent-aware bus from the canonical crates.

pub use toka_events_core::{EventHeader, EventPayload, EventId, IntentId, CausalDigest, causal_hash};
pub use toka_bus_persist::{AgentBus, OnlineClusterStrategy}; 