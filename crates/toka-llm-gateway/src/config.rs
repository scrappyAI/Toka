//! Secure configuration loading with memory-safe environment variable handling.
//!
//! This module provides secure configuration loading from environment variables
//! with automatic cleanup of sensitive data to prevent memory leaks and exposure.

use std::collections::HashMap;
use std::env;

use anyhow::{Context, Result};
use secrecy::{ExposeSecret, Secret, Zeroize};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::providers::{LlmProvider, AnthropicProvider, OpenAiProvider};
use crate::DEFAULT_RATE_LIMIT;

/// Configuration for the LLM gateway with secure secret handling.
#[derive(Debug, Clone)]
pub struct Config {
    /// LLM provider type
    provider: ProviderConfig,
    /// Rate limiting configuration
    rate_limit: u32,
    /// Request timeout in seconds
    timeout_seconds: u64,
    /// Whether to enable debug logging (careful with secrets!)
    debug_mode: bool,
    /// Additional provider-specific settings
    additional_settings: HashMap<String, String>,
}

/// Provider-specific configuration with secure secret storage.
#[derive(Debug, Clone)]
pub enum ProviderConfig {
    /// Anthropic Claude configuration
    Anthropic {
        /// API key (securely stored)
        api_key: Secret<String>,
        /// Model to use (e.g., "claude-3-5-sonnet-20241022")
        model: String,
        /// API base URL (for custom endpoints)
        base_url: Option<String>,
    },
    /// OpenAI GPT configuration
    OpenAi {
        /// API key (securely stored)
        api_key: Secret<String>,
        /// Model to use (e.g., "gpt-4")
        model: String,
        /// Organization ID (optional)
        organization: Option<String>,
        /// API base URL (for custom endpoints)
        base_url: Option<String>,
    },
    /// Local model configuration
    Local {
        /// Local server endpoint
        endpoint: String,
        /// Model identifier
        model: String,
        /// Authentication token (if required)
        auth_token: Option<Secret<String>>,
    },
}

/// Secure environment variable loader with automatic cleanup.
pub struct EnvLoader {
    /// Cached environment variables (will be zeroized on drop)
    env_cache: HashMap<String, Secret<String>>,
}

impl EnvLoader {
    /// Create a new environment loader.
    ///
    /// # Security
    /// This function loads all relevant environment variables into a secure
    /// cache that will be automatically zeroized when dropped.
    pub fn new() -> Result<Self> {
        let mut env_cache = HashMap::new();
        
        // List of environment variables to securely cache
        let env_vars = [
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY", 
            "LLM_PROVIDER",
            "LLM_MODEL",
            "LLM_BASE_URL",
            "LLM_RATE_LIMIT",
            "LLM_TIMEOUT",
            "LLM_DEBUG",
            "OPENAI_ORGANIZATION",
            "LOCAL_LLM_ENDPOINT",
            "LOCAL_LLM_AUTH_TOKEN",
        ];
        
        // Securely load environment variables
        for var_name in &env_vars {
            if let Ok(value) = env::var(var_name) {
                // Store in secure cache with automatic cleanup
                env_cache.insert(
                    var_name.to_string(),
                    Secret::new(value)
                );
            }
        }
        
        debug!("Loaded {} environment variables into secure cache", env_cache.len());
        
        Ok(Self { env_cache })
    }
    
    /// Get a required environment variable securely.
    ///
    /// # Security
    /// Returns a reference to the securely stored value without copying.
    pub fn get_required(&self, key: &str) -> Result<&Secret<String>> {
        self.env_cache.get(key)
            .with_context(|| format!("Required environment variable {} not found", key))
    }
    
    /// Get an optional environment variable securely.
    pub fn get_optional(&self, key: &str) -> Option<&Secret<String>> {
        self.env_cache.get(key)
    }
    
    /// Get a non-sensitive configuration value.
    pub fn get_public(&self, key: &str) -> Option<String> {
        self.env_cache.get(key)
            .map(|secret| secret.expose_secret().clone())
    }
    
    /// Parse a numeric environment variable with a default.
    pub fn get_numeric<T>(&self, key: &str, default: T) -> T 
    where
        T: std::str::FromStr + Copy,
        T::Err: std::fmt::Display,
    {
        self.get_public(key)
            .and_then(|value| {
                value.parse().map_err(|e| {
                    warn!("Failed to parse {} as numeric: {}", key, e);
                    e
                }).ok()
            })
            .unwrap_or(default)
    }
    
    /// Parse a boolean environment variable with a default.
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get_public(key)
            .map(|value| {
                matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
            })
            .unwrap_or(default)
    }
}

impl Drop for EnvLoader {
    fn drop(&mut self) {
        // Securely clear all cached environment variables
        for (key, mut secret) in self.env_cache.drain() {
            debug!("Clearing cached environment variable: {}", key);
            // The Secret<String> will automatically zeroize when dropped
            drop(secret);
        }
        debug!("Environment variable cache cleared");
    }
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// ## Required (provider-specific):
    /// - `ANTHROPIC_API_KEY` - Anthropic API key (if using Anthropic)
    /// - `OPENAI_API_KEY` - OpenAI API key (if using OpenAI)
    /// - `LOCAL_LLM_ENDPOINT` - Local LLM endpoint (if using local)
    ///
    /// ## Optional:
    /// - `LLM_PROVIDER` - Provider type: "anthropic", "openai", "local" (default: auto-detect)
    /// - `LLM_MODEL` - Model to use (provider-specific defaults)
    /// - `LLM_BASE_URL` - Custom API base URL
    /// - `LLM_RATE_LIMIT` - Requests per minute (default: 60)
    /// - `LLM_TIMEOUT` - Request timeout in seconds (default: 30)
    /// - `LLM_DEBUG` - Enable debug mode: "true"/"false" (default: false)
    /// - `OPENAI_ORGANIZATION` - OpenAI organization ID
    /// - `LOCAL_LLM_AUTH_TOKEN` - Local LLM authentication token
    ///
    /// # Security
    /// All API keys and sensitive data are stored using the `secrecy` crate
    /// and will be automatically zeroized when the configuration is dropped.
    pub fn from_env() -> Result<Self> {
        let env_loader = EnvLoader::new()
            .context("Failed to create secure environment loader")?;
        
        Self::from_env_loader(env_loader)
    }
    
    /// Load configuration from an environment loader.
    ///
    /// # Security
    /// This method allows for dependency injection of the environment loader
    /// for testing while maintaining secure environment variable handling.
    pub fn from_env_loader(env_loader: EnvLoader) -> Result<Self> {
        // Determine provider type
        let provider_type = env_loader.get_public("LLM_PROVIDER")
            .unwrap_or_else(|| Self::auto_detect_provider(&env_loader));
        
        debug!("Using LLM provider: {}", provider_type);
        
        // Load provider-specific configuration
        let provider = match provider_type.to_lowercase().as_str() {
            "anthropic" => {
                let api_key = env_loader.get_required("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY required for Anthropic provider")?
                    .clone();
                
                let model = env_loader.get_public("LLM_MODEL")
                    .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
                
                let base_url = env_loader.get_public("LLM_BASE_URL");
                
                ProviderConfig::Anthropic {
                    api_key,
                    model,
                    base_url,
                }
            }
            "openai" => {
                let api_key = env_loader.get_required("OPENAI_API_KEY")
                    .context("OPENAI_API_KEY required for OpenAI provider")?
                    .clone();
                
                let model = env_loader.get_public("LLM_MODEL")
                    .unwrap_or_else(|| "gpt-4".to_string());
                
                let organization = env_loader.get_public("OPENAI_ORGANIZATION");
                let base_url = env_loader.get_public("LLM_BASE_URL");
                
                ProviderConfig::OpenAi {
                    api_key,
                    model,
                    organization,
                    base_url,
                }
            }
            "local" => {
                let endpoint = env_loader.get_public("LOCAL_LLM_ENDPOINT")
                    .context("LOCAL_LLM_ENDPOINT required for local provider")?;
                
                let model = env_loader.get_public("LLM_MODEL")
                    .unwrap_or_else(|| "local-model".to_string());
                
                let auth_token = env_loader.get_optional("LOCAL_LLM_AUTH_TOKEN")
                    .cloned();
                
                ProviderConfig::Local {
                    endpoint,
                    model,
                    auth_token,
                }
            }
            unknown => {
                anyhow::bail!("Unknown LLM provider: {}. Supported: anthropic, openai, local", unknown);
            }
        };
        
        // Load general configuration
        let rate_limit = env_loader.get_numeric("LLM_RATE_LIMIT", DEFAULT_RATE_LIMIT);
        let timeout_seconds = env_loader.get_numeric("LLM_TIMEOUT", 30u64);
        let debug_mode = env_loader.get_bool("LLM_DEBUG", false);
        
        if debug_mode {
            warn!("Debug mode enabled - be careful with sensitive data in logs!");
        }
        
        Ok(Self {
            provider,
            rate_limit,
            timeout_seconds,
            debug_mode,
            additional_settings: HashMap::new(),
        })
    }
    
    /// Auto-detect provider based on available API keys.
    fn auto_detect_provider(env_loader: &EnvLoader) -> String {
        if env_loader.get_optional("ANTHROPIC_API_KEY").is_some() {
            "anthropic".to_string()
        } else if env_loader.get_optional("OPENAI_API_KEY").is_some() {
            "openai".to_string()
        } else if env_loader.get_optional("LOCAL_LLM_ENDPOINT").is_some() {
            "local".to_string()
        } else {
            "anthropic".to_string() // Default fallback
        }
    }
    
    /// Create a provider instance from this configuration.
    ///
    /// # Security
    /// The provider will have access to the API keys but they remain
    /// securely stored and will be zeroized when dropped.
    pub async fn create_provider(&self) -> Result<Box<dyn LlmProvider>> {
        match &self.provider {
            ProviderConfig::Anthropic { api_key, model, base_url } => {
                let provider = AnthropicProvider::new(
                    api_key.clone(),
                    model.clone(),
                    base_url.clone(),
                    self.timeout_seconds,
                ).await?;
                Ok(Box::new(provider))
            }
            ProviderConfig::OpenAi { api_key, model, organization, base_url } => {
                let provider = OpenAiProvider::new(
                    api_key.clone(),
                    model.clone(),
                    organization.clone(),
                    base_url.clone(),
                    self.timeout_seconds,
                ).await?;
                Ok(Box::new(provider))
            }
            ProviderConfig::Local { endpoint, model, auth_token } => {
                // Local provider implementation would go here
                anyhow::bail!("Local LLM provider not yet implemented");
            }
        }
    }
    
    /// Get the provider name for logging and metrics.
    pub fn provider_name(&self) -> &'static str {
        match &self.provider {
            ProviderConfig::Anthropic { .. } => "anthropic",
            ProviderConfig::OpenAi { .. } => "openai", 
            ProviderConfig::Local { .. } => "local",
        }
    }
    
    /// Get the configured rate limit.
    pub fn rate_limit(&self) -> u32 {
        self.rate_limit
    }
    
    /// Get the configured timeout.
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }
    
    /// Check if debug mode is enabled.
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }
    
    /// Get the model name being used.
    pub fn model_name(&self) -> &str {
        match &self.provider {
            ProviderConfig::Anthropic { model, .. } => model,
            ProviderConfig::OpenAi { model, .. } => model,
            ProviderConfig::Local { model, .. } => model,
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        debug!("Cleaning up LLM gateway configuration");
        // Provider configs contain Secret<String> which will auto-zeroize
        // Additional cleanup can be added here if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_env_loader_security() {
        // Set a test environment variable
        env::set_var("TEST_SECRET", "sensitive_value");
        
        {
            let loader = EnvLoader::new().unwrap();
            // Verify we can access the value
            assert!(loader.get_optional("TEST_SECRET").is_some());
        } // EnvLoader dropped here, should clear cache
        
        // Clean up
        env::remove_var("TEST_SECRET");
    }
    
    #[test]
    fn test_config_auto_detection() {
        // Clear any existing provider env vars
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("LOCAL_LLM_ENDPOINT");
        
        // Test Anthropic detection
        env::set_var("ANTHROPIC_API_KEY", "test_key");
        let loader = EnvLoader::new().unwrap();
        assert_eq!(Config::auto_detect_provider(&loader), "anthropic");
        env::remove_var("ANTHROPIC_API_KEY");
        
        // Test OpenAI detection
        env::set_var("OPENAI_API_KEY", "test_key");
        let loader = EnvLoader::new().unwrap();
        assert_eq!(Config::auto_detect_provider(&loader), "openai");
        env::remove_var("OPENAI_API_KEY");
        
        // Test local detection
        env::set_var("LOCAL_LLM_ENDPOINT", "http://localhost:8080");
        let loader = EnvLoader::new().unwrap();
        assert_eq!(Config::auto_detect_provider(&loader), "local");
        env::remove_var("LOCAL_LLM_ENDPOINT");
    }
} 