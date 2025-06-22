//! # Toka Ledger Core – minimal event store
//!
//! This crate provides the *foundational* pieces of the Toka event-sourcing
//! stack: hashing, persistence and broadcast streaming.  It deliberately leaves
//! out higher-level semantics such as intent clustering so that domain crates
//! (e.g. `toka-ledger-agents`, `toka-ledger-finance`) can build specialised
//! behaviour without bloating the core.
//!
//! *No breaking changes to the wire format were made:* the `EventHeader` struct
//! is identical to the original monolithic crate so existing databases remain
//! readable.
//!
//! ## Modules
//! * `core`  – fundamental types (`EventHeader`, `EventPayload`, …)
//! * `hash`  – Blake3 causal hashing utilities
//! * `bus`   – `VaultBus` storage + broadcast API (no clustering)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core;
pub mod hash;
pub mod bus;

// ---------------------------------------------------------------------------
// Re-exports for ergonomic downstream use
// ---------------------------------------------------------------------------

pub use core::{EventHeader, EventPayload, EventId, IntentId, CausalDigest};
pub use bus::VaultBus;
pub use hash::causal_hash; 