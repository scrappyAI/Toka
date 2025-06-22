//! # Toka Ledger – Agent Extensions
//!
//! This crate builds on [`toka_ledger_core`] by adding *semantic* functionality
//! such as **online intent clustering**.  It intentionally keeps the storage
//! layer identical so events written by agents can be consumed by any other
//! projection (finance, analytics, etc.).
//!
//! ## Key additions
//! * [`IntentStore`] – cosine-similarity clustering of embeddings.
//! * [`AgentBus`]   – drop-in replacement for `VaultBus` that automatically
//!   assigns each event to an intent.
//!
//! Down-stream code that doesn't care about intent semantics can continue to
//! use the core `VaultBus` directly.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod intent;
pub mod bus;

// Re-export common primitives from `toka_ledger_core` so users only need this
// single crate in agent contexts.

pub use toka_ledger_core::{EventHeader, EventPayload, EventId, IntentId, CausalDigest, causal_hash};
pub use bus::AgentBus; 