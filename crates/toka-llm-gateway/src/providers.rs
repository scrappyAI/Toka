//! LLM provider implementations with secure API key handling.
//!
//! This module provides secure implementations for various LLM providers
//! with automatic cleanup of sensitive data and comprehensive error handling.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use secrecy::{ExposeSecret, Secret, Zeroize};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use url::Url;

use crate::{LlmRequest, LlmResponse, TokenUsage};

/// Trait for LLM providers with secure operations.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Complete a text generation request.
    ///
    /// # Security
    /// Implementations must ensure API keys are handled securely and
    /// responses are validated before returning.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse>;
    
    /// Get the provider name for logging and metrics.
    fn provider_name(&self) -> &'static str;
    
    /// Get the model being used.
    fn model_name(&self) -> &str;
    
    /// Get maximum token limit for this provider/model.
    fn max_tokens(&self) -> u32;
    
    /// Health check for the provider.
    async fn health_check(&self) -> Result<()>;
}

/// Anthropic Claude provider with secure API key handling.
pub struct AnthropicProvider {
    client: Client,
    api_key: Secret<String>,
    model: String,
    base_url: Url,
    max_tokens: u32,
}

/// OpenAI GPT provider with secure API key handling.
pub struct OpenAiProvider {
    client: Client,
    api_key: Secret<String>,
    model: String,
    organization: Option<String>,
    base_url: Url,
    max_tokens: u32,
}

// Anthropic API types
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// OpenAI API types
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with secure configuration.
    ///
    /// # Security
    /// API key is stored securely and will be zeroized when dropped.
    pub async fn new(
        api_key: Secret<String>,
        model: String,
        base_url: Option<String>,
        timeout_seconds: u64,
    ) -> Result<Self> {
        // Validate API key format (should start with 'sk-ant-')
        if !api_key.expose_secret().starts_with("sk-ant-") {
            anyhow::bail!("Invalid Anthropic API key format");
        }
        
        let base_url = base_url
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        let base_url = Url::parse(&base_url)
            .context("Invalid Anthropic base URL")?;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent("toka-llm-gateway/0.3.0")
            .build()
            .context("Failed to create HTTP client")?;
        
        // Set max tokens based on model
        let max_tokens = match model.as_str() {
            "claude-3-5-sonnet-20241022" => 8192,
            "claude-3-haiku-20240307" => 4096,
            "claude-3-opus-20240229" => 4096,
            _ => 4096, // Conservative default
        };
        
        debug!("Initialized Anthropic provider with model: {}", model);
        
        Ok(Self {
            client,
            api_key,
            model,
            base_url,
            max_tokens,
        })
    }
    
    /// Create authorization headers for Anthropic API.
    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        
        // Add authorization header
        let auth_value = format!("Bearer {}", self.api_key.expose_secret());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .context("Invalid authorization header value")?
        );
        
        // Add required Anthropic headers
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        );
        headers.insert(
            "x-api-version",
            HeaderValue::from_static("2023-06-01")
        );
        
        Ok(headers)
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    #[instrument(skip(self, request), fields(model = %self.model))]
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let start_time = std::time::Instant::now();
        
        debug!("Making Anthropic API request for model: {}", self.model);
        
        // Prepare request
        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens().unwrap_or(self.max_tokens).min(self.max_tokens),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: request.prompt().to_string(),
            }],
            temperature: request.temperature(),
        };
        
        // Create headers
        let headers = self.create_headers()
            .context("Failed to create request headers")?;
        
        // Make API request
        let url = self.base_url.join("/v1/messages")
            .context("Failed to construct API URL")?;
        
        let response = self.client
            .post(url)
            .headers(headers)
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;
        
        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error {}: {}", status, error_text);
            anyhow::bail!("Anthropic API error {}: {}", status, error_text);
        }
        
        // Parse response
        let anthropic_response: AnthropicResponse = response.json().await
            .context("Failed to parse Anthropic API response")?;
        
        // Extract content
        let content = anthropic_response.content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .unwrap_or_default();
        
        if content.is_empty() {
            anyhow::bail!("Empty response from Anthropic API");
        }
        
        // Create token usage
        let usage = TokenUsage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
        };
        
        let duration = start_time.elapsed();
        debug!("Anthropic API request completed in {}ms", duration.as_millis());
        
        LlmResponse::new(
            content,
            usage,
            "anthropic".to_string(),
            anthropic_response.model,
            duration,
        )
    }
    
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn max_tokens(&self) -> u32 {
        self.max_tokens
    }
    
    async fn health_check(&self) -> Result<()> {
        // Simple health check by making a minimal request
        let test_request = LlmRequest::new("Test")?
            .with_max_tokens(1);
        
        match self.complete(&test_request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.context("Anthropic health check failed")),
        }
    }
}

impl OpenAiProvider {
    /// Create a new OpenAI provider with secure configuration.
    ///
    /// # Security
    /// API key is stored securely and will be zeroized when dropped.
    pub async fn new(
        api_key: Secret<String>,
        model: String,
        organization: Option<String>,
        base_url: Option<String>,
        timeout_seconds: u64,
    ) -> Result<Self> {
        // Validate API key format (should start with 'sk-')
        if !api_key.expose_secret().starts_with("sk-") {
            anyhow::bail!("Invalid OpenAI API key format");
        }
        
        let base_url = base_url
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let base_url = Url::parse(&base_url)
            .context("Invalid OpenAI base URL")?;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent("toka-llm-gateway/0.3.0")
            .build()
            .context("Failed to create HTTP client")?;
        
        // Set max tokens based on model
        let max_tokens = match model.as_str() {
            "gpt-4" => 8192,
            "gpt-4-turbo" => 4096,
            "gpt-3.5-turbo" => 4096,
            _ => 4096, // Conservative default
        };
        
        debug!("Initialized OpenAI provider with model: {}", model);
        
        Ok(Self {
            client,
            api_key,
            model,
            organization,
            base_url,
            max_tokens,
        })
    }
    
    /// Create authorization headers for OpenAI API.
    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        
        // Add authorization header
        let auth_value = format!("Bearer {}", self.api_key.expose_secret());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .context("Invalid authorization header value")?
        );
        
        // Add content type
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        );
        
        // Add organization header if provided
        if let Some(org) = &self.organization {
            headers.insert(
                "OpenAI-Organization",
                HeaderValue::from_str(org)
                    .context("Invalid organization header value")?
            );
        }
        
        Ok(headers)
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    #[instrument(skip(self, request), fields(model = %self.model))]
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let start_time = std::time::Instant::now();
        
        debug!("Making OpenAI API request for model: {}", self.model);
        
        // Prepare request
        let openai_request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: request.prompt().to_string(),
            }],
            max_tokens: request.max_tokens().map(|t| t.min(self.max_tokens)),
            temperature: request.temperature(),
        };
        
        // Create headers
        let headers = self.create_headers()
            .context("Failed to create request headers")?;
        
        // Make API request
        let url = self.base_url.join("/v1/chat/completions")
            .context("Failed to construct API URL")?;
        
        let response = self.client
            .post(url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;
        
        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error {}: {}", status, error_text);
            anyhow::bail!("OpenAI API error {}: {}", status, error_text);
        }
        
        // Parse response
        let openai_response: OpenAiResponse = response.json().await
            .context("Failed to parse OpenAI API response")?;
        
        // Extract content
        let content = openai_response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();
        
        if content.is_empty() {
            anyhow::bail!("Empty response from OpenAI API");
        }
        
        // Create token usage
        let usage = TokenUsage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
            total_tokens: openai_response.usage.total_tokens,
        };
        
        let duration = start_time.elapsed();
        debug!("OpenAI API request completed in {}ms", duration.as_millis());
        
        LlmResponse::new(
            content,
            usage,
            "openai".to_string(),
            openai_response.model,
            duration,
        )
    }
    
    fn provider_name(&self) -> &'static str {
        "openai"
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn max_tokens(&self) -> u32 {
        self.max_tokens
    }
    
    async fn health_check(&self) -> Result<()> {
        // Simple health check by making a minimal request
        let test_request = LlmRequest::new("Test")?
            .with_max_tokens(1);
        
        match self.complete(&test_request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.context("OpenAI health check failed")),
        }
    }
}

// Implement Drop for secure cleanup
impl Drop for AnthropicProvider {
    fn drop(&mut self) {
        debug!("Cleaning up Anthropic provider");
        // API key will be automatically zeroized by secrecy crate
    }
}

impl Drop for OpenAiProvider {
    fn drop(&mut self) {
        debug!("Cleaning up OpenAI provider");
        // API key will be automatically zeroized by secrecy crate
    }
} 