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

// ---------------------------------------------------------------------------
// Modern re-exports – legacy modules removed to avoid duplication.
// ---------------------------------------------------------------------------

// Core primitives & hashing
pub use toka_events_core::{EventHeader, EventPayload, EventId, IntentId, CausalDigest, causal_hash};

// Persistent storage-backed event bus
pub use toka_bus_persist::{VaultBus, PersistentEventBus};

// NOTE: The original internal `core`, `hash`, and `bus` modules were removed
// in favour of the canonical implementations in `toka-events-core` and
// `toka-bus-persist`. Downstream code can continue to depend on
// `toka_ledger_core::*` without any changes. 