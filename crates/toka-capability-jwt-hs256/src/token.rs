use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use jsonwebtoken::{encode, decode, Algorithm, Header, Validation, EncodingKey, DecodingKey, TokenData};
use uuid::Uuid;
use toka_capability_core::prelude::{Claims, CapabilityToken};
use async_trait::async_trait;

/// Concrete JWT (HS256) capability token implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHs256Token {
    token: String,
}

impl JwtHs256Token {
    /// Decode and validate, returning Claims (strict expiry).
    pub fn claims(&self, secret: &str) -> Result<Claims> {
        Ok(Self::decode_internal(&self.token, secret)?.claims)
    }

    /// Authenticity + expiry quick check.
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

    /// Legacy helper kept for compatibility – *synchronous* shorthand that
    /// mints a token with `subject`, `vault`, `permissions` and TTL.
    pub fn new(subject: &str, vault_id: &str, permissions: Vec<String>, secret: &str, ttl_secs: u64) -> Result<Self> {
        let claims = build_claims(subject, vault_id, permissions, ttl_secs)?;
        // direct encode (internal helper) – reuse mint but synchronously via block_in_place? simpler duplicate logic.
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("toka.cap+jwt".into());
        let jwt = encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))
            .map_err(|e| anyhow!(e))?;
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
        ).map_err(|e| anyhow!(e))?;
        Ok(Self { token: jwt })
    }

    fn as_str(&self) -> &str {
        &self.token
    }
}

/// Helper to build standard claims with right timestamps.
pub fn build_claims(subject: &str, vault: &str, permissions: Vec<String>, ttl_secs: u64) -> Result<Claims> {
    let issued_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(Claims {
        sub: subject.to_owned(),
        vault: vault.to_owned(),
        permissions,
        iat: issued_at,
        exp: issued_at + ttl_secs,
        jti: Uuid::new_v4().to_string(),
    })
}