//! Secure configuration management for Toka runtime
//!
//! This module handles loading API keys and sensitive credentials from environment
//! variables, ensuring they are never hardcoded in the application source.
//!
//! Security principles:
//! - All credentials loaded from environment variables
//! - Clear separation between sandbox and production
//! - No credentials stored in source code or version control
//! - Fail fast if required credentials are missing

use std::env;

/// Runtime environment configuration
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Sandbox,
    Production,
}

impl Environment {
    /// Parse environment from string (case-insensitive)
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "sandbox" | "test" => Ok(Environment::Sandbox),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(ConfigError::InvalidEnvironment(s.to_string())),
        }
    }

    /// Get environment from TOKA_ENV variable, defaulting to Development
    pub fn from_env() -> Self {
        env::var("TOKA_ENV")
            .map(|s| Self::from_str(&s).unwrap_or(Environment::Development))
            .unwrap_or(Environment::Development)
    }
}

/// Stripe-specific configuration
#[derive(Debug, Clone)]
pub struct StripeConfig {
    /// Stripe API secret key (sk_test_... for sandbox, sk_live_... for production)
    pub api_key: String,
    /// Stripe webhook secret for verifying incoming webhooks
    pub webhook_secret: String,
    /// Whether we're in test mode (true for sandbox/dev, false for production)
    pub test_mode: bool,
    /// Environment this config is for
    pub environment: Environment,
}

impl StripeConfig {
    /// Load Stripe configuration from environment variables
    /// 
    /// Expected environment variables:
    /// - STRIPE_API_KEY: Your Stripe secret key
    /// - STRIPE_WEBHOOK_SECRET: Your webhook endpoint secret
    /// - TOKA_ENV: Environment (development, sandbox, production)
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();
        
        let api_key = env::var("STRIPE_API_KEY")
            .map_err(|_| ConfigError::MissingCredential("STRIPE_API_KEY".to_string()))?;
        
        let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET")
            .map_err(|_| ConfigError::MissingCredential("STRIPE_WEBHOOK_SECRET".to_string()))?;

        // Validate API key format based on environment
        let test_mode = match environment {
            Environment::Production => {
                if !api_key.starts_with("sk_live_") {
                    return Err(ConfigError::InvalidCredential(
                        "Production environment requires live API key (sk_live_...)".to_string()
                    ));
                }
                false
            }
            Environment::Sandbox | Environment::Development => {
                if !api_key.starts_with("sk_test_") {
                    return Err(ConfigError::InvalidCredential(
                        "Sandbox/Development environment requires test API key (sk_test_...)".to_string()
                    ));
                }
                true
            }
        };

        Ok(StripeConfig {
            api_key,
            webhook_secret,
            test_mode,
            environment,
        })
    }

    /// Validate that the configuration is consistent and secure
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check API key length (Stripe keys are typically 107+ characters)
        if self.api_key.len() < 50 {
            return Err(ConfigError::InvalidCredential("API key appears too short".to_string()));
        }

        // Check webhook secret length
        if self.webhook_secret.len() < 20 {
            return Err(ConfigError::InvalidCredential("Webhook secret appears too short".to_string()));
        }

        // Ensure test mode matches environment
        match self.environment {
            Environment::Production if self.test_mode => {
                return Err(ConfigError::InvalidCredential(
                    "Production environment cannot use test mode".to_string()
                ));
            }
            Environment::Sandbox | Environment::Development if !self.test_mode => {
                return Err(ConfigError::InvalidCredential(
                    "Sandbox/Development environment must use test mode".to_string()
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

/// General runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Stripe payment provider configuration
    pub stripe: StripeConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Current environment
    pub environment: Environment,
}

impl RuntimeConfig {
    /// Load complete runtime configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();
        let stripe = StripeConfig::from_env()?;
        let server = ServerConfig::from_env()?;

        // Validate that all configs use the same environment
        if stripe.environment != environment {
            return Err(ConfigError::InvalidCredential(
                "Stripe config environment doesn't match runtime environment".to_string()
            ));
        }

        let config = RuntimeConfig {
            stripe,
            server,
            environment,
        };

        // Validate the entire configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.stripe.validate()?;
        self.server.validate()?;
        Ok(())
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Base URL for webhook endpoints
    pub webhook_base_url: String,
}

impl ServerConfig {
    /// Load server configuration from environment variables
    /// 
    /// Expected environment variables:
    /// - SERVER_HOST: Host to bind to (default: 127.0.0.1)
    /// - SERVER_PORT: Port to bind to (default: 8080)
    /// - WEBHOOK_BASE_URL: Base URL for webhooks (required)
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidCredential("SERVER_PORT must be a valid port number".to_string()))?;

        let webhook_base_url = env::var("WEBHOOK_BASE_URL")
            .map_err(|_| ConfigError::MissingCredential("WEBHOOK_BASE_URL".to_string()))?;

        Ok(ServerConfig {
            host,
            port,
            webhook_base_url,
        })
    }

    /// Validate server configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::InvalidCredential("Port cannot be 0".to_string()));
        }

        if !self.webhook_base_url.starts_with("http") {
            return Err(ConfigError::InvalidCredential(
                "Webhook base URL must start with http:// or https://".to_string()
            ));
        }

        Ok(())
    }
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    /// Required credential is missing from environment
    MissingCredential(String),
    /// Credential format or value is invalid
    InvalidCredential(String),
    /// Environment value is invalid
    InvalidEnvironment(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingCredential(cred) => {
                write!(f, "Missing required credential: {}", cred)
            }
            ConfigError::InvalidCredential(msg) => {
                write!(f, "Invalid credential: {}", msg)
            }
            ConfigError::InvalidEnvironment(env) => {
                write!(f, "Invalid environment: {}", env)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        assert_eq!(Environment::from_str("development").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("SANDBOX").unwrap(), Environment::Sandbox);
        assert_eq!(Environment::from_str("prod").unwrap(), Environment::Production);
        assert!(Environment::from_str("invalid").is_err());
    }
} 