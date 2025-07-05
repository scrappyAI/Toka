#![forbid(unsafe_code)]

//! **toka-key-rotation** – Automatic JWT key rotation for Toka capability tokens
//!
//! This crate provides secure automatic rotation of JWT signing keys to enhance
//! the security posture of the Toka platform. Key features include:
//!
//! * **Automatic Rotation**: Configurable rotation intervals with overlap periods
//! * **Multiple Active Keys**: Support for gradual key rollover during rotation
//! * **Key Versioning**: Track key generations for audit and debugging
//! * **Secure Generation**: Cryptographically secure key generation
//! * **Event Notifications**: Audit trail for key rotation events
//!
//! The design follows security best practices for key rotation:
//! - Keys are rotated at configurable intervals (default: 24 hours)
//! - Old keys remain valid during overlap period (default: 1 hour)
//! - All key operations are logged for audit trails
//! - Keys are generated using cryptographically secure random sources

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod manager;
pub mod events;
pub mod validator;

/// Represents a versioned JWT signing key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersion {
    /// Unique identifier for this key version
    pub id: Uuid,
    /// The actual key material (base64 encoded)
    pub key_material: String,
    /// When this key was created
    pub created_at: DateTime<Utc>,
    /// When this key expires and should no longer be used for signing
    pub expires_at: DateTime<Utc>,
    /// When this key should no longer be accepted for validation
    pub valid_until: DateTime<Utc>,
    /// Whether this key is currently active for signing new tokens
    pub is_active: bool,
    /// Generation number for this key (incrementing counter)
    pub generation: u64,
}

impl KeyVersion {
    /// Create a new key version with secure random key material
    pub fn new(generation: u64, rotation_interval: Duration, overlap_period: Duration) -> Result<Self> {
        let now = Utc::now();
        let key_material = generate_secure_key()?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            key_material,
            created_at: now,
            expires_at: now + rotation_interval,
            valid_until: now + rotation_interval + overlap_period,
            is_active: true,
            generation,
        })
    }

    /// Check if this key is still valid for token validation
    pub fn is_valid_for_validation(&self) -> bool {
        let now = Utc::now();
        now < self.valid_until
    }

    /// Check if this key should be used for signing new tokens
    pub fn is_active_for_signing(&self) -> bool {
        let now = Utc::now();
        self.is_active && now < self.expires_at
    }

    /// Get the key material as bytes for JWT operations
    pub fn key_bytes(&self) -> Result<Vec<u8>> {
        hex::decode(&self.key_material)
            .map_err(|e| anyhow::anyhow!("Failed to decode key material: {}", e))
    }
}

/// Configuration for key rotation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// How often to rotate keys (default: 24 hours)
    pub rotation_interval: Duration,
    /// How long old keys remain valid after rotation (default: 1 hour)
    pub overlap_period: Duration,
    /// Maximum number of key versions to keep (default: 10)
    pub max_key_versions: usize,
    /// Whether to enable automatic rotation (default: true)
    pub auto_rotation_enabled: bool,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            rotation_interval: Duration::hours(24),
            overlap_period: Duration::hours(1),
            max_key_versions: 10,
            auto_rotation_enabled: true,
        }
    }
}

/// Trait for storing and retrieving key versions
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Store a new key version
    async fn store_key(&self, key: &KeyVersion) -> Result<()>;
    
    /// Retrieve a specific key version by ID
    async fn get_key(&self, key_id: &Uuid) -> Result<Option<KeyVersion>>;
    
    /// Get the current active key for signing
    async fn get_active_key(&self) -> Result<Option<KeyVersion>>;
    
    /// Get all valid keys for token validation
    async fn get_valid_keys(&self) -> Result<Vec<KeyVersion>>;
    
    /// Mark a key as inactive
    async fn deactivate_key(&self, key_id: &Uuid) -> Result<()>;
    
    /// Remove expired keys from storage
    async fn cleanup_expired_keys(&self) -> Result<usize>;
    
    /// Get all key versions (for audit/debugging)
    async fn get_all_keys(&self) -> Result<Vec<KeyVersion>>;
}

/// Trait for receiving key rotation events
#[async_trait]
pub trait RotationEventHandler: Send + Sync {
    /// Called when a new key is generated
    async fn on_key_generated(&self, key: &KeyVersion) -> Result<()>;
    
    /// Called when a key is rotated (new key becomes active)
    async fn on_key_rotated(&self, old_key: &KeyVersion, new_key: &KeyVersion) -> Result<()>;
    
    /// Called when keys are cleaned up
    async fn on_keys_cleaned_up(&self, count: usize) -> Result<()>;
    
    /// Called when rotation fails
    async fn on_rotation_failed(&self, error: &anyhow::Error) -> Result<()>;
}

/// Generate a cryptographically secure key for JWT signing
fn generate_secure_key() -> Result<String> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 32]; // 256-bit key
    rng.fill_bytes(&mut key_bytes);
    Ok(hex::encode(key_bytes))
}

/// Convenience re-exports for common usage
pub mod prelude {
    pub use super::{
        KeyVersion, RotationConfig, KeyStore, RotationEventHandler,
        manager::KeyRotationManager,
        validator::RotatingJwtValidator,
        events::AuditEventHandler,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_version_creation() {
        let config = RotationConfig::default();
        let key = KeyVersion::new(1, config.rotation_interval, config.overlap_period).unwrap();
        
        assert_eq!(key.generation, 1);
        assert!(key.is_active);
        assert!(key.is_active_for_signing());
        assert!(key.is_valid_for_validation());
        assert_eq!(key.key_material.len(), 64); // 32 bytes hex encoded
    }

    #[test]
    fn test_key_expiry() {
        let past_time = Utc::now() - Duration::hours(25);
        let key = KeyVersion {
            id: Uuid::new_v4(),
            key_material: generate_secure_key().unwrap(),
            created_at: past_time,
            expires_at: past_time + Duration::hours(24),
            valid_until: past_time + Duration::hours(25),
            is_active: true,
            generation: 1,
        };
        
        assert!(!key.is_active_for_signing());
        assert!(!key.is_valid_for_validation());
    }

    #[test]
    fn test_secure_key_generation() {
        let key1 = generate_secure_key().unwrap();
        let key2 = generate_secure_key().unwrap();
        
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 64); // 32 bytes hex encoded
        assert_eq!(key2.len(), 64);
    }
} 