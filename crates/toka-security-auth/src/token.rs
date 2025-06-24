use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use jsonwebtoken::{encode, decode, Algorithm, Header, Validation, EncodingKey, DecodingKey, TokenData};

/// JWT claim-set used by capability tokens.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Claims {
    /// Subject – usually the *user* or *agent* identifier.
    pub sub: String,
    /// Vault the subject wishes to access.
    pub vault: String,
    /// Ordered list of permissions (e.g. `read`, `write`).
    pub permissions: Vec<String>,
    /// Issued-at (seconds since Unix epoch).
    pub iat: usize,
    /// Expiration timestamp (seconds since Unix epoch).
    pub exp: usize,
}

/// Thin wrapper around a JWT-encoded capability token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    token: String,
}

impl CapabilityToken {
    /// Generate a new signed JWT containing the provided claims.
    pub fn new(
        subject: &str,
        vault_id: &str,
        permissions: Vec<String>,
        secret: &str,
        ttl_secs: u64,
    ) -> Result<Self> {
        let issued_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as usize;
        let expires_at = issued_at + ttl_secs as usize;

        let claims = Claims {
            sub: subject.to_owned(),
            vault: vault_id.to_owned(),
            permissions,
            iat: issued_at,
            exp: expires_at,
        };

        // Default header = HS256
        let header = Header::new(Algorithm::HS256);
        let jwt = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| anyhow!(e))?;

        Ok(Self { token: jwt })
    }

    /// Borrow the inner JWT string.
    pub fn as_str(&self) -> &str {
        &self.token
    }

    /// Decode and validate the token, returning the underlying [`Claims`].
    pub fn claims(&self, secret: &str) -> Result<Claims> {
        Ok(Self::decode_internal(&self.token, secret)?.claims)
    }

    /// Fast authenticity + expiry check.
    ///
    /// The underlying `jsonwebtoken` validation treats a token whose `exp`
    /// timestamp is **equal** to the current Unix time as *still valid*.
    /// For our use-case we want a stricter policy – a token is considered
    /// invalid the *moment* we reach its expiry second.  We therefore perform
    /// an extra comparison on the decoded claims after signature/auth checks
    /// succeed.
    pub fn is_valid(&self, secret: &str) -> bool {
        let data = match Self::decode_internal(&self.token, secret) {
            Ok(d) => d,
            Err(_) => return false,
        };

        // Strict expiration check (no leeway, strictly before `exp`).
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs() as usize,
            Err(_) => return false, // system clock before epoch – treat as invalid
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
}
