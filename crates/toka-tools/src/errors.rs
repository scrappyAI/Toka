//! Error types for the toka-tools crate
//!
//! This module provides structured error types using thiserror for better
//! error handling and debugging. All errors include rich context and
//! are designed to be easily chainable.

use thiserror::Error;
use std::path::PathBuf;

/// Main error type for the toka-tools crate
/// 
/// This enum covers all possible error conditions that can occur
/// during tool operations, providing rich context for debugging.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::errors::ToolError;
/// 
/// let error = ToolError::ToolNotFound {
///     name: "nonexistent_tool".to_string(),
/// };
/// 
/// assert_eq!(error.to_string(), "Tool 'nonexistent_tool' not found in registry");
/// ```
#[derive(Error, Debug)]
pub enum ToolError {
    /// Tool not found in registry
    #[error("Tool '{name}' not found in registry")]
    ToolNotFound {
        /// Name of the tool that was not found
        name: String,
    },

    /// Tool already registered
    #[error("Tool '{name}' is already registered")]
    ToolAlreadyRegistered {
        /// Name of the tool that was already registered
        name: String,
    },

    /// Parameter validation failed
    #[error("Parameter validation failed for tool '{tool_name}': {reason}")]
    ParameterValidation {
        /// Name of the tool
        tool_name: String,
        /// Reason for validation failure
        reason: String,
    },

    /// Required parameter missing
    #[error("Required parameter '{param_name}' missing for tool '{tool_name}'")]
    MissingParameter {
        /// Name of the tool
        tool_name: String,
        /// Name of the missing parameter
        param_name: String,
    },

    /// Invalid parameter value
    #[error("Invalid value for parameter '{param_name}' in tool '{tool_name}': {reason}")]
    InvalidParameter {
        /// Name of the tool
        tool_name: String,
        /// Name of the parameter
        param_name: String,
        /// Reason why the value is invalid
        reason: String,
    },

    /// Tool execution failed
    #[error("Tool '{tool_name}' execution failed: {reason}")]
    ExecutionFailed {
        /// Name of the tool
        tool_name: String,
        /// Reason for execution failure
        reason: String,
    },

    /// Tool execution timeout
    #[error("Tool '{tool_name}' execution timed out after {timeout_ms}ms")]
    ExecutionTimeout {
        /// Name of the tool
        tool_name: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// File operation error
    #[error("File operation failed for '{path}': {reason}")]
    FileOperation {
        /// Path that caused the error
        path: PathBuf,
        /// Reason for the failure
        reason: String,
    },

    /// Security validation failed
    #[error("Security validation failed for tool '{tool_name}': {reason}")]
    SecurityValidation {
        /// Name of the tool
        tool_name: String,
        /// Reason for security failure
        reason: String,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded for tool '{tool_name}': {resource_type} limit of {limit} exceeded")]
    ResourceLimitExceeded {
        /// Name of the tool
        tool_name: String,
        /// Type of resource (memory, cpu, etc.)
        resource_type: String,
        /// The limit that was exceeded
        limit: String,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration {
        /// Configuration error message
        message: String,
    },

    /// Serialization error
    #[error("Serialization error: {message}")]
    Serialization {
        /// Serialization error message
        message: String,
    },

    /// Network error
    #[error("Network error: {message}")]
    Network {
        /// Network error message
        message: String,
    },

    /// Generic I/O error
    #[error("I/O error: {message}")]
    Io {
        /// I/O error message
        message: String,
    },
}

/// Registry-specific error type
/// 
/// Specialized error type for tool registry operations.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::errors::RegistryError;
/// 
/// let error = RegistryError::DuplicateRegistration {
///     tool_name: "duplicate_tool".to_string(),
/// };
/// 
/// assert_eq!(error.to_string(), "Tool 'duplicate_tool' is already registered in the registry");
/// ```
#[derive(Error, Debug)]
pub enum RegistryError {
    /// Duplicate tool registration
    #[error("Tool '{tool_name}' is already registered in the registry")]
    DuplicateRegistration {
        /// Name of the tool
        tool_name: String,
    },

    /// Tool not found during lookup
    #[error("Tool '{tool_name}' not found in registry")]
    ToolNotFound {
        /// Name of the tool
        tool_name: String,
    },

    /// Registry is locked and cannot be modified
    #[error("Registry is locked and cannot be modified")]
    RegistryLocked,

    /// Registry corruption detected
    #[error("Registry corruption detected: {details}")]
    RegistryCorruption {
        /// Details about the corruption
        details: String,
    },
}

/// Validation-specific error type
/// 
/// Specialized error type for parameter and input validation.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::errors::ValidationError;
/// 
/// let error = ValidationError::RequiredFieldMissing {
///     field_name: "file_path".to_string(),
///     context: "file reading operation".to_string(),
/// };
/// 
/// assert_eq!(error.to_string(), "Required field 'file_path' missing in file reading operation");
/// ```
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Required field missing
    #[error("Required field '{field_name}' missing in {context}")]
    RequiredFieldMissing {
        /// Name of the missing field
        field_name: String,
        /// Context where the field was expected
        context: String,
    },

    /// Invalid field value
    #[error("Invalid value for field '{field_name}' in {context}: {reason}")]
    InvalidFieldValue {
        /// Name of the field
        field_name: String,
        /// Context where the field was used
        context: String,
        /// Reason why the value is invalid
        reason: String,
    },

    /// Format validation failed
    #[error("Format validation failed for '{field_name}': expected {expected_format}, got {actual_format}")]
    FormatMismatch {
        /// Name of the field
        field_name: String,
        /// Expected format
        expected_format: String,
        /// Actual format received
        actual_format: String,
    },

    /// Range validation failed
    #[error("Value for '{field_name}' is out of range: {value} not in [{min}, {max}]")]
    OutOfRange {
        /// Name of the field
        field_name: String,
        /// The value that was out of range
        value: String,
        /// Minimum allowed value
        min: String,
        /// Maximum allowed value
        max: String,
    },
}

/// Security-specific error type
/// 
/// Specialized error type for security-related operations.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::errors::SecurityError;
/// 
/// let error = SecurityError::InsufficientPermissions {
///     required_capability: "filesystem-write".to_string(),
///     tool_name: "file_writer".to_string(),
/// };
/// 
/// assert_eq!(error.to_string(), "Tool 'file_writer' requires capability 'filesystem-write'");
/// ```
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Insufficient permissions for operation
    #[error("Tool '{tool_name}' requires capability '{required_capability}'")]
    InsufficientPermissions {
        /// Name of the tool
        tool_name: String,
        /// Required capability
        required_capability: String,
    },

    /// Sandbox violation
    #[error("Sandbox violation by tool '{tool_name}': {violation}")]
    SandboxViolation {
        /// Name of the tool
        tool_name: String,
        /// Description of the violation
        violation: String,
    },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed {
        /// Reason for authentication failure
        reason: String,
    },

    /// Authorization failed
    #[error("Authorization failed for operation '{operation}': {reason}")]
    AuthorizationFailed {
        /// Operation that was denied
        operation: String,
        /// Reason for authorization failure
        reason: String,
    },

    /// Security policy violation
    #[error("Security policy violation: {policy} - {details}")]
    PolicyViolation {
        /// Name of the violated policy
        policy: String,
        /// Details about the violation
        details: String,
    },
}

/// Result type alias for tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Result type alias for registry operations
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Result type alias for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Result type alias for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

// Conversion implementations for better error chaining

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::Io {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::Serialization {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

impl From<serde_yaml::Error> for ToolError {
    fn from(err: serde_yaml::Error) -> Self {
        ToolError::Serialization {
            message: format!("YAML serialization error: {}", err),
        }
    }
}

impl From<reqwest::Error> for ToolError {
    fn from(err: reqwest::Error) -> Self {
        ToolError::Network {
            message: err.to_string(),
        }
    }
}

impl From<RegistryError> for ToolError {
    fn from(err: RegistryError) -> Self {
        match err {
            RegistryError::DuplicateRegistration { tool_name } => {
                ToolError::ToolAlreadyRegistered { name: tool_name }
            }
            RegistryError::ToolNotFound { tool_name } => {
                ToolError::ToolNotFound { name: tool_name }
            }
            RegistryError::RegistryLocked => {
                ToolError::Configuration {
                    message: "Registry is locked".to_string(),
                }
            }
            RegistryError::RegistryCorruption { details } => {
                ToolError::Configuration {
                    message: format!("Registry corruption: {}", details),
                }
            }
        }
    }
}

impl From<ValidationError> for ToolError {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::RequiredFieldMissing { field_name, context } => {
                ToolError::MissingParameter {
                    tool_name: context,
                    param_name: field_name,
                }
            }
            ValidationError::InvalidFieldValue { field_name, context, reason } => {
                ToolError::InvalidParameter {
                    tool_name: context,
                    param_name: field_name,
                    reason,
                }
            }
            ValidationError::FormatMismatch { field_name, expected_format, actual_format } => {
                ToolError::ParameterValidation {
                    tool_name: "unknown".to_string(),
                    reason: format!("Format mismatch for '{}': expected {}, got {}", 
                                  field_name, expected_format, actual_format),
                }
            }
            ValidationError::OutOfRange { field_name, value, min, max } => {
                ToolError::ParameterValidation {
                    tool_name: "unknown".to_string(),
                    reason: format!("Value '{}' for '{}' is out of range [{}, {}]", 
                                  value, field_name, min, max),
                }
            }
        }
    }
}

impl From<SecurityError> for ToolError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::InsufficientPermissions { tool_name, required_capability } => {
                ToolError::SecurityValidation {
                    tool_name,
                    reason: format!("Missing required capability: {}", required_capability),
                }
            }
            SecurityError::SandboxViolation { tool_name, violation } => {
                ToolError::SecurityValidation {
                    tool_name,
                    reason: format!("Sandbox violation: {}", violation),
                }
            }
            SecurityError::AuthenticationFailed { reason } => {
                ToolError::SecurityValidation {
                    tool_name: "system".to_string(),
                    reason: format!("Authentication failed: {}", reason),
                }
            }
            SecurityError::AuthorizationFailed { operation, reason } => {
                ToolError::SecurityValidation {
                    tool_name: "system".to_string(),
                    reason: format!("Authorization failed for '{}': {}", operation, reason),
                }
            }
            SecurityError::PolicyViolation { policy, details } => {
                ToolError::SecurityValidation {
                    tool_name: "system".to_string(),
                    reason: format!("Policy violation '{}': {}", policy, details),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error_display() {
        let error = ToolError::ToolNotFound {
            name: "test_tool".to_string(),
        };
        assert_eq!(error.to_string(), "Tool 'test_tool' not found in registry");
    }

    #[test]
    fn test_registry_error_display() {
        let error = RegistryError::DuplicateRegistration {
            tool_name: "duplicate".to_string(),
        };
        assert_eq!(error.to_string(), "Tool 'duplicate' is already registered in the registry");
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::RequiredFieldMissing {
            field_name: "path".to_string(),
            context: "file operation".to_string(),
        };
        assert_eq!(error.to_string(), "Required field 'path' missing in file operation");
    }

    #[test]
    fn test_security_error_display() {
        let error = SecurityError::InsufficientPermissions {
            tool_name: "file_reader".to_string(),
            required_capability: "filesystem-read".to_string(),
        };
        assert_eq!(error.to_string(), "Tool 'file_reader' requires capability 'filesystem-read'");
    }

    #[test]
    fn test_error_conversion() {
        let registry_error = RegistryError::ToolNotFound {
            tool_name: "missing".to_string(),
        };
        let tool_error: ToolError = registry_error.into();
        
        match tool_error {
            ToolError::ToolNotFound { name } => assert_eq!(name, "missing"),
            _ => panic!("Unexpected error type"),
        }
    }
} 