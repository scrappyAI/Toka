#![forbid(unsafe_code)]

//! **toka-rate-limiter** – Authentication rate limiting middleware for Toka
//!
//! This crate provides comprehensive rate limiting capabilities to protect against
//! brute force attacks, abuse, and resource exhaustion. It implements multiple
//! rate limiting algorithms and strategies:
//!
//! * **Token Bucket**: Burst-friendly with sustained rate control
//! * **Fixed Window**: Simple time window-based limiting
//! * **Sliding Window**: More accurate rate limiting with memory efficiency
//! * **Adaptive Limiting**: Dynamic rate adjustment based on conditions
//!
//! The crate integrates seamlessly with Toka's security framework and provides:
//! - Multi-dimensional rate limiting (IP, user, endpoint)
//! - Configurable policies and thresholds
//! - Audit integration for security monitoring
//! - Distributed rate limiting support
//! - Performance optimizations for high-throughput scenarios

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;
use uuid::Uuid;

pub mod algorithms;
pub mod middleware;
pub mod policies;
pub mod storage;

/// Rate limiting result indicating whether an operation should be allowed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Operation is allowed to proceed
    Allowed {
        /// Remaining capacity in the current window
        remaining: u64,
        /// Time until the next reset
        reset_after: Duration,
        /// Current limit being enforced
        limit: u64,
    },
    /// Operation is rate limited
    Limited {
        /// How long to wait before retrying
        retry_after: Duration,
        /// Current limit being enforced
        limit: u64,
        /// How many requests were made in the current window
        current_usage: u64,
    },
}

impl RateLimitResult {
    /// Check if the operation is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    /// Get retry after duration if rate limited
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            RateLimitResult::Limited { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }

    /// Get remaining capacity if allowed
    pub fn remaining(&self) -> Option<u64> {
        match self {
            RateLimitResult::Allowed { remaining, .. } => Some(*remaining),
            _ => None,
        }
    }
}

/// Rate limiting error types
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Storage backend error: {0}")]
    Storage(#[from] anyhow::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Rate limit exceeded: {0}")]
    LimitExceeded(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Identifier for rate limiting subjects (what we're rate limiting)
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitKey {
    /// Rate limit by IP address
    IpAddress(IpAddr),
    /// Rate limit by user ID
    UserId(String),
    /// Rate limit by API key
    ApiKey(String),
    /// Rate limit by endpoint
    Endpoint(String),
    /// Rate limit by custom key
    Custom(String),
    /// Composite key for multi-dimensional limiting
    Composite(Vec<String>),
}

impl RateLimitKey {
    /// Convert to string representation for storage
    pub fn to_storage_key(&self) -> String {
        match self {
            RateLimitKey::IpAddress(ip) => format!("ip:{}", ip),
            RateLimitKey::UserId(id) => format!("user:{}", id),
            RateLimitKey::ApiKey(key) => format!("apikey:{}", key),
            RateLimitKey::Endpoint(endpoint) => format!("endpoint:{}", endpoint),
            RateLimitKey::Custom(key) => format!("custom:{}", key),
            RateLimitKey::Composite(parts) => format!("composite:{}", parts.join(":")),
        }
    }

    /// Create composite key from multiple dimensions
    pub fn composite(parts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        RateLimitKey::Composite(parts.into_iter().map(|p| p.into()).collect())
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub limit: u64,
    /// Time window for the limit
    pub window: Duration,
    /// Rate limiting algorithm to use
    pub algorithm: RateLimitAlgorithm,
    /// Burst capacity (for token bucket)
    pub burst_capacity: Option<u64>,
    /// Whether to enable distributed rate limiting
    pub distributed: bool,
    /// Whether to enable adaptive rate limiting
    pub adaptive: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            limit: 100,
            window: Duration::minutes(1),
            algorithm: RateLimitAlgorithm::TokenBucket,
            burst_capacity: Some(10),
            distributed: false,
            adaptive: false,
        }
    }
}

/// Rate limiting algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    /// Token bucket algorithm (burst-friendly)
    TokenBucket,
    /// Fixed window algorithm (simple)
    FixedWindow,
    /// Sliding window algorithm (accurate)
    SlidingWindow,
    /// Adaptive algorithm (dynamic)
    Adaptive,
}

/// Authentication-specific rate limiting context
#[derive(Debug, Clone)]
pub struct AuthRateLimitContext {
    /// Client IP address
    pub ip_address: Option<IpAddr>,
    /// User identifier (if authenticated)
    pub user_id: Option<String>,
    /// API key (if using API key auth)
    pub api_key: Option<String>,
    /// Request endpoint/path
    pub endpoint: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<Uuid>,
    /// Whether this is a failed authentication attempt
    pub is_auth_failure: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AuthRateLimitContext {
    /// Create new authentication context
    pub fn new() -> Self {
        Self {
            ip_address: None,
            user_id: None,
            api_key: None,
            endpoint: None,
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
            is_auth_failure: false,
            metadata: HashMap::new(),
        }
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Mark as authentication failure
    pub fn with_auth_failure(mut self) -> Self {
        self.is_auth_failure = true;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Generate rate limiting keys for this context
    pub fn generate_keys(&self) -> Vec<RateLimitKey> {
        let mut keys = Vec::new();

        // Always rate limit by IP if available
        if let Some(ip) = self.ip_address {
            keys.push(RateLimitKey::IpAddress(ip));
        }

        // Rate limit by user if authenticated
        if let Some(ref user_id) = self.user_id {
            keys.push(RateLimitKey::UserId(user_id.clone()));
        }

        // Rate limit by API key if using API auth
        if let Some(ref api_key) = self.api_key {
            keys.push(RateLimitKey::ApiKey(api_key.clone()));
        }

        // Rate limit by endpoint
        if let Some(ref endpoint) = self.endpoint {
            keys.push(RateLimitKey::Endpoint(endpoint.clone()));
        }

        // For auth failures, create composite keys for more restrictive limiting
        if self.is_auth_failure {
            if let (Some(ip), Some(ref endpoint)) = (self.ip_address, &self.endpoint) {
                keys.push(RateLimitKey::composite([
                    format!("auth_fail_ip:{}", ip),
                    format!("endpoint:{}", endpoint),
                ]));
            }

            if let (Some(ref user_id), Some(ref endpoint)) = (&self.user_id, &self.endpoint) {
                keys.push(RateLimitKey::composite([
                    format!("auth_fail_user:{}", user_id),
                    format!("endpoint:{}", endpoint),
                ]));
            }
        }

        keys
    }
}

impl Default for AuthRateLimitContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for rate limiting storage backends
#[async_trait]
pub trait RateLimitStorage: Send + Sync {
    /// Get current usage for a key
    async fn get_usage(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>>;
    
    /// Update usage for a key
    async fn update_usage(&self, key: &RateLimitKey, usage: &RateLimitUsage) -> Result<()>;
    
    /// Increment usage atomically
    async fn increment_usage(&self, key: &RateLimitKey, amount: u64) -> Result<RateLimitUsage>;
    
    /// Clean up expired entries
    async fn cleanup_expired(&self) -> Result<usize>;
    
    /// Reset usage for a key
    async fn reset_usage(&self, key: &RateLimitKey) -> Result<()>;
}

/// Rate limiting usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsage {
    /// Current usage count
    pub count: u64,
    /// Window start time
    pub window_start: DateTime<Utc>,
    /// First request in window
    pub first_request: DateTime<Utc>,
    /// Last request in window
    pub last_request: DateTime<Utc>,
    /// Total requests ever (for analytics)
    pub total_requests: u64,
}

impl RateLimitUsage {
    /// Create new usage tracker
    pub fn new(count: u64, window_start: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            count,
            window_start,
            first_request: now,
            last_request: now,
            total_requests: count,
        }
    }

    /// Check if usage is within a time window
    pub fn is_within_window(&self, window_duration: Duration) -> bool {
        let now = Utc::now();
        now - self.window_start <= window_duration
    }

    /// Reset to new window
    pub fn reset_window(&mut self, new_start: DateTime<Utc>) {
        self.count = 0;
        self.window_start = new_start;
        self.first_request = new_start;
        self.last_request = new_start;
    }

    /// Increment usage
    pub fn increment(&mut self, amount: u64) {
        self.count += amount;
        self.total_requests += amount;
        self.last_request = Utc::now();
    }
}

/// Main rate limiter interface
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if an operation should be allowed
    async fn check_rate_limit(
        &self,
        context: &AuthRateLimitContext,
    ) -> Result<RateLimitResult, RateLimitError>;
    
    /// Record a successful operation (may affect adaptive limiting)
    async fn record_success(&self, context: &AuthRateLimitContext) -> Result<()>;
    
    /// Record a failed operation (may trigger stricter limiting)
    async fn record_failure(&self, context: &AuthRateLimitContext) -> Result<()>;
    
    /// Get current usage statistics
    async fn get_stats(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>>;
    
    /// Reset rate limits for a key (admin operation)
    async fn reset_limits(&self, key: &RateLimitKey) -> Result<()>;
}

/// Convenience re-exports for common usage
pub mod prelude {
    pub use super::{
        RateLimiter, RateLimitResult, RateLimitKey, RateLimitConfig,
        AuthRateLimitContext, RateLimitAlgorithm, RateLimitError,
        algorithms::{TokenBucketLimiter, RateLimitPolicy},
        middleware::AuthRateLimitMiddleware,
        policies::{CompositePolicy, GeographicalPolicy, TemporalPolicy},
        storage::MemoryRateLimitStorage,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_key_generation() {
        let ip = "192.168.1.1".parse().unwrap();
        let context = AuthRateLimitContext::new()
            .with_ip(ip)
            .with_user_id("user123")
            .with_endpoint("/api/login");

        let keys = context.generate_keys();
        
        assert!(keys.contains(&RateLimitKey::IpAddress(ip)));
        assert!(keys.contains(&RateLimitKey::UserId("user123".to_string())));
        assert!(keys.contains(&RateLimitKey::Endpoint("/api/login".to_string())));
    }

    #[test]
    fn test_rate_limit_result() {
        let allowed = RateLimitResult::Allowed {
            remaining: 50,
            reset_after: Duration::minutes(1),
            limit: 100,
        };

        assert!(allowed.is_allowed());
        assert_eq!(allowed.remaining(), Some(50));
        assert!(allowed.retry_after().is_none());

        let limited = RateLimitResult::Limited {
            retry_after: Duration::seconds(30),
            limit: 100,
            current_usage: 100,
        };

        assert!(!limited.is_allowed());
        assert!(limited.remaining().is_none());
        assert_eq!(limited.retry_after(), Some(Duration::seconds(30)));
    }

    #[test]
    fn test_auth_failure_keys() {
        let ip = "192.168.1.1".parse().unwrap();
        let context = AuthRateLimitContext::new()
            .with_ip(ip)
            .with_user_id("user123")
            .with_endpoint("/api/login")
            .with_auth_failure();

        let keys = context.generate_keys();
        
        // Should have composite keys for auth failures
        let has_auth_fail_composite = keys.iter().any(|k| {
            matches!(k, RateLimitKey::Composite(parts) if parts.iter().any(|p| p.contains("auth_fail")))
        });
        
        assert!(has_auth_fail_composite);
    }
} 