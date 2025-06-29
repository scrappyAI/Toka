//! **toka-capability-core**
//!
//! Core, `no_std`‐friendly building blocks for capability‐based security in
//! the Toka platform.  The crate owns the canonical [`Claims`] data structure
//! and the two foundational traits – [`CapabilityToken`] and
//! [`TokenValidator`].  It is intentionally *crypto-agnostic* and free of I/O
//! so that alternative formats (JWT, Biscuit, Paseto, …) can be layered on
//! top without incurring additional dependencies.
//!
//! The wire-format details live in sibling *implementation* crates such as
//! `toka-capability-jwt-hs256`.  For architectural background refer to
//! `docs/40_capability_tokens_spec_v0.1.md` and
//! `docs/41_capability_tokens_architecture.md`.
//!
//! # Crate Features
//! * `std` *(default)* – Enables the Rust standard library and lives in
//!   regular server/workstation builds.
//! * *no feature* – Compiles in pure `no_std` + `alloc` mode for embedded or
//!   constrained WASM targets.
//!
//! This crate forbids `unsafe_code` and aims to stay dependency-light.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

// When in no_std mode we need alloc for heap-backed types (String, Vec).
#[cfg_attr(not(feature = "std"), macro_use)]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use alloc::boxed::Box;

/// Canonical claim-set used by capability tokens across the Toka platform.
///
/// This struct is intentionally *dumb*: it performs *no* validation, has
/// *no* knowledge about cryptography or clocks and is fully `no_std`
/// compatible (behind the `alloc` feature).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject – usually the *user* or *agent* identifier.
    pub sub: String,
    /// Vault / workspace identifier the subject wishes to access.
    pub vault: String,
    /// Ordered list of permissions (e.g. `read`, `write`).
    pub permissions: Vec<String>,
    /// Issued-at timestamp (seconds since Unix epoch).
    pub iat: u64,
    /// Absolute expiry timestamp (seconds since Unix epoch).
    pub exp: u64,
    /// Unique token identifier (e.g. UUIDv4) for audit / replay-protection.
    pub jti: String,
}

/// Result type used throughout **toka-capability-core**.
pub type Result<T> = core::result::Result<T, Error>;

/// Minimal error type keeping the crate `no_std` + `alloc` friendly.
#[derive(Debug)]
pub struct Error {
    msg: alloc::string::String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Self { msg: msg.into() }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.msg.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Behaviour common to **all** concrete capability token formats.
///
/// A token type implementing this trait is responsible for *encoding* the
/// provided [`Claims`] into its wire format (e.g. a JWT string) and for
/// *decoding* it back into the canonical struct.  The trait is deliberately
/// small so that alternative algorithms (Biscuit, Paseto, …) can plug in
/// without imposing additional dependencies.
#[async_trait]
pub trait CapabilityToken: Sized + Send + Sync {
    /// Create a new token instance from raw claims and a secret / key.
    async fn mint(claims: &Claims, key: &[u8]) -> Result<Self>;

    /// Return the serialized wire-format representation (e.g. the raw JWT).
    fn as_str(&self) -> &str;
}

/// Verification behaviour shared across the platform.
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Verify `raw` token authenticity and semantic correctness.
    async fn validate(&self, raw: &str) -> Result<Claims>;
}

/// Convenience module collecting the most commonly used exports so that
/// downstream crates only need a single `use` line.
pub mod prelude {
    pub use super::{Claims, CapabilityToken, TokenValidator};
}