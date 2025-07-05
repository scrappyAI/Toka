//! Middleware for integrating rate limiting with web frameworks

use crate::{
    RateLimiter, RateLimitResult, RateLimitError, AuthRateLimitContext,
    RateLimitConfig,
};
use anyhow::Result;
use chrono::Duration;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Authentication rate limiting middleware
/// 
/// This middleware can be used with web frameworks to automatically
/// apply rate limiting to authentication endpoints and other routes.
pub struct AuthRateLimitMiddleware<R>
where
    R: RateLimiter,
{
    rate_limiter: Arc<R>,
    config: MiddlewareConfig,
}

/// Configuration for the rate limiting middleware
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    /// Skip rate limiting for certain paths
    pub skip_paths: Vec<String>,
    /// Custom rate limiting rules per endpoint
    pub endpoint_configs: std::collections::HashMap<String, RateLimitConfig>,
    /// Whether to include rate limit headers in responses
    pub include_headers: bool,
    /// Custom header prefix (default: "X-RateLimit-")
    pub header_prefix: String,
    /// Whether to log rate limit events
    pub enable_logging: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            skip_paths: vec!["/health".to_string(), "/metrics".to_string()],
            endpoint_configs: std::collections::HashMap::new(),
            include_headers: true,
            header_prefix: "X-RateLimit-".to_string(),
            enable_logging: true,
        }
    }
}

impl<R> AuthRateLimitMiddleware<R>
where
    R: RateLimiter,
{
    /// Create a new rate limiting middleware
    pub fn new(rate_limiter: Arc<R>, config: MiddlewareConfig) -> Self {
        Self {
            rate_limiter,
            config,
        }
    }

    /// Create a new rate limiting middleware with default configuration
    pub fn with_defaults(rate_limiter: Arc<R>) -> Self {
        Self::new(rate_limiter, MiddlewareConfig::default())
    }

    /// Check if a path should be skipped
    pub fn should_skip_path(&self, path: &str) -> bool {
        self.config.skip_paths.iter().any(|skip_path| {
            path.starts_with(skip_path)
        })
    }

    /// Extract rate limiting context from request information
    pub fn extract_context(
        &self,
        ip: Option<IpAddr>,
        user_id: Option<String>,
        path: &str,
        user_agent: Option<String>,
        is_auth_failure: bool,
    ) -> AuthRateLimitContext {
        let mut context = AuthRateLimitContext::new()
            .with_endpoint(path.to_string())
            .with_request_id(Uuid::new_v4());

        if let Some(ip) = ip {
            context = context.with_ip(ip);
        }

        if let Some(user_id) = user_id {
            context = context.with_user_id(user_id);
        }

        if let Some(user_agent) = user_agent {
            context = context.with_metadata("user_agent", user_agent);
        }

        if is_auth_failure {
            context = context.with_auth_failure();
        }

        context
    }

    /// Apply rate limiting and return the result
    pub async fn apply_rate_limit(
        &self,
        context: &AuthRateLimitContext,
    ) -> Result<RateLimitResult, RateLimitError> {
        let result = self.rate_limiter.check_rate_limit(context).await?;

        if self.config.enable_logging {
            match &result {
                RateLimitResult::Allowed { remaining, limit, .. } => {
                    info!(
                        ip = ?context.ip_address,
                        user_id = ?context.user_id,
                        endpoint = ?context.endpoint,
                        remaining = remaining,
                        limit = limit,
                        "Rate limit check passed"
                    );
                }
                RateLimitResult::Limited { retry_after, limit, current_usage } => {
                    warn!(
                        ip = ?context.ip_address,
                        user_id = ?context.user_id,
                        endpoint = ?context.endpoint,
                        retry_after = ?retry_after,
                        limit = limit,
                        current_usage = current_usage,
                        "Rate limit exceeded"
                    );
                }
            }
        }

        Ok(result)
    }

    /// Generate HTTP headers for rate limiting information
    pub fn generate_headers(&self, result: &RateLimitResult) -> Vec<(String, String)> {
        if !self.config.include_headers {
            return vec![];
        }

        let mut headers = vec![];
        let prefix = &self.config.header_prefix;

        match result {
            RateLimitResult::Allowed { remaining, reset_after, limit } => {
                headers.push((format!("{}Limit", prefix), limit.to_string()));
                headers.push((format!("{}Remaining", prefix), remaining.to_string()));
                headers.push((format!("{}Reset", prefix), reset_after.num_seconds().to_string()));
            }
            RateLimitResult::Limited { retry_after, limit, current_usage } => {
                headers.push((format!("{}Limit", prefix), limit.to_string()));
                headers.push((format!("{}Remaining", prefix), "0".to_string()));
                headers.push((format!("{}Retry-After", prefix), retry_after.num_seconds().to_string()));
                headers.push((format!("{}Current", prefix), current_usage.to_string()));
            }
        }

        headers
    }

    /// Record a successful authentication (may affect adaptive limiting)
    pub async fn record_success(&self, context: &AuthRateLimitContext) -> Result<()> {
        self.rate_limiter.record_success(context).await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Record a failed authentication (may trigger stricter limiting)
    pub async fn record_failure(&self, context: &AuthRateLimitContext) -> Result<()> {
        self.rate_limiter.record_failure(context).await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

/// HTTP status codes for rate limiting responses
pub mod status_codes {
    /// Too Many Requests (rate limit exceeded)
    pub const TOO_MANY_REQUESTS: u16 = 429;
    /// Forbidden (may be used for severe rate limiting violations)
    pub const FORBIDDEN: u16 = 403;
    /// Service Unavailable (may be used when rate limiter is overloaded)
    pub const SERVICE_UNAVAILABLE: u16 = 503;
}

/// Standard HTTP response for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitResponse {
    /// HTTP status code
    pub status: u16,
    /// Response body
    pub body: String,
    /// HTTP headers
    pub headers: Vec<(String, String)>,
}

impl RateLimitResponse {
    /// Create a rate limit exceeded response
    pub fn too_many_requests(
        retry_after: Duration,
        limit: u64,
        headers: Vec<(String, String)>,
    ) -> Self {
        Self {
            status: status_codes::TOO_MANY_REQUESTS,
            body: serde_json::json!({
                "error": "Too Many Requests",
                "message": format!("Rate limit exceeded. Try again in {} seconds.", retry_after.num_seconds()),
                "retry_after": retry_after.num_seconds(),
                "limit": limit
            }).to_string(),
            headers,
        }
    }

    /// Create a forbidden response for severe violations
    pub fn forbidden(headers: Vec<(String, String)>) -> Self {
        Self {
            status: status_codes::FORBIDDEN,
            body: serde_json::json!({
                "error": "Forbidden",
                "message": "Request blocked due to rate limiting policy violation"
            }).to_string(),
            headers,
        }
    }

    /// Create a service unavailable response
    pub fn service_unavailable(headers: Vec<(String, String)>) -> Self {
        Self {
            status: status_codes::SERVICE_UNAVAILABLE,
            body: serde_json::json!({
                "error": "Service Unavailable",
                "message": "Rate limiting service is temporarily unavailable"
            }).to_string(),
            headers,
        }
    }
}

/// Rate limiting policy for different endpoint types
#[derive(Debug, Clone)]
pub enum EndpointPolicy {
    /// Authentication endpoints (login, token refresh, etc.)
    Authentication {
        /// Normal rate limit
        normal_limit: u64,
        /// Stricter limit for failures
        failure_limit: u64,
        /// Time window
        window: Duration,
    },
    /// API endpoints
    Api {
        /// Requests per window
        limit: u64,
        /// Time window
        window: Duration,
        /// Burst capacity
        burst: Option<u64>,
    },
    /// Public endpoints (less restrictive)
    Public {
        /// Requests per window
        limit: u64,
        /// Time window
        window: Duration,
    },
    /// Admin endpoints (very restrictive)
    Admin {
        /// Requests per window
        limit: u64,
        /// Time window
        window: Duration,
        /// Require additional authentication
        require_auth: bool,
    },
}

impl EndpointPolicy {
    /// Convert to rate limit config
    pub fn to_config(&self) -> RateLimitConfig {
        match self {
            EndpointPolicy::Authentication { normal_limit, window, .. } => {
                RateLimitConfig {
                    limit: *normal_limit,
                    window: *window,
                    burst_capacity: Some(*normal_limit / 10),
                    ..Default::default()
                }
            }
            EndpointPolicy::Api { limit, window, burst } => {
                RateLimitConfig {
                    limit: *limit,
                    window: *window,
                    burst_capacity: *burst,
                    ..Default::default()
                }
            }
            EndpointPolicy::Public { limit, window } => {
                RateLimitConfig {
                    limit: *limit,
                    window: *window,
                    burst_capacity: Some(*limit / 2),
                    ..Default::default()
                }
            }
            EndpointPolicy::Admin { limit, window, .. } => {
                RateLimitConfig {
                    limit: *limit,
                    window: *window,
                    burst_capacity: Some(1), // Very restrictive
                    adaptive: true,
                    ..Default::default()
                }
            }
        }
    }

    /// Check if this policy requires authentication
    pub fn requires_auth(&self) -> bool {
        matches!(self, EndpointPolicy::Admin { require_auth: true, .. })
    }
}

/// Builder for creating rate limiting policies
pub struct PolicyBuilder {
    endpoint_policies: std::collections::HashMap<String, EndpointPolicy>,
}

impl PolicyBuilder {
    /// Create a new policy builder
    pub fn new() -> Self {
        Self {
            endpoint_policies: std::collections::HashMap::new(),
        }
    }

    /// Add an authentication endpoint policy
    pub fn auth_endpoint(
        mut self,
        path: impl Into<String>,
        normal_limit: u64,
        failure_limit: u64,
        window: Duration,
    ) -> Self {
        self.endpoint_policies.insert(
            path.into(),
            EndpointPolicy::Authentication {
                normal_limit,
                failure_limit,
                window,
            },
        );
        self
    }

    /// Add an API endpoint policy
    pub fn api_endpoint(
        mut self,
        path: impl Into<String>,
        limit: u64,
        window: Duration,
        burst: Option<u64>,
    ) -> Self {
        self.endpoint_policies.insert(
            path.into(),
            EndpointPolicy::Api {
                limit,
                window,
                burst,
            },
        );
        self
    }

    /// Add a public endpoint policy
    pub fn public_endpoint(
        mut self,
        path: impl Into<String>,
        limit: u64,
        window: Duration,
    ) -> Self {
        self.endpoint_policies.insert(
            path.into(),
            EndpointPolicy::Public {
                limit,
                window,
            },
        );
        self
    }

    /// Add an admin endpoint policy
    pub fn admin_endpoint(
        mut self,
        path: impl Into<String>,
        limit: u64,
        window: Duration,
        require_auth: bool,
    ) -> Self {
        self.endpoint_policies.insert(
            path.into(),
            EndpointPolicy::Admin {
                limit,
                window,
                require_auth,
            },
        );
        self
    }

    /// Build the middleware configuration
    pub fn build(self) -> MiddlewareConfig {
        let mut config = MiddlewareConfig::default();
        
        for (path, policy) in self.endpoint_policies {
            config.endpoint_configs.insert(path, policy.to_config());
        }
        
        config
    }
}

impl Default for PolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algorithms::TokenBucketLimiter,
        storage::MemoryRateLimitStorage,
        RateLimitConfig,
    };
    use std::net::IpAddr;

    #[tokio::test]
    async fn test_middleware_context_extraction() {
        let storage = Arc::new(MemoryRateLimitStorage::new());
        let config = RateLimitConfig::default();
        let limiter = Arc::new(TokenBucketLimiter::new(storage, config).unwrap());
        
        let middleware = AuthRateLimitMiddleware::with_defaults(limiter);
        
        let context = middleware.extract_context(
            Some("192.168.1.1".parse::<IpAddr>().unwrap()),
            Some("user123".to_string()),
            "/api/login",
            Some("Mozilla/5.0".to_string()),
            false,
        );
        
        assert_eq!(context.ip_address, Some("192.168.1.1".parse().unwrap()));
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.endpoint, Some("/api/login".to_string()));
        assert!(!context.is_auth_failure);
    }

    #[tokio::test]
    async fn test_skip_paths() {
        let storage = Arc::new(MemoryRateLimitStorage::new());
        let config = RateLimitConfig::default();
        let limiter = Arc::new(TokenBucketLimiter::new(storage, config).unwrap());
        
        let middleware = AuthRateLimitMiddleware::with_defaults(limiter);
        
        assert!(middleware.should_skip_path("/health"));
        assert!(middleware.should_skip_path("/metrics"));
        assert!(!middleware.should_skip_path("/api/login"));
    }

    #[tokio::test]
    async fn test_header_generation() {
        let storage = Arc::new(MemoryRateLimitStorage::new());
        let config = RateLimitConfig::default();
        let limiter = Arc::new(TokenBucketLimiter::new(storage, config).unwrap());
        
        let middleware = AuthRateLimitMiddleware::with_defaults(limiter);
        
        let result = RateLimitResult::Allowed {
            remaining: 50,
            reset_after: Duration::minutes(1),
            limit: 100,
        };
        
        let headers = middleware.generate_headers(&result);
        
        assert!(!headers.is_empty());
        assert!(headers.iter().any(|(k, v)| k == "X-RateLimit-Limit" && v == "100"));
        assert!(headers.iter().any(|(k, v)| k == "X-RateLimit-Remaining" && v == "50"));
    }

    #[tokio::test]
    async fn test_policy_builder() {
        let config = PolicyBuilder::new()
            .auth_endpoint("/api/login", 10, 5, Duration::minutes(1))
            .api_endpoint("/api/data", 100, Duration::minutes(1), Some(10))
            .public_endpoint("/public", 1000, Duration::minutes(1))
            .admin_endpoint("/admin", 5, Duration::minutes(1), true)
            .build();
        
        assert_eq!(config.endpoint_configs.len(), 4);
        assert!(config.endpoint_configs.contains_key("/api/login"));
        assert!(config.endpoint_configs.contains_key("/api/data"));
        assert!(config.endpoint_configs.contains_key("/public"));
        assert!(config.endpoint_configs.contains_key("/admin"));
    }

    #[test]
    fn test_rate_limit_response() {
        let response = RateLimitResponse::too_many_requests(
            Duration::seconds(30),
            100,
            vec![("X-RateLimit-Limit".to_string(), "100".to_string())],
        );
        
        assert_eq!(response.status, 429);
        assert!(!response.body.is_empty());
        assert_eq!(response.headers.len(), 1);
    }
} 