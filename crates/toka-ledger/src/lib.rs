//! # Toka Vaults – The Event-Driven OS Layer
//!
//! This crate implements the "Vault OS" concept for Toka: a causal, content-addressed
//! event store with semantic intent clustering. Agents operate by committing events
//! to the vault and subscribing to relevant event streams.
//!
//! ## Architecture
//!
//! The vault system consists of several key components:
//!
//! - **Core Types** (`core` module): Event headers, payloads, and fundamental primitives
//! - **Causal Hashing** (`hash` module): Content-addressed storage with causal dependencies
//! - **Intent Clustering** (`intent` module): Online semantic clustering of event embeddings
//! - **Event Bus** (`bus` module): Persistent storage and live event streaming
//!
//! ## Usage
//!
//! ```rust,no_run
//! use toka_ledger::{VaultBus, EventPayload};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Mint { amount: u64 }
//!
//! # async fn example() -> anyhow::Result<()> {
//! let vault = VaultBus::open("./vault-data")?;
//! let embedding = ndarray::arr1(&[1.0, 0.5, -0.2]); // Simple example embedding
//!
//! let header = vault.commit(
//!     &Mint { amount: 100 },
//!     &[], // No parents
//!     "ledger.mint",
//!     embedding
//! ).await?;
//!
//! println!("Committed event: {}", header.id);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core;
pub mod hash;
pub mod intent;
pub mod bus;

// Re-export the main types for convenience
pub use core::{EventHeader, EventPayload, EventId, IntentId, CausalDigest};
pub use bus::VaultBus;
pub use intent::IntentStore;
pub use hash::causal_hash; 