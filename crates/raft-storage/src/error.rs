//! Error types for Raft storage operations.

use thiserror::Error;

/// Result type alias for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Comprehensive error types for storage operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StorageError {
    /// Entry not found at the specified index
    #[error("Log entry not found at index {index}")]
    EntryNotFound { index: u64 },

    /// Index out of range
    #[error("Index {index} is out of range (first: {first}, last: {last})")]
    IndexOutOfRange {
        index: u64,
        first: u64,
        last: u64,
    },

    /// Storage corruption detected
    #[error("Storage corruption detected: {message}")]
    Corruption { message: String },

    /// I/O error during storage operation
    #[error("I/O error: {message}")]
    IoError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Invalid storage format
    #[error("Invalid storage format: {message}")]
    InvalidFormat { message: String },

    /// Storage is read-only
    #[error("Storage is read-only")]
    ReadOnly,

    /// Insufficient disk space
    #[error("Insufficient disk space: needed {needed} bytes, available {available} bytes")]
    InsufficientSpace { needed: u64, available: u64 },

    /// Lock contention error
    #[error("Lock contention: {message}")]
    LockContention { message: String },

    /// Snapshot error
    #[error("Snapshot error: {message}")]
    SnapshotError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Generic internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl StorageError {
    /// Create a new IoError
    pub fn io<S: Into<String>>(message: S) -> Self {
        StorageError::IoError {
            message: message.into(),
        }
    }

    /// Create a new SerializationError
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        StorageError::SerializationError {
            message: message.into(),
        }
    }

    /// Create a new Corruption error
    pub fn corruption<S: Into<String>>(message: S) -> Self {
        StorageError::Corruption {
            message: message.into(),
        }
    }

    /// Create a new InvalidFormat error
    pub fn invalid_format<S: Into<String>>(message: S) -> Self {
        StorageError::InvalidFormat {
            message: message.into(),
        }
    }

    /// Create a new Internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        StorageError::Internal {
            message: message.into(),
        }
    }

    /// Create a new SnapshotError
    pub fn snapshot<S: Into<String>>(message: S) -> Self {
        StorageError::SnapshotError {
            message: message.into(),
        }
    }

    /// Create a new ConfigurationError
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        StorageError::ConfigurationError {
            message: message.into(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StorageError::IoError { .. }
                | StorageError::LockContention { .. }
                | StorageError::InsufficientSpace { .. }
        )
    }

    /// Check if this error indicates corruption
    pub fn is_corruption(&self) -> bool {
        matches!(
            self,
            StorageError::Corruption { .. } | StorageError::InvalidFormat { .. }
        )
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !self.is_corruption() && !matches!(self, StorageError::ReadOnly)
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::io(err.to_string())
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::serialization(err.to_string())
    }
}

/// Convert from raft_core::RaftError
impl From<raft_core::RaftError> for StorageError {
    fn from(err: raft_core::RaftError) -> Self {
        match err {
            raft_core::RaftError::LogEntryNotFound { index } => {
                StorageError::EntryNotFound { index }
            }
            raft_core::RaftError::StorageError { message } => {
                StorageError::internal(message)
            }
            _ => StorageError::internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let io_error = StorageError::io("File not found");
        assert!(io_error.is_retryable());
        assert!(!io_error.is_corruption());
        assert!(io_error.is_recoverable());

        let corruption_error = StorageError::corruption("Checksum mismatch");
        assert!(!corruption_error.is_retryable());
        assert!(corruption_error.is_corruption());
        assert!(!corruption_error.is_recoverable());
    }

    #[test]
    fn test_error_classification() {
        let read_only = StorageError::ReadOnly;
        assert!(!read_only.is_retryable());
        assert!(!read_only.is_corruption());
        assert!(!read_only.is_recoverable());

        let lock_contention = StorageError::LockContention {
            message: "Lock timeout".to_string(),
        };
        assert!(lock_contention.is_retryable());
        assert!(!lock_contention.is_corruption());
        assert!(lock_contention.is_recoverable());
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let storage_err = StorageError::from(io_err);
        assert!(matches!(storage_err, StorageError::IoError { .. }));

        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let storage_err = StorageError::from(json_err);
        assert!(matches!(storage_err, StorageError::SerializationError { .. }));
    }
}