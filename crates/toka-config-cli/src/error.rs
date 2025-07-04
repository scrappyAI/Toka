//! Error types and handling for configuration management operations.
//!
//! This module defines comprehensive error types using thiserror to provide
//! structured error handling with meaningful context and user-friendly messages.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for configuration management operations.
///
/// Provides structured error information with context for different failure modes
/// including file operations, format parsing, validation, and I/O errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found at the specified path.
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid or unsupported configuration file format.
    #[error("Invalid configuration format for file '{file}': {reason}")]
    InvalidFormat {
        /// The file path that caused the error
        file: PathBuf,
        /// Detailed reason for the format error
        reason: String,
    },

    /// Configuration validation failed.
    #[error("Configuration validation failed for '{file}': {details}")]
    ValidationError {
        /// The file path that failed validation
        file: PathBuf,
        /// Detailed validation error information
        details: String,
    },

    /// Failed to parse configuration content.
    #[error("Failed to parse configuration in '{file}': {source}")]
    ParseError {
        /// The file path that failed to parse
        file: PathBuf,
        /// The underlying parsing error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// File I/O operation failed.
    #[error("I/O error for file '{file}': {source}")]
    IoError {
        /// The file path involved in the I/O operation
        file: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Key not found in configuration.
    #[error("Key '{key}' not found in configuration file '{file}'")]
    KeyNotFound {
        /// The key that was not found
        key: String,
        /// The file where the key was searched
        file: PathBuf,
    },

    /// Invalid key path format.
    #[error("Invalid key path '{key}': {reason}")]
    InvalidKeyPath {
        /// The invalid key path
        key: String,
        /// Reason why the key path is invalid
        reason: String,
    },

    /// Failed to serialize configuration data.
    #[error("Failed to serialize configuration to '{format}' format: {source}")]
    SerializationError {
        /// The target format for serialization
        format: String,
        /// The underlying serialization error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Permission denied for file operation.
    #[error("Permission denied for file operation on '{file}'")]
    PermissionDenied {
        /// The file path where permission was denied
        file: PathBuf,
    },

    /// Directory operation failed.
    #[error("Directory operation failed for '{directory}': {reason}")]
    DirectoryError {
        /// The directory path that caused the error
        directory: PathBuf,
        /// Detailed reason for the directory error
        reason: String,
    },
}

impl ConfigError {
    /// Create a new file not found error.
    pub fn file_not_found(path: PathBuf) -> Self {
        Self::FileNotFound(path)
    }

    /// Create a new invalid format error with context.
    pub fn invalid_format<P: Into<PathBuf>, S: Into<String>>(file: P, reason: S) -> Self {
        Self::InvalidFormat {
            file: file.into(),
            reason: reason.into(),
        }
    }

    /// Create a new validation error with details.
    pub fn validation_error<P: Into<PathBuf>, S: Into<String>>(file: P, details: S) -> Self {
        Self::ValidationError {
            file: file.into(),
            details: details.into(),
        }
    }

    /// Create a new parse error with source.
    pub fn parse_error<P: Into<PathBuf>, E: std::error::Error + Send + Sync + 'static>(
        file: P,
        source: E,
    ) -> Self {
        Self::ParseError {
            file: file.into(),
            source: Box::new(source),
        }
    }

    /// Create a new I/O error with file context.
    pub fn io_error<P: Into<PathBuf>>(file: P, source: std::io::Error) -> Self {
        Self::IoError {
            file: file.into(),
            source,
        }
    }

    /// Create a new key not found error.
    pub fn key_not_found<K: Into<String>, P: Into<PathBuf>>(key: K, file: P) -> Self {
        Self::KeyNotFound {
            key: key.into(),
            file: file.into(),
        }
    }

    /// Create a new invalid key path error.
    pub fn invalid_key_path<K: Into<String>, R: Into<String>>(key: K, reason: R) -> Self {
        Self::InvalidKeyPath {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Create a new serialization error.
    pub fn serialization_error<F: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        format: F,
        source: E,
    ) -> Self {
        Self::SerializationError {
            format: format.into(),
            source: Box::new(source),
        }
    }

    /// Create a new permission denied error.
    pub fn permission_denied<P: Into<PathBuf>>(file: P) -> Self {
        Self::PermissionDenied { file: file.into() }
    }

    /// Create a new directory error.
    pub fn directory_error<P: Into<PathBuf>, R: Into<String>>(directory: P, reason: R) -> Self {
        Self::DirectoryError {
            directory: directory.into(),
            reason: reason.into(),
        }
    }

    /// Check if this error indicates a missing file.
    pub fn is_file_not_found(&self) -> bool {
        matches!(self, Self::FileNotFound(_))
    }

    /// Check if this error indicates a permission problem.
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Self::PermissionDenied { .. })
    }

    /// Check if this error indicates a validation problem.
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError { .. })
    }
}

/// Type alias for results that may contain configuration errors.
pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_error_construction() {
        let path = PathBuf::from("test.yaml");
        
        // Test file not found error
        let error = ConfigError::file_not_found(path.clone());
        assert!(error.is_file_not_found());
        assert!(!error.is_permission_error());
        
        // Test validation error
        let error = ConfigError::validation_error(&path, "Invalid syntax");
        assert!(error.is_validation_error());
        assert!(!error.is_file_not_found());
    }

    #[test]
    fn test_error_display() {
        let path = PathBuf::from("config.json");
        let error = ConfigError::file_not_found(path);
        
        let error_string = error.to_string();
        assert!(error_string.contains("Configuration file not found"));
        assert!(error_string.contains("config.json"));
    }

    #[test]
    fn test_invalid_format_error() {
        let error = ConfigError::invalid_format("test.xyz", "Unsupported extension");
        
        match error {
            ConfigError::InvalidFormat { file, reason } => {
                assert_eq!(file, PathBuf::from("test.xyz"));
                assert_eq!(reason, "Unsupported extension");
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }
}