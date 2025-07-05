#![forbid(unsafe_code)]

//! `toka-capability-jwt-hs256` – concrete HS256 JWT capability token format.
//!
//! This crate wires the *pure* definitions from [`toka-capability-core`]
//! onto the ubiquitous HS256 JSON Web Token algorithm, reusing the
//! `jsonwebtoken` crate for the heavy lifting.
//!
//! Most users will only interact with [`JwtHs256Token`] and
//! [`JwtHs256Validator`] or the glob import `toka_capability_jwt_hs256::prelude::*`.
//!
//! # Security Enhancements
//!
//! This crate integrates with the Toka security framework extensions:
//! - **JWT Key Rotation** via `toka-key-rotation` for automatic key management
//! - **Rate Limiting** via `toka-rate-limiter` for authentication throttling
//! - **Capability Delegation** via `toka-capability-delegation` for hierarchical permissions
//!
//! These components can be used together to provide comprehensive security.

pub mod token;
pub mod validator;

pub mod prelude {
    pub use super::token::JwtHs256Token;
    pub use toka_capability_core::prelude::Claims;
    pub use super::validator::{JwtHs256Validator};
    pub use toka_capability_core::prelude::{TokenValidator, CapabilityToken};
    
    // Security enhancements are available as separate crates:
    // - toka-key-rotation: Automatic JWT key rotation
    // - toka-rate-limiter: Rate limiting middleware
    // - toka-capability-delegation: Hierarchical permission delegation
}

pub use prelude::*;