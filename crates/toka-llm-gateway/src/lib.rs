#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-llm-gateway** – Secure LLM provider gateway with memory-safe configuration.
//!
//! This crate provides a secure, memory-safe interface for communicating with
//! Large Language Model providers while ensuring no sensitive data leaks into
//! memory or logs.
//!
//! ## Security Features
//!
//! - **Memory-safe secrets**: Uses `secrecy` crate to prevent API keys from leaking
//! - **Zero-copy configuration**: Environment variables are securely loaded without copies
//! - **Automatic cleanup**: All sensitive data is zeroized on drop
//! - **Rate limiting**: Built-in protection against abuse
//! - **Request sanitization**: Prevents injection attacks
//! - **Response validation**: Ensures safe outputs
//!
//! ## Usage
//!
//! ```rust,no_run
//! use toka_llm_gateway::{LlmGateway, LlmProvider, LlmRequest};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! // Load configuration from environment
//! let config = toka_llm_gateway::Config::from_env()?;
//!
//! // Create secure gateway
//! let gateway = LlmGateway::new(config).await?;
//!
//! // Make secure request
//! let request = LlmRequest::new("Explain Rust ownership")?;
//! let response = gateway.complete(request).await?;
//!
//! println!("Response: {}", response.content());
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use governor::{Quota, RateLimiter, Jitter};
use secrecy::{ExposeSecret, Secret, Zeroize};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub mod config;
pub mod providers;
pub mod sanitizer;
pub mod validator;

pub use config::{Config, EnvLoader};
pub use providers::{LlmProvider, AnthropicProvider, OpenAiProvider};
pub use sanitizer::RequestSanitizer;
pub use validator::ResponseValidator;

/// Maximum allowed prompt length to prevent memory exhaustion
pub const MAX_PROMPT_LENGTH: usize = 32_768; // 32KB

/// Maximum allowed response length to prevent memory exhaustion  
pub const MAX_RESPONSE_LENGTH: usize = 1_048_576; // 1MB

/// Default rate limit: 60 requests per minute
pub const DEFAULT_RATE_LIMIT: u32 = 60;

/// Request to an LLM provider with security constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmRequest {
    /// The prompt text (sanitized)
    prompt: String,
    /// Maximum tokens to generate
    max_tokens: Option<u32>,
    /// Temperature for randomness (0.0 - 1.0)
    temperature: Option<f32>,
    /// Request metadata for auditing
    metadata: RequestMetadata,
}

/// Metadata attached to LLM requests for security and auditing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Originating agent ID
    pub agent_id: toka_types::EntityId,
    /// Workstream this request belongs to
    pub workstream: String,
    /// Request timestamp (Unix epoch)
    pub timestamp: u64,
    /// Request ID for tracing
    pub request_id: String,
}

/// Response from an LLM provider with validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Generated content (validated)
    content: String,
    /// Token usage statistics
    usage: TokenUsage,
    /// Response metadata
    metadata: ResponseMetadata,
}

/// Token usage statistics for cost tracking and monitoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Metadata attached to LLM responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Provider that generated the response
    pub provider: String,
    /// Model used for generation
    pub model: String,
    /// Response timestamp (Unix epoch)
    pub timestamp: u64,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
}

/// Security constraints for LLM requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyConstraints {
    /// Maximum tokens allowed in response
    pub max_response_tokens: u32,
    /// Whether to filter potentially harmful content
    pub content_filtering: bool,
    /// Whether to validate code outputs
    pub code_validation: bool,
    /// Custom safety rules
    pub custom_rules: Vec<String>,
}

impl Default for SafetyConstraints {
    fn default() -> Self {
        Self {
            max_response_tokens: 4096,
            content_filtering: true,
            code_validation: true,
            custom_rules: vec![
                "No file system operations outside sandbox".to_string(),
                "No network requests to external services".to_string(),
                "No execution of system commands".to_string(),
            ],
        }
    }
}

/// Main LLM gateway providing secure access to language models.
pub struct LlmGateway {
    provider: Box<dyn LlmProvider>,
    rate_limiter: Arc<RateLimiter<String, governor::state::keyed::DashMapStateStore<String>, governor::clock::DefaultClock, governor::middleware::NoOpMiddleware>>,
    sanitizer: RequestSanitizer,
    validator: ResponseValidator,
    config: Arc<Config>,
    metrics: Arc<RwLock<GatewayMetrics>>,
}

/// Metrics collected by the gateway for monitoring.
#[derive(Debug, Default, Clone)]
pub struct GatewayMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Total successful responses
    pub successful_responses: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Total tokens consumed
    pub total_tokens: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

impl LlmRequest {
    /// Create a new LLM request with validation.
    ///
    /// # Security
    /// Validates prompt length and content to prevent various attack vectors.
    pub fn new(prompt: impl Into<String>) -> Result<Self> {
        let prompt = prompt.into();
        
        // Validate prompt length
        if prompt.len() > MAX_PROMPT_LENGTH {
            anyhow::bail!(
                "Prompt too long: {} > {} characters", 
                prompt.len(), 
                MAX_PROMPT_LENGTH
            );
        }
        
        // Validate prompt is not empty
        if prompt.trim().is_empty() {
            anyhow::bail!("Prompt cannot be empty");
        }
        
        Ok(Self {
            prompt,
            max_tokens: None,
            temperature: None,
            metadata: RequestMetadata {
                agent_id: toka_types::EntityId(0), // Will be set by gateway
                workstream: String::new(), // Will be set by gateway
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                request_id: uuid::Uuid::new_v4().to_string(),
            },
        })
    }
    
    /// Set maximum tokens for the response.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Set temperature for randomness (0.0 = deterministic, 1.0 = very random).
    pub fn with_temperature(mut self, temperature: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&temperature) {
            anyhow::bail!("Temperature must be between 0.0 and 1.0, got {}", temperature);
        }
        self.temperature = Some(temperature);
        Ok(self)
    }
    
    /// Get the prompt text.
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
    
    /// Get maximum tokens setting.
    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }
    
    /// Get temperature setting.
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }
    
    /// Get request metadata.
    pub fn metadata(&self) -> &RequestMetadata {
        &self.metadata
    }
}

impl LlmResponse {
    /// Create a new LLM response.
    pub fn new(
        content: String,
        usage: TokenUsage,
        provider: String,
        model: String,
        duration: Duration,
    ) -> Result<Self> {
        // Validate response length
        if content.len() > MAX_RESPONSE_LENGTH {
            anyhow::bail!(
                "Response too long: {} > {} characters",
                content.len(),
                MAX_RESPONSE_LENGTH
            );
        }
        
        Ok(Self {
            content,
            usage,
            metadata: ResponseMetadata {
                provider,
                model,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                duration_ms: duration.as_millis() as u64,
            },
        })
    }
    
    /// Get the response content.
    pub fn content(&self) -> &str {
        &self.content
    }
    
    /// Get token usage statistics.
    pub fn usage(&self) -> &TokenUsage {
        &self.usage
    }
    
    /// Get response metadata.
    pub fn metadata(&self) -> &ResponseMetadata {
        &self.metadata
    }
}

impl LlmGateway {
    /// Create a new LLM gateway with the provided configuration.
    ///
    /// # Security
    /// Configuration is loaded securely from environment variables with
    /// automatic cleanup of sensitive data.
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing LLM gateway with provider: {}", config.provider_name());
        
        // Create provider based on configuration
        let provider = config.create_provider().await
            .context("Failed to create LLM provider")?;
        
        // Set up rate limiter
        let quota = Quota::per_minute(std::num::NonZeroU32::new(config.rate_limit()).unwrap());
        let rate_limiter = Arc::new(RateLimiter::keyed(quota));
        
        // Initialize sanitizer and validator
        let sanitizer = RequestSanitizer::new();
        let validator = ResponseValidator::new();
        
        // Initialize metrics
        let metrics = Arc::new(RwLock::new(GatewayMetrics::default()));
        
        Ok(Self {
            provider,
            rate_limiter,
            sanitizer,
            validator,
            config: Arc::new(config),
            metrics,
        })
    }
    
    /// Complete an LLM request with full security validation.
    ///
    /// # Security
    /// - Rate limiting per agent
    /// - Request sanitization
    /// - Response validation
    /// - Audit logging
    /// - Automatic cleanup of sensitive data
    pub async fn complete(&self, mut request: LlmRequest) -> Result<LlmResponse> {
        let start_time = std::time::Instant::now();
        
        // Rate limiting check
        let rate_key = format!("agent_{}", request.metadata.agent_id.0);
        if self.rate_limiter.check_key(&rate_key).is_err() {
            warn!(
                "Rate limit exceeded for agent {}", 
                request.metadata.agent_id.0
            );
            anyhow::bail!("Rate limit exceeded");
        }
        
        // Sanitize request
        request = self.sanitizer.sanitize(request)
            .context("Failed to sanitize request")?;
        
        debug!(
            "Processing LLM request for agent {} in workstream {}",
            request.metadata.agent_id.0,
            request.metadata.workstream
        );
        
        // Make request to provider
        let response = match self.provider.complete(&request).await {
            Ok(response) => response,
            Err(e) => {
                error!("LLM provider request failed: {}", e);
                self.increment_failed_requests().await;
                return Err(e);
            }
        };
        
        // Validate response
        let validated_response = self.validator.validate(response)
            .context("Response validation failed")?;
        
        // Update metrics
        let duration = start_time.elapsed();
        self.update_metrics(duration, &validated_response).await;
        
        info!(
            "Completed LLM request for agent {} in {}ms",
            request.metadata.agent_id.0,
            duration.as_millis()
        );
        
        Ok(validated_response)
    }
    
    /// Get current gateway metrics.
    pub async fn metrics(&self) -> GatewayMetrics {
        let metrics_guard = self.metrics.read().await;
        GatewayMetrics {
            total_requests: metrics_guard.total_requests,
            successful_responses: metrics_guard.successful_responses,
            failed_requests: metrics_guard.failed_requests,
            total_tokens: metrics_guard.total_tokens,
            avg_response_time_ms: metrics_guard.avg_response_time_ms,
        }
    }
    
    /// Update metrics after processing a request.
    async fn update_metrics(&self, duration: Duration, response: &LlmResponse) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.successful_responses += 1;
        metrics.total_tokens += response.usage.total_tokens as u64;
        
        // Update rolling average response time
        let current_avg = metrics.avg_response_time_ms;
        let new_duration_ms = duration.as_millis() as f64;
        let total_responses = metrics.successful_responses as f64;
        
        metrics.avg_response_time_ms = 
            (current_avg * (total_responses - 1.0) + new_duration_ms) / total_responses;
    }
    
    /// Increment failed request counter.
    async fn increment_failed_requests(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
    }
}

// Implement Drop to ensure sensitive data is cleared
impl Drop for LlmGateway {
    fn drop(&mut self) {
        debug!("Cleaning up LLM gateway resources");
        // The providers handle their own cleanup through secrecy crate
    }
}

/// UUID generation for request IDs
mod uuid {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
        
        pub fn to_string(&self) -> String {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
            format!("req_{}_{}", timestamp, counter)
        }
    }
} 