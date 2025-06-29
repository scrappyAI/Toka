use anyhow::Result;
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use toka_capability_core::prelude::{Claims, TokenValidator};

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
        Self {
            secret: secret.into(),
            validation,
        }
    }
}

#[async_trait]
impl TokenValidator for JwtHs256Validator {
    async fn validate(&self, raw: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            raw,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &self.validation,
        )?;
        Ok(data.claims)
    }
}