//! Unified error handling for Toka Agent OS
//!
//! This module provides a single error type that consolidates all error handling
//! across the Toka ecosystem, replacing the fragmented error types in individual crates.

use thiserror::Error;

/// Unified error type for the entire Toka system
///
/// This replaces all the individual error types across different crates
/// with a single, comprehensive error type that provides structured error
/// handling and context propagation.
#[derive(Debug, Error)]
pub enum TokaError {
    /// Storage-related errors
    #[error("Storage error: {message}")]
    Storage {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Tool-related errors
    #[error("Tool error: {message}")]
    Tool {
        message: String,
        tool_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// LLM-related errors
    #[error("LLM error: {message}")]
    Llm {
        message: String,
        provider: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Agent-related errors
    #[error("Agent error: {message}")]
    Agent {
        message: String,
        agent_id: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        key: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Security and capability errors
    #[error("Security error: {message}")]
    Security {
        message: String,
        context: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Runtime and execution errors
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        context: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network and communication errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Serialization and deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        format: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        value: Option<String>,
    },

    /// I/O errors
    #[error("I/O error: {message}")]
    Io {
        message: String,
        path: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal system errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        context: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Convenient type alias for Result with TokaError
pub type TokaResult<T> = Result<T, TokaError>;

impl TokaError {
    /// Create a storage error with a message
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
            source: None,
        }
    }

    /// Create a storage error with a message and source
    pub fn storage_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Storage {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a tool error with a message
    pub fn tool(message: impl Into<String>) -> Self {
        Self::Tool {
            message: message.into(),
            tool_name: None,
            source: None,
        }
    }

    /// Create a tool error with name and message
    pub fn tool_with_name(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Tool {
            message: message.into(),
            tool_name: Some(tool_name.into()),
            source: None,
        }
    }

    /// Create an LLM error with a message
    pub fn llm(message: impl Into<String>) -> Self {
        Self::Llm {
            message: message.into(),
            provider: None,
            source: None,
        }
    }

    /// Create an LLM error with provider and message
    pub fn llm_with_provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Llm {
            message: message.into(),
            provider: Some(provider.into()),
            source: None,
        }
    }

    /// Create an agent error with a message
    pub fn agent(message: impl Into<String>) -> Self {
        Self::Agent {
            message: message.into(),
            agent_id: None,
            source: None,
        }
    }

    /// Create an agent error with ID and message
    pub fn agent_with_id(agent_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Agent {
            message: message.into(),
            agent_id: Some(agent_id.into()),
            source: None,
        }
    }

    /// Create a configuration error with a message
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: None,
            source: None,
        }
    }

    /// Create a configuration error with key and message
    pub fn config_with_key(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: Some(key.into()),
            source: None,
        }
    }

    /// Create a security error with a message
    pub fn security(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Create a runtime error with a message
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Create a network error with a message
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: None,
            source: None,
        }
    }

    /// Create a validation error with a message
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            value: None,
        }
    }

    /// Create a validation error with field and message
    pub fn validation_with_field(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.into()),
            value: None,
        }
    }

    /// Create an I/O error with a message
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            path: None,
            source: None,
        }
    }

    /// Create an I/O error with path and message
    pub fn io_with_path(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            path: Some(path.into()),
            source: None,
        }
    }

    /// Create an internal error with a message
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Add context to any error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        match &mut self {
            Self::Storage { .. } => {} // Storage errors don't have context field
            Self::Tool { .. } => {} // Tool errors don't have context field
            Self::Llm { .. } => {} // LLM errors don't have context field
            Self::Agent { .. } => {} // Agent errors don't have context field
            Self::Config { .. } => {} // Config errors don't have context field
            Self::Security { context: ctx, .. } 
            | Self::Runtime { context: ctx, .. } 
            | Self::Internal { context: ctx, .. } => {
                *ctx = Some(context.into());
            }
            Self::Network { .. } => {} // Network errors use endpoint field
            Self::Serialization { .. } => {} // Serialization errors use format field
            Self::Validation { .. } => {} // Validation errors use field field
            Self::Io { .. } => {} // I/O errors use path field
        }
        self
    }
}

// Implement conversions from common error types
impl From<std::io::Error> for TokaError {
    fn from(err: std::io::Error) -> Self {
        Self::io_with_source("I/O operation failed", err)
    }
}

impl From<serde_json::Error> for TokaError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: "JSON serialization failed".to_string(),
            format: Some("json".to_string()),
            source: Some(Box::new(err)),
        }
    }
}

impl From<uuid::Error> for TokaError {
    fn from(err: uuid::Error) -> Self {
        Self::validation_with_source("Invalid UUID format", err)
    }
}

// Helper methods for common patterns
impl TokaError {
    fn io_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Io {
            message: message.into(),
            path: None,
            source: Some(Box::new(source)),
        }
    }

    fn validation_with_source(
        message: impl Into<String>,
        _source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            value: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = TokaError::storage("Test storage error");
        assert!(matches!(err, TokaError::Storage { .. }));
        assert_eq!(err.to_string(), "Storage error: Test storage error");
    }

    #[test]
    fn test_tool_error_with_name() {
        let err = TokaError::tool_with_name("test_tool", "Tool failed");
        match err {
            TokaError::Tool { message, tool_name, .. } => {
                assert_eq!(message, "Tool failed");
                assert_eq!(tool_name, Some("test_tool".to_string()));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[test]
    fn test_result_type() {
        let success: TokaResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        let error: TokaResult<i32> = Err(TokaError::validation("Invalid input"));
        assert!(error.is_err());
    }
}