//! Token validation traits & implementations
//!
//! This module intentionally mirrors the behaviour of `jsonwebtoken` but
//! wraps it in a trait so that *callers stay decoupled from crypto choices*.
//! Future versions may offer Paseto, Biscuit or custom MAC schemes behind the
//! same interface.

use anyhow::Result;
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::token::Claims;

/// Generic validation behaviour for capability tokens.
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Verify `raw` token authenticity and semantic correctness.
    ///
    /// On success returns the decoded [`Claims`].
    async fn validate(&self, raw: &str) -> Result<Claims>;
}

/// Symmetric-key JWT validator (HS256).
#[derive(Clone, Debug)]
pub struct JwtValidator {
    secret: String,
    validation: Validation,
}

impl JwtValidator {
    /// Build a new validator using `secret`.  Accepts only HS256 for now.
    pub fn new(secret: impl Into<String>) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        Self {
            secret: secret.into(),
            validation,
        }
    }
}

#[async_trait]
impl TokenValidator for JwtValidator {
    async fn validate(&self, raw: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            raw,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &self.validation,
        )?;
        Ok(data.claims)
    }
}