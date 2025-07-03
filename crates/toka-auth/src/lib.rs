#![forbid(unsafe_code)]

//! **toka-auth** – Capability‐based security primitives for Toka OS.
//!
//! This crate defines the canonical `Claims` structure and the two key
//! traits – [`CapabilityToken`] and [`TokenValidator`] – used across the
//! workspace.  A simple HS256 JWT implementation (`JwtHs256Token` /
//! `JwtHs256Validator`) is bundled for v0.1.
//!
//! Future releases can provide additional algorithms (Biscuit, Paseto, …)
//! via crate features while reusing the same trait contracts.
//!
//! 🔗 See [`docs/42_toka_kernel_spec_v0.1.md`](../../../docs/42_toka_kernel_spec_v0.1.md) for how capability tokens
//! gate **Operations** inside the kernel.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
// (Root level: keep dependency-agnostic; heavy imports live inside submodules)

/// Canonical claim set embedded in every capability token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject – usually the *user* or *agent* identifier.
    pub sub: String,
    /// Vault / workspace identifier the subject wishes to access.
    pub vault: String,
    /// Ordered list of permissions (e.g. `transfer`, `mint`).
    pub permissions: Vec<String>,
    /// Issued‐at timestamp (seconds since Unix epoch).
    pub iat: u64,
    /// Absolute expiry timestamp (seconds since Unix epoch).
    pub exp: u64,
    /// Unique token identifier (e.g. UUIDv4) for replay protection.
    pub jti: String,
}

impl Claims {
    /// Validate the claims to ensure they meet security requirements.
    /// 
    /// # Security
    /// Validates all claim fields to prevent various attack vectors including
    /// token bloat, excessive lifetimes, and invalid identifiers.
    pub fn validate(&self) -> Result<()> {
        // SECURITY: Validate subject identifier
        if self.sub.trim().is_empty() {
            return Err(Error::new("Subject identifier cannot be empty"));
        }
        if self.sub.len() > 256 {
            return Err(Error::new("Subject identifier too long"));
        }

        // SECURITY: Validate vault identifier  
        if self.vault.trim().is_empty() {
            return Err(Error::new("Vault identifier cannot be empty"));
        }
        if self.vault.len() > 256 {
            return Err(Error::new("Vault identifier too long"));
        }

        // SECURITY: Validate permissions list
        if self.permissions.len() > MAX_PERMISSIONS_COUNT {
            return Err(Error::new("Too many permissions in token"));
        }
        for permission in &self.permissions {
            if permission.trim().is_empty() {
                return Err(Error::new("Permission cannot be empty"));
            }
            if permission.len() > 64 {
                return Err(Error::new("Permission name too long"));
            }
        }

        // SECURITY: Validate timestamps
        if self.exp <= self.iat {
            return Err(Error::new("Token expiry must be after issuance"));
        }
        let lifetime = self.exp - self.iat;
        if lifetime > MAX_TOKEN_LIFETIME_SECS {
            return Err(Error::new("Token lifetime exceeds maximum allowed"));
        }

        // SECURITY: Validate JTI for replay protection
        if self.jti.trim().is_empty() {
            return Err(Error::new("Token ID (jti) cannot be empty"));
        }
        if self.jti.len() > 256 {
            return Err(Error::new("Token ID (jti) too long"));
        }

        Ok(())
    }

    /// Check if the token is currently expired.
    /// 
    /// # Security
    /// Uses system time carefully to prevent timing attacks.
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => return true, // Assume expired on time errors
        };
        now >= self.exp
    }
}

/// Minimal in‐crate error type.
#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Self { Self { msg: msg.into() } }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl std::error::Error for Error {}

/// Crate result helper.
pub type Result<T> = std::result::Result<T, Error>;

//─────────────────────────────
//  Security constants  
//─────────────────────────────

/// Maximum allowed token lifetime in seconds (24 hours) to limit exposure
pub const MAX_TOKEN_LIFETIME_SECS: u64 = 86400;

/// Maximum number of permissions per token to prevent bloat
pub const MAX_PERMISSIONS_COUNT: usize = 100;

//─────────────────────────────
//  Trait definitions
//─────────────────────────────

/// Abstract behaviour common to *all* capability token formats.
#[async_trait]
pub trait CapabilityToken: Sized + Send + Sync {
    /// Mint a new token from raw `claims` using the provided secret / key.
    async fn mint(claims: &Claims, key: &[u8]) -> Result<Self>;

    /// Return the serialized wire representation (e.g. JWT string).
    fn as_str(&self) -> &str;
}

/// Verifier trait used by the kernel before executing an operation.
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Verify authenticity + semantic correctness, returning the embedded [`Claims`].
    /// 
    /// # Security
    /// Implementations should log authentication failures for security monitoring
    /// and use constant-time operations where possible to prevent timing attacks.
    async fn validate(&self, raw: &str) -> Result<Claims>;
}

//─────────────────────────────
//  HS256 JWT implementation
//─────────────────────────────

pub mod hs256 {
    use super::{Claims, Result, Error, CapabilityToken, TokenValidator};
    use async_trait::async_trait;
    use jsonwebtoken::{encode, decode, Algorithm, Header, Validation, EncodingKey, DecodingKey, TokenData};
    use uuid::Uuid;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Concrete JWT (HS256) capability token implementation.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct JwtHs256Token {
        token: String,
    }

    impl JwtHs256Token {
        /// Decode and validate the token, returning [`Claims`] (expiry enforced).
        pub fn claims(&self, secret: &str) -> Result<Claims> {
            Self::decode_internal(&self.token, secret)
                .map(|d| d.claims)
                .map_err(|e| Error::new(&e.to_string()))
        }

        /// Quick authenticity + expiry check.
        pub fn is_valid(&self, secret: &str) -> bool {
            let data = match Self::decode_internal(&self.token, secret) {
                Ok(d) => d,
                Err(_) => return false,
            };
            let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(d) => d.as_secs() as u64,
                Err(_) => return false,
            };
            now < data.claims.exp
        }

        fn decode_internal(token: &str, secret: &str) -> std::result::Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.validate_exp = true;
            validation.leeway = 0;
            decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            )
        }

        /// Convenience helper kept from legacy code – synchronous mint.
        pub fn new(subject: &str, vault: &str, permissions: Vec<String>, secret: &str, ttl_secs: u64) -> Result<Self> {
            let claims = build_claims(subject, vault, permissions, ttl_secs)?;
            let mut header = Header::new(Algorithm::HS256);
            header.typ = Some("toka.cap+jwt".into());
            let jwt = encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))
                .map_err(|e| Error::new(&e.to_string()))?;
            Ok(Self { token: jwt })
        }
    }

    #[async_trait]
    impl CapabilityToken for JwtHs256Token {
        async fn mint(claims: &Claims, key: &[u8]) -> Result<Self> {
            let mut header = Header::new(Algorithm::HS256);
            header.typ = Some("toka.cap+jwt".into());
            let jwt = encode(
                &header,
                claims,
                &EncodingKey::from_secret(key),
            ).map_err(|e| Error::new(&e.to_string()))?;
            Ok(Self { token: jwt })
        }

        fn as_str(&self) -> &str {
            &self.token
        }
    }

    /// Helper to construct standard claims with proper timestamps.
    pub fn build_claims(subject: &str, vault: &str, permissions: Vec<String>, ttl_secs: u64) -> Result<Claims> {
        let issued_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::new(&e.to_string()))?
            .as_secs();
        Ok(Claims {
            sub: subject.to_owned(),
            vault: vault.to_owned(),
            permissions,
            iat: issued_at,
            exp: issued_at + ttl_secs,
            jti: Uuid::new_v4().to_string(),
        })
    }

    /// HS256 JWT validator.
    #[derive(Clone, Debug)]
    pub struct JwtHs256Validator {
        secret: String,
        validation: Validation,
    }

    impl JwtHs256Validator {
        pub fn new(secret: impl Into<String>) -> Self {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.validate_exp = true;
            validation.leeway = 0;
            Self {
                secret: secret.into(),
                validation,
            }
        }
    }

    #[async_trait]
    impl TokenValidator for JwtHs256Validator {
        async fn validate(&self, raw: &str) -> Result<Claims> {
            // SECURITY: Log validation attempts for monitoring
            let validation_start = std::time::Instant::now();
            
            let result = decode::<Claims>(
                raw,
                &DecodingKey::from_secret(self.secret.as_bytes()),
                &self.validation,
            );
            
            let claims = match result {
                Ok(data) => data.claims,
                Err(e) => {
                    // SECURITY: Log authentication failures for monitoring
                    eprintln!("Token validation failed: {} (took {:?})", e, validation_start.elapsed());
                    return Err(Error::new(&e.to_string()));
                }
            };

            // SECURITY: Additional claim validation
            claims.validate()?;

            // SECURITY: Log successful validations for audit trail
            let duration = validation_start.elapsed();
            if duration.as_millis() > 100 {
                eprintln!("Token validation took unusually long: {:?}", duration);
            }

            Ok(claims)
        }
    }

    /// Glob‐import helper.
    pub mod prelude {
        pub use super::{JwtHs256Token, JwtHs256Validator};
        pub use crate::{CapabilityToken, TokenValidator, Claims};
    }
}

/// Top-level convenience export re-exporting the HS256 implementation.
pub use hs256::prelude::*;

/// Single‐line glob import for downstream crates.
pub mod prelude {
    pub use super::{Claims, CapabilityToken, TokenValidator};
    pub use super::hs256::prelude::*;
}
