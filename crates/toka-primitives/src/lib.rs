//! # Toka Primitives
//!
//! Fundamental, opinion-free building blocks used across the Toka platform.
//! These primitives are intentionally free of vertical opinions (creator-centric,
//! FinOps-centric, etc.) so that higher-level crates can compose them without
//! unwanted baggage.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Module declarations (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "ids")]
pub mod ids;
#[cfg(feature = "currency")]
pub mod currency;

// ---------------------------------------------------------------------------
// Re-exports for ergonomic downstream use
// ---------------------------------------------------------------------------

#[cfg(feature = "ids")]
pub use ids::{AgentID, ModelID, ProductID, ResourceID, TransactionID, UserID, VaultID};

#[cfg(feature = "currency")]
pub use currency::MicroUSD; 