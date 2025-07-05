//! Enhanced JWT validator with key rotation support

use crate::{KeyStore, KeyVersion, events::{AuditEventHandler, ClientInfo, SecurityAlertType, AlertSeverity, AuditStore}};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use toka_capability_core::prelude::{Claims, TokenValidator};
use toka_capability_core::{Result as CoreResult, Error as CoreError};
use tracing::{debug, error};
use uuid::Uuid;

/// Enhanced JWT validator that supports multiple active keys and comprehensive audit logging
pub struct RotatingJwtValidator<S, A>
where
    S: KeyStore,
    A: AuditStore,
{
    key_store: Arc<S>,
    audit_handler: Option<Arc<AuditEventHandler<A>>>,
    validation_config: ValidationConfig,
    key_cache: Arc<RwLock<KeyCache>>,
    metrics: Arc<RwLock<ValidationMetrics>>,
}

/// Configuration for JWT validation behavior
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to enable strict validation (reject tokens with expired keys)
    pub strict_validation: bool,
    /// Maximum allowed clock skew in seconds
    pub clock_skew_seconds: u64,
    /// Whether to cache keys for performance
    pub enable_key_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum number of validation failures before alerting
    pub max_validation_failures: usize,
    /// Time window for counting validation failures (seconds)
    pub failure_window_seconds: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            clock_skew_seconds: 30,
            enable_key_cache: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_validation_failures: 10,
            failure_window_seconds: 300, // 5 minutes
        }
    }
}

/// Cache for JWT validation keys
#[derive(Debug, Default)]
struct KeyCache {
    keys: HashMap<Uuid, CachedKey>,
    last_refresh: Option<DateTime<Utc>>,
}

/// Cached key with metadata
#[derive(Clone)]
struct CachedKey {
    key_version: KeyVersion,
    cached_at: DateTime<Utc>,
    decoding_key: DecodingKey,
}

impl std::fmt::Debug for CachedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedKey")
            .field("key_version", &self.key_version)
            .field("cached_at", &self.cached_at)
            .field("decoding_key", &"<redacted>")
            .finish()
    }
}

/// Metrics for validation operations
#[derive(Debug, Default, Clone)]
pub struct ValidationMetrics {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub key_rotations_detected: u64,
    pub suspicious_activities: u64,
    pub validation_failures_by_key: HashMap<Uuid, u64>,
    pub last_validation_time: Option<DateTime<Utc>>,
}

/// Validation failure reasons for detailed monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationFailureReason {
    /// Token has expired
    TokenExpired,
    /// Key not found or inactive
    KeyNotFound,
    /// Invalid signature
    InvalidSignature,
    /// Malformed token
    MalformedToken,
    /// Clock skew too large
    ClockSkew,
    /// Key has been revoked
    KeyRevoked,
    /// Rate limit exceeded
    RateLimited,
}

impl<S, A> RotatingJwtValidator<S, A>
where
    S: KeyStore + 'static,
    A: AuditStore + 'static,
{
    /// Create a new rotating JWT validator
    pub fn new(
        key_store: Arc<S>,
        audit_handler: Option<Arc<AuditEventHandler<A>>>,
        config: ValidationConfig,
    ) -> Self {
        Self {
            key_store,
            audit_handler,
            validation_config: config,
            key_cache: Arc::new(RwLock::new(KeyCache::default())),
            metrics: Arc::new(RwLock::new(ValidationMetrics::default())),
        }
    }

    /// Validate a JWT token with comprehensive audit logging
    pub async fn validate_with_context(
        &self,
        token: &str,
        client_info: Option<ClientInfo>,
    ) -> CoreResult<Claims> {
        let start_time = std::time::Instant::now();
        let validation_result = self.validate(token).await;
        let duration = start_time.elapsed();

        // Log validation attempt
        if let Some(ref audit_handler) = self.audit_handler {
            // Extract key ID from token header for logging
            let key_id = self.extract_key_id_from_token(token).unwrap_or_else(|| Uuid::new_v4());
            
            if let Err(e) = audit_handler.log_key_validation(
                key_id,
                validation_result.is_ok(),
                client_info,
            ).await {
                error!("Failed to log validation event: {}", e);
            }

            // Check for suspicious patterns
            if validation_result.is_err() {
                self.check_for_suspicious_activity(token, &validation_result).await;
            }
        }

        // Update metrics
        self.update_validation_metrics(&validation_result, duration).await;

        validation_result
    }

    /// Get current validation metrics
    pub async fn get_metrics(&self) -> ValidationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Refresh the key cache
    pub async fn refresh_cache(&self) -> Result<()> {
        if !self.validation_config.enable_key_cache {
            return Ok(());
        }

        debug!("Refreshing key cache");
        let valid_keys = self.key_store.get_valid_keys().await?;
        let mut cache = self.key_cache.write().await;
        
        cache.keys.clear();
        
        for key in valid_keys {
            if let Ok(key_bytes) = key.key_bytes() {
                let decoding_key = DecodingKey::from_secret(&key_bytes);
                cache.keys.insert(key.id, CachedKey {
                    key_version: key,
                    cached_at: Utc::now(),
                    decoding_key,
                });
            }
        }
        
        cache.last_refresh = Some(Utc::now());
        debug!("Key cache refreshed with {} keys", cache.keys.len());
        Ok(())
    }

    /// Check if cache needs refresh
    async fn should_refresh_cache(&self) -> bool {
        if !self.validation_config.enable_key_cache {
            return false;
        }

        let cache = self.key_cache.read().await;
        match cache.last_refresh {
            Some(last_refresh) => {
                let age = Utc::now() - last_refresh;
                age.num_seconds() > self.validation_config.cache_ttl_seconds as i64
            }
            None => true,
        }
    }

    /// Get decoding key for validation
    async fn get_decoding_key(&self, key_id: &Uuid) -> Result<Option<DecodingKey>> {
        if self.validation_config.enable_key_cache {
            // Check cache first
            {
                let cache = self.key_cache.read().await;
                if let Some(cached_key) = cache.keys.get(key_id) {
                    let age = Utc::now() - cached_key.cached_at;
                    if age.num_seconds() <= self.validation_config.cache_ttl_seconds as i64 {
                        let mut metrics = self.metrics.write().await;
                        metrics.cache_hits += 1;
                        return Ok(Some(cached_key.decoding_key.clone()));
                    }
                }
            }

            // Cache miss - refresh if needed
            if self.should_refresh_cache().await {
                self.refresh_cache().await?;
            }

            // Try cache again
            let cache = self.key_cache.read().await;
            if let Some(cached_key) = cache.keys.get(key_id) {
                let mut metrics = self.metrics.write().await;
                metrics.cache_hits += 1;
                return Ok(Some(cached_key.decoding_key.clone()));
            }

            // Still not found
            let mut metrics = self.metrics.write().await;
            metrics.cache_misses += 1;
        }

        // Fallback to direct key store lookup
        if let Some(key) = self.key_store.get_key(key_id).await? {
            if key.is_valid_for_validation() {
                let key_bytes = key.key_bytes()?;
                return Ok(Some(DecodingKey::from_secret(&key_bytes)));
            }
        }

        Ok(None)
    }

    /// Extract key ID from token header (simplified - in practice would parse JWT header)
    fn extract_key_id_from_token(&self, _token: &str) -> Option<Uuid> {
        // In a real implementation, this would parse the JWT header
        // and extract the key ID from a custom claim like "kid"
        None
    }

    /// Check for suspicious validation patterns
    async fn check_for_suspicious_activity(
        &self,
        token: &str,
        validation_result: &CoreResult<Claims>,
    ) {
        if let Some(ref audit_handler) = self.audit_handler {
            let now = Utc::now();
            
            // Check for repeated failures
            let mut metrics = self.metrics.write().await;
            metrics.suspicious_activities += 1;
            
            // In a real implementation, this would track failures by IP, user, etc.
            if metrics.failed_validations % (self.validation_config.max_validation_failures as u64) == 0 {
                if let Err(e) = audit_handler.log_security_alert(
                    SecurityAlertType::RepeatedFailures,
                    format!("High number of validation failures detected: {}", metrics.failed_validations),
                    AlertSeverity::Medium,
                    serde_json::json!({
                        "total_failures": metrics.failed_validations,
                        "time_window": self.validation_config.failure_window_seconds
                    }),
                ).await {
                    error!("Failed to log security alert: {}", e);
                }
            }

            // Check for potential token reuse
            if token.len() > 1000 {
                if let Err(e) = audit_handler.log_security_alert(
                    SecurityAlertType::InvalidKeyFormat,
                    "Unusually large token detected".to_string(),
                    AlertSeverity::Low,
                    serde_json::json!({
                        "token_length": token.len()
                    }),
                ).await {
                    error!("Failed to log security alert: {}", e);
                }
            }
        }
    }

    /// Update validation metrics
    async fn update_validation_metrics(
        &self,
        result: &CoreResult<Claims>,
        duration: std::time::Duration,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.total_validations += 1;
        metrics.last_validation_time = Some(Utc::now());

        match result {
            Ok(_) => {
                metrics.successful_validations += 1;
            }
            Err(_) => {
                metrics.failed_validations += 1;
            }
        }

        // Log slow validations
        if duration.as_millis() > 100 {
            debug!("Slow validation detected: {:?}", duration);
        }
    }
}

#[async_trait]
impl<S, A> TokenValidator for RotatingJwtValidator<S, A>
where
    S: KeyStore + 'static,
    A: AuditStore + 'static,
{
    async fn validate(&self, raw: &str) -> CoreResult<Claims> {
        // Try to validate with all valid keys
        let valid_keys = self.key_store.get_valid_keys().await
            .map_err(|e| CoreError::new(&format!("Failed to get valid keys: {}", e)))?;

        if valid_keys.is_empty() {
            return Err(CoreError::new("No valid keys available for validation"));
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = self.validation_config.clock_skew_seconds;

        // Try each valid key until one works
        for key in valid_keys {
            let key_bytes = key.key_bytes()
                .map_err(|e| CoreError::new(&format!("Failed to get key bytes: {}", e)))?;
            
            let decoding_key = DecodingKey::from_secret(&key_bytes);
            
            match decode::<Claims>(raw, &decoding_key, &validation) {
                Ok(token_data) => {
                    debug!("Token validated successfully with key generation {}", key.generation);
                    return Ok(token_data.claims);
                }
                Err(e) => {
                    debug!("Validation failed with key {}: {}", key.id, e);
                    continue;
                }
            }
        }

        Err(CoreError::new("Token validation failed with all available keys"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyStore;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    use chrono::Duration;
    use jsonwebtoken::{encode, Header, EncodingKey};

    #[derive(Default)]
    struct MockKeyStore {
        keys: Mutex<HashMap<Uuid, KeyVersion>>,
    }

    #[async_trait]
    impl KeyStore for MockKeyStore {
        async fn store_key(&self, key: &KeyVersion) -> Result<()> {
            self.keys.lock().await.insert(key.id, key.clone());
            Ok(())
        }

        async fn get_key(&self, key_id: &Uuid) -> Result<Option<KeyVersion>> {
            Ok(self.keys.lock().await.get(key_id).cloned())
        }

        async fn get_active_key(&self) -> Result<Option<KeyVersion>> {
            let keys = self.keys.lock().await;
            Ok(keys.values().find(|k| k.is_active).cloned())
        }

        async fn get_valid_keys(&self) -> Result<Vec<KeyVersion>> {
            let keys = self.keys.lock().await;
            Ok(keys.values().filter(|k| k.is_valid_for_validation()).cloned().collect())
        }

        async fn deactivate_key(&self, key_id: &Uuid) -> Result<()> {
            if let Some(key) = self.keys.lock().await.get_mut(key_id) {
                key.is_active = false;
            }
            Ok(())
        }

        async fn cleanup_expired_keys(&self) -> Result<usize> {
            let mut keys = self.keys.lock().await;
            let before_count = keys.len();
            keys.retain(|_, v| v.is_valid_for_validation());
            Ok(before_count - keys.len())
        }

        async fn get_all_keys(&self) -> Result<Vec<KeyVersion>> {
            Ok(self.keys.lock().await.values().cloned().collect())
        }
    }

    #[derive(Default)]
    struct MockAuditStore {
        events: Mutex<Vec<crate::events::AuditEvent>>,
    }

    #[async_trait]
    impl crate::events::AuditStore for MockAuditStore {
        async fn store_event(&self, event: &crate::events::AuditEvent) -> Result<()> {
            self.events.lock().await.push(event.clone());
            Ok(())
        }

        async fn query_events(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _event_types: Option<Vec<String>>,
        ) -> Result<Vec<crate::events::AuditEvent>> {
            Ok(self.events.lock().await.clone())
        }

        async fn get_alerts(&self, _min_severity: AlertSeverity) -> Result<Vec<crate::events::AuditEvent>> {
            Ok(vec![])
        }

        async fn archive_events(&self, _before: DateTime<Utc>) -> Result<usize> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_rotating_validator() {
        let key_store = Arc::new(MockKeyStore::default());
        let audit_store = Arc::new(MockAuditStore::default());
        let audit_handler = Arc::new(AuditEventHandler::new(
            audit_store.clone(),
            crate::events::AuditConfig::default(),
        ));

        let validator = RotatingJwtValidator::new(
            key_store.clone(),
            Some(audit_handler),
            ValidationConfig::default(),
        );

        // Create a test key
        let key = KeyVersion::new(1, Duration::hours(24), Duration::hours(1)).unwrap();
        key_store.store_key(&key).await.unwrap();

        // Create a test token
        let claims = toka_capability_core::Claims {
            sub: "test-user".to_string(),
            vault: "test-vault".to_string(),
            permissions: vec!["read".to_string()],
            iat: Utc::now().timestamp() as u64,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        };

        let key_bytes = key.key_bytes().unwrap();
        let encoding_key = EncodingKey::from_secret(&key_bytes);
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();

        // Test validation
        let result = validator.validate(&token).await;
        assert!(result.is_ok());

        let metrics = validator.get_metrics().await;
        assert_eq!(metrics.total_validations, 1);
        assert_eq!(metrics.successful_validations, 1);
    }
} 