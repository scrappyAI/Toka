#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-raft** – Raft consensus-backed storage for Toka OS.
//!
//! This crate provides a distributed storage backend that uses Raft consensus
//! to ensure consistency across multiple nodes in a Toka cluster. It bridges
//! the existing Raft implementation with Toka's storage abstraction layer.
//!
//! ## Key Components
//!
//! - **RaftStorage**: Main storage backend implementing StorageBackend trait
//! - **TokaStateMachine**: Raft state machine for Toka operations
//! - **RaftNetwork**: Network layer for inter-node Raft communication
//! - **ClusterConfig**: Configuration for Raft cluster setup
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use toka_store_raft::{RaftStorage, RaftClusterConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RaftClusterConfig {
//!     node_id: 1,
//!     peers: vec![(2, "node2:8080".to_string()), (3, "node3:8080".to_string())],
//!     heartbeat_interval: Duration::from_millis(50),
//!     election_timeout: (Duration::from_millis(150), Duration::from_millis(300)),
//!     storage_path: "/var/lib/toka/raft".into(),
//!     bind_address: "0.0.0.0:8080".to_string(),
//! };
//!
//! let storage = RaftStorage::new(config).await?;
//! // Use storage with Toka runtime...
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use toka_store_core::{EventHeader, EventId, StorageBackend, CausalDigest};
use toka_bus_core::{EventBus, KernelEvent};
use toka_types::Message;
use raft_core::{LogEntry, RaftNode, RaftConfig, Term};
use raft_storage::Storage;
use uuid::Uuid;

pub mod storage;
pub mod state_machine;
pub mod network;
pub mod config;
pub mod error;
pub mod distributed_kernel;

pub use storage::RaftStorage;
pub use state_machine::TokaStateMachine;
pub use network::RaftNetwork;
pub use config::RaftClusterConfig;
pub use error::{RaftStorageError, RaftStorageResult};

/// Operations that can be proposed to the Raft cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokaOperation {
    /// Commit an event to storage
    CommitEvent {
        /// Event header
        header: EventHeader,
        /// Event payload bytes
        payload: Vec<u8>,
    },
    /// Process a message through the distributed kernel
    ProcessMessage {
        /// Message to be processed
        message: Message,
        /// Unique request ID for tracking
        request_id: Uuid,
    },
    /// Compact log entries before a certain index
    CompactLog {
        /// Index before which to compact
        before_index: u64,
    },
    /// Take a snapshot of the current state
    TakeSnapshot,
    /// Install a snapshot
    InstallSnapshot {
        /// Snapshot data
        data: Vec<u8>,
        /// Last included index
        last_included_index: u64,
        /// Last included term
        last_included_term: Term,
    },
}

/// Result of executing a Toka operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokaOperationResult {
    /// Event was successfully committed
    EventCommitted {
        /// Event ID that was committed
        event_id: EventId,
    },
    /// Message was successfully processed
    MessageProcessed {
        /// Resulting kernel event
        event: KernelEvent,
    },
    /// Log was compacted
    LogCompacted {
        /// Number of entries removed
        entries_removed: u64,
    },
    /// Snapshot was taken
    SnapshotTaken {
        /// Size of snapshot in bytes
        snapshot_size: usize,
    },
    /// Snapshot was installed
    SnapshotInstalled {
        /// Last included index
        last_included_index: u64,
    },
    /// Operation failed
    Failed {
        /// Error message
        error: String,
    },
}

/// Status of a node in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is active and healthy
    Active,
    /// Node is inactive or unreachable
    Inactive,
    /// Node status is unknown
    Unknown,
    /// Node has failed
    Failed,
}

/// Node information for cluster members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID
    pub id: u64,
    /// Network address
    pub address: String,
    /// Current status of the node
    pub status: NodeStatus,
    /// Last known health check timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Cluster topology information
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    /// Information about all nodes in the cluster
    pub nodes: HashMap<u64, NodeInfo>,
    /// Current leader node ID (if known)
    pub leader: Option<u64>,
    /// Current term
    pub term: Term,
}

/// Metrics for monitoring Raft cluster health
#[derive(Debug, Clone, Default)]
pub struct RaftMetrics {
    /// Number of leadership changes
    pub leadership_changes: u64,
    /// Average log replication latency
    pub replication_latency_ms: f64,
    /// Current consensus throughput (operations per second)
    pub consensus_throughput: f64,
    /// Number of failed consensus attempts
    pub failed_consensus_attempts: u64,
    /// Current log size
    pub log_size: u64,
    /// Number of snapshots taken
    pub snapshots_taken: u64,
}

/// Health status of a cluster node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeHealth {
    /// Node is healthy and responsive
    Healthy,
    /// Node is slow to respond
    Slow,
    /// Node is unreachable
    Unreachable,
    /// Node has failed
    Failed,
} 