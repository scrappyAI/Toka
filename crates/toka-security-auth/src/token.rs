use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub subject: String,
    pub vault_id: String,
    pub permissions: Vec<String>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub signature: String,
}

impl CapabilityToken {
    pub fn new(subject: &str, vault_id: &str, permissions: Vec<String>, secret: &str, ttl_secs: u64) -> Self {
        let issued_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let expires_at = issued_at + ttl_secs;
        let mut token = CapabilityToken {
            subject: subject.to_string(),
            vault_id: vault_id.to_string(),
            permissions,
            issued_at,
            expires_at,
            signature: String::new(),
        };
        token.signature = token.compute_signature(secret);
        token
    }

    pub fn compute_signature(&self, secret: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.subject.as_bytes());
        hasher.update(self.vault_id.as_bytes());
        hasher.update(self.permissions.join(",").as_bytes());
        hasher.update(self.issued_at.to_le_bytes());
        hasher.update(self.expires_at.to_le_bytes());
        hasher.update(secret.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn is_valid(&self, secret: &str) -> bool {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() < self.expires_at
            && self.signature == self.compute_signature(secret)
    }
} 