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
use anyhow::Result;
use async_trait::async_trait;

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