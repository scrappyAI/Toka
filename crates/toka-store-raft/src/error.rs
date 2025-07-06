//! Error types for Raft storage operations.

use thiserror::Error;
use raft_core::RaftError;
use toka_store_core::EventId;

/// Result type for Raft storage operations
pub type RaftStorageResult<T> = Result<T, RaftStorageError>;

/// Errors that can occur during Raft storage operations
#[derive(Debug, Error)]
pub enum RaftStorageError {
    /// Raft consensus error
    #[error("Raft consensus error: {0}")]
    RaftConsensus(#[from] RaftError),

    /// Storage backend error
    #[error("Storage backend error: {0}")]
    StorageBackend(#[from] anyhow::Error),

    /// Network communication error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// MessagePack serialization error
    #[error("MessagePack serialization error: {0}")]
    MessagePack(#[from] rmp_serde::encode::Error),

    /// MessagePack deserialization error
    #[error("MessagePack deserialization error: {0}")]
    MessagePackDecode(#[from] rmp_serde::decode::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Cluster membership error
    #[error("Cluster membership error: {0}")]
    ClusterMembership(String),

    /// Node not found in cluster
    #[error("Node {node_id} not found in cluster")]
    NodeNotFound {
        /// Node ID that was not found
        node_id: u64,
    },

    /// Not the leader node
    #[error("Operation requires leader node, current leader is {leader:?}")]
    NotLeader {
        /// Current leader node ID (if known)
        leader: Option<u64>,
    },

    /// Consensus timeout
    #[error("Consensus operation timed out after {timeout_ms}ms")]
    ConsensusTimeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Event not found
    #[error("Event {event_id} not found in storage")]
    EventNotFound {
        /// Event ID that was not found
        event_id: EventId,
    },

    /// Invalid operation
    #[error("Invalid operation: {reason}")]
    InvalidOperation {
        /// Reason why the operation is invalid
        reason: String,
    },

    /// Cluster is not ready
    #[error("Cluster is not ready: {reason}")]
    ClusterNotReady {
        /// Reason why cluster is not ready
        reason: String,
    },

    /// Snapshot error
    #[error("Snapshot error: {0}")]
    Snapshot(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Task join error
    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    /// Channel send error
    #[error("Channel send error")]
    ChannelSend,

    /// Channel receive error
    #[error("Channel receive error")]
    ChannelReceive,

    /// Lock poisoned
    #[error("Lock poisoned")]
    LockPoisoned,

    /// Operation cancelled
    #[error("Operation was cancelled")]
    Cancelled,

    /// Resource exhausted
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted {
        /// Resource that was exhausted
        resource: String,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl RaftStorageError {
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a cluster membership error
    pub fn cluster_membership(msg: impl Into<String>) -> Self {
        Self::ClusterMembership(msg.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(reason: impl Into<String>) -> Self {
        Self::InvalidOperation {
            reason: reason.into(),
        }
    }

    /// Create a cluster not ready error
    pub fn cluster_not_ready(reason: impl Into<String>) -> Self {
        Self::ClusterNotReady {
            reason: reason.into(),
        }
    }

    /// Create a snapshot error
    pub fn snapshot(msg: impl Into<String>) -> Self {
        Self::Snapshot(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::ConsensusTimeout { .. } => true,
            Self::NotLeader { .. } => true,
            Self::ClusterNotReady { .. } => true,
            Self::ChannelSend => true,
            Self::ChannelReceive => true,
            Self::Cancelled => true,
            Self::ResourceExhausted { .. } => true,
            _ => false,
        }
    }

    /// Check if this error indicates a leadership change
    pub fn is_leadership_change(&self) -> bool {
        matches!(self, Self::NotLeader { .. })
    }

    /// Check if this error indicates a network issue
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::Network(_) | Self::ConsensusTimeout { .. })
    }
}

// Generic implementation for all SendError types
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for RaftStorageError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelSend
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for RaftStorageError {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::ChannelReceive
    }
}

impl<T> From<std::sync::PoisonError<T>> for RaftStorageError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
} 