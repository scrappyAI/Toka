//! # Toka Products
//!
//! Defines user-facing offerings, such as credit packages, that can be purchased.
//! These are the "SKUs" of the platform.

pub mod credit_packages;

// Re-export key types for convenience
pub use self::credit_packages::{CreditPackage, CreditPackageTier, CreditPackageView}; 