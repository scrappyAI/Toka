#![forbid(unsafe_code)]
#![deprecated(note = "toka-capability has been split into toka-capability-core + toka-capability-jwt-hs256. Depend on those crates directly.")]

//! DEPRECATED – use `toka-capability-core` + `toka-capability-jwt-hs256` instead.

pub use toka_capability_core as core;
pub use toka_capability_jwt_hs256::*;

// Backwards compatibility: keep original names via re-export.
pub use toka_capability_core::prelude::Claims;
pub use toka_capability_jwt_hs256::token::JwtHs256Token as CapabilityToken;
pub use toka_capability_jwt_hs256::validator::JwtHs256Validator as JwtValidator;
// Keep selective re-exports to avoid naming clashes.
pub use toka_capability_core::prelude::{TokenValidator};

/// Prelude retaining the legacy (v0.1) surface while delegating to new crates.
pub mod prelude {
    pub use super::{Claims, CapabilityToken, JwtValidator};
    pub use toka_capability_core::prelude::TokenValidator;
}