#![forbid(unsafe_code)]

//! `toka-capability-jwt-hs256` – concrete HS256 JWT capability token format.
//!
//! This crate wires the *pure* definitions from [`toka-capability-core`]
//! onto the ubiquitous HS256 JSON Web Token algorithm, reusing the
//! `jsonwebtoken` crate for the heavy lifting.
//!
//! Most users will only interact with [`JwtHs256Token`] and
//! [`JwtHs256Validator`] or the glob import `toka_capability_jwt_hs256::prelude::*`.

pub mod token;
pub mod validator;

pub mod prelude {
    pub use super::token::JwtHs256Token;
    pub use toka_capability_core::prelude::Claims;
    pub use super::validator::{JwtHs256Validator};
    pub use toka_capability_core::prelude::{TokenValidator, CapabilityToken};
}

pub use prelude::*;