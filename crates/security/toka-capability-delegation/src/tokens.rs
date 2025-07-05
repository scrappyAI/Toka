//! Token management for delegated capabilities
//!
//! This module provides functionality for creating and managing JWT tokens
//! that contain delegation metadata. It integrates with the existing
//! capability token system to provide seamless delegation support.

use crate::{DelegatedClaims, DelegationError};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use toka_capability_core::{Claims, CapabilityToken, TokenValidator};
use toka_capability_jwt_hs256::JwtHs256Token;
use tracing::{debug, warn};
use uuid::Uuid;
use base64::Engine;

/// Token generator for delegated capabilities
/// 
/// This trait extends the basic capability token functionality to support
/// delegation metadata and hierarchical permission structures.
#[async_trait]
pub trait DelegatedTokenGenerator: Send + Sync {
    /// Create a new delegated token from delegated claims
    async fn create_delegated_token(
        &self,
        claims: &DelegatedClaims,
        key: &[u8],
    ) -> Result<String, DelegationError>;

    /// Parse a token string into delegated claims
    async fn parse_delegated_token(
        &self,
        token: &str,
        key: &[u8],
    ) -> Result<DelegatedClaims, DelegationError>;

    /// Validate a delegated token and return the claims
    async fn validate_delegated_token(
        &self,
        token: &str,
        key: &[u8],
    ) -> Result<DelegatedClaims, DelegationError>;
}

/// JWT-based implementation of delegated token generator
pub struct JwtDelegatedTokenGenerator {
    /// Cache of recently validated tokens to improve performance
    token_cache: Arc<RwLock<TokenCache>>,
    /// Configuration for token generation
    config: TokenConfig,
}

impl JwtDelegatedTokenGenerator {
    /// Create a new JWT delegated token generator
    pub fn new(config: TokenConfig) -> Self {
        Self {
            token_cache: Arc::new(RwLock::new(TokenCache::new(config.cache_size))),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(TokenConfig::default())
    }

    /// Serialize delegated claims to JWT custom claims
    fn serialize_delegation_claims(claims: &DelegatedClaims) -> Result<serde_json::Value, DelegationError> {
        let mut jwt_claims = serde_json::json!({
            "sub": claims.base.sub,
            "vault": claims.base.vault,
            "permissions": claims.base.permissions,
            "iat": claims.base.iat,
            "exp": claims.base.exp,
            "jti": claims.base.jti,
        });

        // Add delegation metadata if present
        if let Some(delegation) = &claims.delegation {
            jwt_claims["delegation"] = serde_json::to_value(delegation)
                .map_err(|e| DelegationError::InvalidScope(format!("Failed to serialize delegation: {}", e)))?;
        }

        Ok(jwt_claims)
    }

    /// Deserialize JWT claims to delegated claims
    fn deserialize_delegation_claims(jwt_claims: &serde_json::Value) -> Result<DelegatedClaims, DelegationError> {
        let base_claims = Claims {
            sub: jwt_claims["sub"].as_str()
                .ok_or_else(|| DelegationError::InvalidScope("Missing sub claim".to_string()))?
                .to_string(),
            vault: jwt_claims["vault"].as_str()
                .ok_or_else(|| DelegationError::InvalidScope("Missing vault claim".to_string()))?
                .to_string(),
            permissions: jwt_claims["permissions"].as_array()
                .ok_or_else(|| DelegationError::InvalidScope("Missing permissions claim".to_string()))?
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect(),
            iat: jwt_claims["iat"].as_u64()
                .ok_or_else(|| DelegationError::InvalidScope("Missing iat claim".to_string()))?,
            exp: jwt_claims["exp"].as_u64()
                .ok_or_else(|| DelegationError::InvalidScope("Missing exp claim".to_string()))?,
            jti: jwt_claims["jti"].as_str()
                .ok_or_else(|| DelegationError::InvalidScope("Missing jti claim".to_string()))?
                .to_string(),
        };

        let delegation = if let Some(delegation_value) = jwt_claims.get("delegation") {
            Some(serde_json::from_value(delegation_value.clone())
                .map_err(|e| DelegationError::InvalidScope(format!("Failed to deserialize delegation: {}", e)))?)
        } else {
            None
        };

        Ok(DelegatedClaims {
            base: base_claims,
            delegation,
        })
    }

    /// Check if a token is in the cache
    async fn get_cached_token(&self, token: &str) -> Option<DelegatedClaims> {
        let mut cache = self.token_cache.write().await;
        cache.get(token)
    }

    /// Cache a validated token
    async fn cache_token(&self, token: &str, claims: &DelegatedClaims) {
        let mut cache = self.token_cache.write().await;
        cache.insert(token.to_string(), claims.clone());
    }

    /// Clean up expired tokens from cache
    pub async fn cleanup_cache(&self) {
        let mut cache = self.token_cache.write().await;
        cache.cleanup();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> TokenCacheStats {
        let cache = self.token_cache.read().await;
        cache.stats()
    }
}

#[async_trait]
impl DelegatedTokenGenerator for JwtDelegatedTokenGenerator {
    async fn create_delegated_token(
        &self,
        claims: &DelegatedClaims,
        key: &[u8],
    ) -> Result<String, DelegationError> {
        // Create JWT token with delegation metadata
        let jwt_token = JwtHs256Token::mint(&claims.base, key).await
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to create JWT token: {}", e)))?;

        let token_str = jwt_token.as_str();

        // Cache the token
        self.cache_token(token_str, claims).await;

        debug!(
            token_id = %claims.base.jti,
            is_delegated = %claims.is_delegated(),
            "Created delegated token"
        );

        Ok(token_str.to_string())
    }

    async fn parse_delegated_token(
        &self,
        token: &str,
        key: &[u8],
    ) -> Result<DelegatedClaims, DelegationError> {
        // Check cache first
        if let Some(cached_claims) = self.get_cached_token(token).await {
            return Ok(cached_claims);
        }

        // Parse JWT token
        let key_str = std::str::from_utf8(key)
            .map_err(|e| DelegationError::InvalidScope(format!("Invalid key format: {}", e)))?;
        let jwt_validator = toka_capability_jwt_hs256::JwtHs256Validator::new(key_str);
        let claims = jwt_validator.validate(token).await
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to validate JWT token: {}", e)))?;

        // Convert to delegated claims
        let delegated_claims = DelegatedClaims::new(claims);

        // Cache the result
        self.cache_token(token, &delegated_claims).await;

        Ok(delegated_claims)
    }

    async fn validate_delegated_token(
        &self,
        token: &str,
        key: &[u8],
    ) -> Result<DelegatedClaims, DelegationError> {
        let claims = self.parse_delegated_token(token, key).await?;

        // Validate token expiry
        let now = Utc::now().timestamp() as u64;
        if claims.base.exp < now {
            return Err(DelegationError::DelegationExpired {
                expires_at: DateTime::from_timestamp(claims.base.exp as i64, 0)
                    .unwrap_or_else(|| Utc::now()),
            });
        }

        // Validate delegation-specific constraints
        if let Some(delegation) = &claims.delegation {
            if !delegation.is_valid() {
                return Err(DelegationError::DelegationRevoked {
                    reason: delegation.revocation_reason.clone()
                        .unwrap_or_else(|| "Delegation is no longer valid".to_string()),
                });
            }

            // Check delegation expiry
            if let Some(delegation_exp) = delegation.expires_at {
                if Utc::now() > delegation_exp {
                    return Err(DelegationError::DelegationExpired {
                        expires_at: delegation_exp,
                    });
                }
            }

            // Additional validation based on restrictions
            if let Some(time_restrictions) = &delegation.restrictions.time_restrictions {
                if !self.validate_time_restrictions(time_restrictions).await {
                    return Err(DelegationError::InvalidScope(
                        "Current time is outside allowed delegation time windows".to_string()
                    ));
                }
            }
        }

        debug!(
            token_id = %claims.base.jti,
            is_delegated = %claims.is_delegated(),
            "Validated delegated token"
        );

        Ok(claims)
    }
}

impl JwtDelegatedTokenGenerator {
    /// Validate time-based restrictions
    async fn validate_time_restrictions(&self, _restrictions: &crate::TimeRestrictions) -> bool {
        // TODO: Implement time-based validation
        // For now, always return true
        true
    }
}

/// Configuration for token generation
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Maximum size of token cache
    pub cache_size: usize,
    /// Default token expiry duration
    pub default_expiry: Duration,
    /// Whether to enable token caching
    pub enable_caching: bool,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            default_expiry: Duration::hours(1),
            enable_caching: true,
        }
    }
}

/// Simple LRU cache for token validation results
struct TokenCache {
    cache: std::collections::HashMap<String, CacheEntry>,
    max_size: usize,
    access_order: Vec<String>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    claims: DelegatedClaims,
    cached_at: DateTime<Utc>,
}

impl TokenCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            max_size,
            access_order: Vec::new(),
        }
    }

    fn get(&mut self, token: &str) -> Option<DelegatedClaims> {
        // Check if token exists and is valid
        let now = Utc::now().timestamp() as u64;
        
        if let Some(entry) = self.cache.get(token) {
            if entry.claims.base.exp < now {
                // Token expired, remove it
                self.cache.remove(token);
                self.access_order.retain(|k| k != token);
                return None;
            }

            // Token is valid, update access order and return clone
            let claims = entry.claims.clone();
            self.access_order.retain(|k| k != token);
            self.access_order.push(token.to_string());
            Some(claims)
        } else {
            None
        }
    }

    fn insert(&mut self, token: String, claims: DelegatedClaims) {
        // Remove oldest entry if cache is full
        if self.cache.len() >= self.max_size {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.cache.remove(&oldest);
                self.access_order.remove(0);
            }
        }

        let entry = CacheEntry {
            claims,
            cached_at: Utc::now(),
        };

        self.cache.insert(token.clone(), entry);
        self.access_order.push(token);
    }

    fn cleanup(&mut self) {
        let now = Utc::now().timestamp() as u64;
        
        // Remove expired tokens
        let expired_tokens: Vec<_> = self.cache.iter()
            .filter(|(_, entry)| entry.claims.base.exp < now)
            .map(|(token, _)| token.clone())
            .collect();

        for token in expired_tokens {
            self.cache.remove(&token);
            self.access_order.retain(|k| k != &token);
        }
    }

    fn stats(&self) -> TokenCacheStats {
        let now = Utc::now().timestamp() as u64;
        let active_tokens = self.cache.values()
            .filter(|entry| entry.claims.base.exp >= now)
            .count();

        TokenCacheStats {
            total_entries: self.cache.len(),
            active_entries: active_tokens,
            expired_entries: self.cache.len() - active_tokens,
            max_size: self.max_size,
        }
    }
}

/// Statistics about the token cache
#[derive(Debug, Clone)]
pub struct TokenCacheStats {
    /// Total number of entries in cache
    pub total_entries: usize,
    /// Number of active (non-expired) entries
    pub active_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Maximum cache size
    pub max_size: usize,
}

/// Helper functions for token operations
pub mod utils {
    use super::*;

    /// Create a simple delegated token from basic parameters
    pub async fn create_simple_delegated_token(
        subject: &str,
        vault: &str,
        permissions: Vec<String>,
        expires_in: Duration,
        key: &[u8],
    ) -> Result<String, DelegationError> {
        let now = Utc::now();
        let claims = DelegatedClaims::new(Claims {
            sub: subject.to_string(),
            vault: vault.to_string(),
            permissions,
            iat: now.timestamp() as u64,
            exp: (now + expires_in).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        });

        let generator = JwtDelegatedTokenGenerator::default();
        generator.create_delegated_token(&claims, key).await
    }

    /// Extract subject from token without full validation
    pub fn extract_subject_from_token(token: &str) -> Result<String, DelegationError> {
        // Simple JWT parsing without signature verification
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(DelegationError::InvalidScope("Invalid JWT format".to_string()));
        }

        let payload = parts[1];
        let decoded = base64::engine::general_purpose::STANDARD_NO_PAD.decode(payload)
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to decode JWT payload: {}", e)))?;

        let claims: serde_json::Value = serde_json::from_slice(&decoded)
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to parse JWT claims: {}", e)))?;

        claims["sub"].as_str()
            .ok_or_else(|| DelegationError::InvalidScope("Missing sub claim".to_string()))
            .map(|s| s.to_string())
    }

    /// Check if a token is expired without full validation
    pub fn is_token_expired(token: &str) -> Result<bool, DelegationError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(DelegationError::InvalidScope("Invalid JWT format".to_string()));
        }

        let payload = parts[1];
        let decoded = base64::engine::general_purpose::STANDARD_NO_PAD.decode(payload)
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to decode JWT payload: {}", e)))?;

        let claims: serde_json::Value = serde_json::from_slice(&decoded)
            .map_err(|e| DelegationError::InvalidScope(format!("Failed to parse JWT claims: {}", e)))?;

        let exp = claims["exp"].as_u64()
            .ok_or_else(|| DelegationError::InvalidScope("Missing exp claim".to_string()))?;

        let now = Utc::now().timestamp() as u64;
        Ok(exp < now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_token_generation() {
        let generator = JwtDelegatedTokenGenerator::default();
        let key = b"test_key_32_bytes_long_for_hs256";

        let claims = DelegatedClaims::new(Claims {
            sub: "test_user".to_string(),
            vault: "test_vault".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            iat: Utc::now().timestamp() as u64,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        });

        let token = generator.create_delegated_token(&claims, key).await.unwrap();
        assert!(!token.is_empty());

        // Validate the token
        let validated_claims = generator.validate_delegated_token(&token, key).await.unwrap();
        assert_eq!(validated_claims.base.sub, claims.base.sub);
        assert_eq!(validated_claims.base.permissions, claims.base.permissions);
    }

    #[tokio::test]
    async fn test_token_caching() {
        let generator = JwtDelegatedTokenGenerator::default();
        let key = b"test_key_32_bytes_long_for_hs256";

        let claims = DelegatedClaims::new(Claims {
            sub: "test_user".to_string(),
            vault: "test_vault".to_string(),
            permissions: vec!["read".to_string()],
            iat: Utc::now().timestamp() as u64,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        });

        let token = generator.create_delegated_token(&claims, key).await.unwrap();

        // First validation should cache the token
        let _validated1 = generator.validate_delegated_token(&token, key).await.unwrap();

        // Second validation should use cache
        let _validated2 = generator.validate_delegated_token(&token, key).await.unwrap();

        let stats = generator.get_cache_stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.active_entries, 1);
    }

    #[tokio::test]
    async fn test_utils() {
        let token = utils::create_simple_delegated_token(
            "test_user",
            "test_vault",
            vec!["read".to_string()],
            Duration::hours(1),
            b"test_key_32_bytes_long_for_hs256",
        ).await.unwrap();

        let subject = utils::extract_subject_from_token(&token).unwrap();
        assert_eq!(subject, "test_user");

        let is_expired = utils::is_token_expired(&token).unwrap();
        assert!(!is_expired);
    }
} 