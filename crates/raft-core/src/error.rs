//! Error types for the Raft consensus algorithm implementation.
//!
//! This module defines all error conditions that can occur during Raft operations,
//! providing rich context for debugging and error recovery.

use thiserror::Error;

/// Result type alias for Raft operations
pub type RaftResult<T> = Result<T, RaftError>;

/// Comprehensive error types for Raft operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RaftError {
    /// Node is not the leader and cannot process write operations
    #[error("Node {node_id} is not the leader (current leader: {current_leader:?})")]
    NotLeader {
        node_id: u64,
        current_leader: Option<u64>,
    },

    /// Invalid term received in message
    #[error("Invalid term: received {received}, current {current}")]
    InvalidTerm { received: u64, current: u64 },

    /// Log entry not found at specified index
    #[error("Log entry not found at index {index}")]
    LogEntryNotFound { index: u64 },

    /// Log inconsistency detected
    #[error("Log inconsistency: expected term {expected} at index {index}, got {actual}")]
    LogInconsistency {
        index: u64,
        expected: u64,
        actual: u64,
    },

    /// Election timeout occurred
    #[error("Election timeout occurred for node {node_id}")]
    ElectionTimeout { node_id: u64 },

    /// Network communication error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Storage operation failed
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Node is in wrong state for operation
    #[error("Invalid state: node {node_id} is {current_state}, expected {expected_state}")]
    InvalidState {
        node_id: u64,
        current_state: String,
        expected_state: String,
    },

    /// Quorum not achieved
    #[error("Quorum not achieved: got {votes} votes, need {required} (cluster size: {cluster_size})")]
    QuorumNotAchieved {
        votes: usize,
        required: usize,
        cluster_size: usize,
    },

    /// Snapshot error
    #[error("Snapshot error: {message}")]
    SnapshotError { message: String },

    /// Invalid message format
    #[error("Invalid message format: {message}")]
    InvalidMessage { message: String },

    /// Timeout during operation
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Generic internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl RaftError {
    /// Create a new NetworkError
    pub fn network<S: Into<String>>(message: S) -> Self {
        RaftError::NetworkError {
            message: message.into(),
        }
    }

    /// Create a new StorageError
    pub fn storage<S: Into<String>>(message: S) -> Self {
        RaftError::StorageError {
            message: message.into(),
        }
    }

    /// Create a new ConfigurationError
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        RaftError::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a new Internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        RaftError::Internal {
            message: message.into(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RaftError::NetworkError { .. }
                | RaftError::Timeout { .. }
                | RaftError::ElectionTimeout { .. }
        )
    }

    /// Check if this error indicates a leadership change
    pub fn is_leadership_change(&self) -> bool {
        matches!(
            self,
            RaftError::NotLeader { .. } | RaftError::InvalidTerm { .. }
        )
    }
}

/// Convert from anyhow::Error to RaftError
impl From<anyhow::Error> for RaftError {
    fn from(err: anyhow::Error) -> Self {
        RaftError::Internal {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RaftError::network("Connection failed");
        assert!(error.is_retryable());
        assert!(!error.is_leadership_change());
    }

    #[test]
    fn test_leadership_change_detection() {
        let error = RaftError::NotLeader {
            node_id: 1,
            current_leader: Some(2),
        };
        assert!(error.is_leadership_change());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        let timeout_error = RaftError::Timeout {
            operation: "append_entries".to_string(),
        };
        assert!(timeout_error.is_retryable());

        let storage_error = RaftError::storage("Disk full");
        assert!(!storage_error.is_retryable());
    }
}