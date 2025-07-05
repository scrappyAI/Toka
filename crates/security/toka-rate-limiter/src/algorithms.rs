//! Rate limiting algorithms implementation

use crate::{
    RateLimiter, RateLimitResult, RateLimitError, RateLimitStorage, RateLimitUsage,
    AuthRateLimitContext, RateLimitConfig, RateLimitKey,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

/// Token bucket rate limiter implementation
/// 
/// The token bucket algorithm allows for burst traffic while maintaining
/// an average rate limit. Tokens are added to the bucket at a constant rate,
/// and each request consumes tokens. If no tokens are available, the request
/// is rate limited.
pub struct TokenBucketLimiter<S>
where
    S: RateLimitStorage,
{
    storage: Arc<S>,
    config: RateLimitConfig,
    policies: Vec<Arc<dyn RateLimitPolicy>>,
}

/// Token bucket state for a specific key
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBucketState {
    /// Current number of tokens in the bucket
    tokens: f64,
    /// Last time tokens were refilled
    last_refill: DateTime<Utc>,
    /// Maximum capacity of the bucket
    capacity: u64,
    /// Rate at which tokens are added (tokens per second)
    refill_rate: f64,
    /// Total requests processed
    total_requests: u64,
    /// Window start for analytics
    window_start: DateTime<Utc>,
}

impl TokenBucketState {
    /// Create new token bucket state
    fn new(capacity: u64, refill_rate: f64) -> Self {
        let now = Utc::now();
        Self {
            tokens: capacity as f64,
            last_refill: now,
            capacity,
            refill_rate,
            total_requests: 0,
            window_start: now,
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = now - self.last_refill;
        let elapsed_seconds = elapsed.num_milliseconds() as f64 / 1000.0;
        
        // Add tokens based on elapsed time
        let new_tokens = elapsed_seconds * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;
    }

    /// Try to consume tokens from the bucket
    fn consume(&mut self, amount: u64) -> bool {
        self.refill();
        
        if self.tokens >= amount as f64 {
            self.tokens -= amount as f64;
            self.total_requests += amount;
            true
        } else {
            false
        }
    }

    /// Get time until enough tokens are available
    fn time_until_available(&self, amount: u64) -> Duration {
        let tokens_needed = amount as f64 - self.tokens;
        if tokens_needed <= 0.0 {
            return Duration::zero();
        }
        
        let seconds_needed = tokens_needed / self.refill_rate;
        Duration::milliseconds((seconds_needed * 1000.0) as i64)
    }

    /// Convert to RateLimitUsage for storage
    fn to_usage(&self, window_duration: Duration) -> RateLimitUsage {
        let now = Utc::now();
        RateLimitUsage {
            count: (self.capacity as f64 - self.tokens) as u64,
            window_start: self.window_start,
            first_request: self.window_start,
            last_request: now,
            total_requests: self.total_requests,
        }
    }

    /// Create from RateLimitUsage
    fn from_usage(usage: &RateLimitUsage, capacity: u64, refill_rate: f64) -> Self {
        let now = Utc::now();
        Self {
            tokens: (capacity - usage.count) as f64,
            last_refill: now,
            capacity,
            refill_rate,
            total_requests: usage.total_requests,
            window_start: usage.window_start,
        }
    }
}

impl<S> TokenBucketLimiter<S>
where
    S: RateLimitStorage,
{
    /// Create a new token bucket rate limiter
    pub fn new(storage: Arc<S>, config: RateLimitConfig) -> Result<Self, RateLimitError> {
        if config.limit == 0 {
            return Err(RateLimitError::InvalidConfig(
                "Rate limit must be greater than 0".to_string(),
            ));
        }

        if config.window.num_seconds() <= 0 {
            return Err(RateLimitError::InvalidConfig(
                "Window duration must be positive".to_string(),
            ));
        }

        Ok(Self {
            storage,
            config,
            policies: Vec::new(),
        })
    }

    /// Add a rate limiting policy
    pub fn with_policy(mut self, policy: Arc<dyn RateLimitPolicy>) -> Self {
        self.policies.push(policy);
        self
    }

    /// Calculate refill rate (tokens per second)
    fn calculate_refill_rate(&self) -> f64 {
        self.config.limit as f64 / self.config.window.num_seconds() as f64
    }

    /// Get bucket capacity
    fn get_capacity(&self) -> u64 {
        self.config.burst_capacity.unwrap_or(self.config.limit)
    }

    /// Check rate limit for a single key
    async fn check_key_rate_limit(&self, key: &RateLimitKey) -> Result<RateLimitResult, RateLimitError> {
        let capacity = self.get_capacity();
        let refill_rate = self.calculate_refill_rate();

        // Get current state from storage
        let usage = self.storage.get_usage(key).await
            .map_err(RateLimitError::Storage)?;

        let mut state = match usage {
            Some(usage) => TokenBucketState::from_usage(&usage, capacity, refill_rate),
            None => TokenBucketState::new(capacity, refill_rate),
        };

        // Try to consume one token
        if state.consume(1) {
            // Request allowed - update storage
            let new_usage = state.to_usage(self.config.window);
            self.storage.update_usage(key, &new_usage).await
                .map_err(RateLimitError::Storage)?;

            Ok(RateLimitResult::Allowed {
                remaining: state.tokens as u64,
                reset_after: Duration::seconds(
                    ((capacity as f64 - state.tokens) / refill_rate) as i64
                ),
                limit: self.config.limit,
            })
        } else {
            // Request rate limited
            let retry_after = state.time_until_available(1);
            
            debug!(
                key = %key.to_storage_key(),
                tokens = state.tokens,
                capacity = capacity,
                "Rate limit exceeded"
            );

            Ok(RateLimitResult::Limited {
                retry_after,
                limit: self.config.limit,
                current_usage: (capacity as f64 - state.tokens) as u64,
            })
        }
    }

    /// Apply additional policies to modify the result
    async fn apply_policies(
        &self,
        context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        let mut final_result = result;

        for policy in &self.policies {
            final_result = policy.apply(context, final_result).await?;
        }

        Ok(final_result)
    }
}

#[async_trait]
impl<S> RateLimiter for TokenBucketLimiter<S>
where
    S: RateLimitStorage + 'static,
{
    async fn check_rate_limit(
        &self,
        context: &AuthRateLimitContext,
    ) -> Result<RateLimitResult, RateLimitError> {
        let keys = context.generate_keys();
        
        if keys.is_empty() {
            warn!("No rate limiting keys generated for context");
            return Ok(RateLimitResult::Allowed {
                remaining: self.config.limit,
                reset_after: self.config.window,
                limit: self.config.limit,
            });
        }

        // Check all keys - if any are rate limited, the request is rate limited
        for key in &keys {
            let result = self.check_key_rate_limit(key).await?;
            
            if !result.is_allowed() {
                debug!(
                    key = %key.to_storage_key(),
                    "Rate limit exceeded for key"
                );
                return self.apply_policies(context, result).await;
            }
        }

        // All keys passed - find the most restrictive remaining count
        let mut min_remaining = u64::MAX;
        let mut max_reset_after = Duration::zero();

        for key in &keys {
            if let Ok(RateLimitResult::Allowed { remaining, reset_after, .. }) = 
                self.check_key_rate_limit(key).await {
                min_remaining = min_remaining.min(remaining);
                max_reset_after = max_reset_after.max(reset_after);
            }
        }

        let result = RateLimitResult::Allowed {
            remaining: min_remaining.min(self.config.limit),
            reset_after: max_reset_after,
            limit: self.config.limit,
        };

        self.apply_policies(context, result).await
    }

    async fn record_success(&self, _context: &AuthRateLimitContext) -> Result<()> {
        // Token bucket doesn't need explicit success recording
        // as the consumption already happened during check
        Ok(())
    }

    async fn record_failure(&self, context: &AuthRateLimitContext) -> Result<()> {
        // For failures, we might want to apply additional restrictions
        // This could be implemented via policies for more complex scenarios
        
        if context.is_auth_failure {
            debug!(
                ip = ?context.ip_address,
                user_id = ?context.user_id,
                endpoint = ?context.endpoint,
                "Authentication failure recorded"
            );
        }

        Ok(())
    }

    async fn get_stats(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>> {
        self.storage.get_usage(key).await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn reset_limits(&self, key: &RateLimitKey) -> Result<()> {
        self.storage.reset_usage(key).await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

/// Trait for rate limiting policies that can modify rate limiting behavior
#[async_trait]
pub trait RateLimitPolicy: Send + Sync {
    /// Apply policy to modify rate limiting result
    async fn apply(
        &self,
        context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError>;
}

/// Policy that applies stricter limits for authentication failures
pub struct AuthFailurePolicy {
    /// Factor to multiply rate limits by for auth failures
    failure_multiplier: f64,
    /// Maximum duration to extend retry times
    max_retry_extension: Duration,
}

impl AuthFailurePolicy {
    /// Create new auth failure policy
    pub fn new(failure_multiplier: f64, max_retry_extension: Duration) -> Self {
        Self {
            failure_multiplier,
            max_retry_extension,
        }
    }
}

#[async_trait]
impl RateLimitPolicy for AuthFailurePolicy {
    async fn apply(
        &self,
        context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        if !context.is_auth_failure {
            return Ok(result);
        }

        match result {
            RateLimitResult::Limited { retry_after, limit, current_usage } => {
                let extended_retry = Duration::milliseconds(
                    (retry_after.num_milliseconds() as f64 * self.failure_multiplier) as i64
                ).min(self.max_retry_extension);

                warn!(
                    ip = ?context.ip_address,
                    user_id = ?context.user_id,
                    original_retry = ?retry_after,
                    extended_retry = ?extended_retry,
                    "Extending retry time for auth failure"
                );

                Ok(RateLimitResult::Limited {
                    retry_after: extended_retry,
                    limit,
                    current_usage,
                })
            }
            other => Ok(other),
        }
    }
}

/// Policy that implements adaptive rate limiting based on success/failure rates
pub struct AdaptivePolicy {
    /// Threshold for failure rate to trigger adaptive limiting
    failure_rate_threshold: f64,
    /// Factor to reduce limits by when adaptive limiting is triggered
    adaptive_reduction_factor: f64,
}

impl AdaptivePolicy {
    /// Create new adaptive policy
    pub fn new(failure_rate_threshold: f64, adaptive_reduction_factor: f64) -> Self {
        Self {
            failure_rate_threshold,
            adaptive_reduction_factor,
        }
    }
}

#[async_trait]
impl RateLimitPolicy for AdaptivePolicy {
    async fn apply(
        &self,
        _context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        // Adaptive policy would analyze historical data to adjust limits
        // This is a simplified implementation - full implementation would
        // track success/failure rates over time and adjust accordingly
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct MockStorage {
        data: Mutex<HashMap<String, RateLimitUsage>>,
    }

    #[async_trait]
    impl RateLimitStorage for MockStorage {
        async fn get_usage(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>> {
            let data = self.data.lock().await;
            Ok(data.get(&key.to_storage_key()).cloned())
        }

        async fn update_usage(&self, key: &RateLimitKey, usage: &RateLimitUsage) -> Result<()> {
            let mut data = self.data.lock().await;
            data.insert(key.to_storage_key(), usage.clone());
            Ok(())
        }

        async fn increment_usage(&self, key: &RateLimitKey, amount: u64) -> Result<RateLimitUsage> {
            let mut data = self.data.lock().await;
            let usage = data.entry(key.to_storage_key())
                .or_insert_with(|| RateLimitUsage::new(0, Utc::now()));
            usage.increment(amount);
            Ok(usage.clone())
        }

        async fn cleanup_expired(&self) -> Result<usize> {
            Ok(0)
        }

        async fn reset_usage(&self, key: &RateLimitKey) -> Result<()> {
            let mut data = self.data.lock().await;
            data.remove(&key.to_storage_key());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_token_bucket_basic() {
        let storage = Arc::new(MockStorage::default());
        let config = RateLimitConfig {
            limit: 10,
            window: Duration::seconds(10),
            ..Default::default()
        };

        let limiter = TokenBucketLimiter::new(storage, config).unwrap();
        let context = AuthRateLimitContext::new()
            .with_ip("192.168.1.1".parse().unwrap());

        // First request should be allowed
        let result = limiter.check_rate_limit(&context).await.unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_auth_failure_policy() {
        let policy = AuthFailurePolicy::new(2.0, Duration::minutes(5));
        let context = AuthRateLimitContext::new().with_auth_failure();

        let original = RateLimitResult::Limited {
            retry_after: Duration::seconds(30),
            limit: 100,
            current_usage: 100,
        };

        let result = policy.apply(&context, original).await.unwrap();
        
        if let RateLimitResult::Limited { retry_after, .. } = result {
            assert!(retry_after > Duration::seconds(30));
        } else {
            panic!("Expected Limited result");
        }
    }
} 