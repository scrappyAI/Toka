//! # Raft Core
//!
//! Core implementation of the Raft consensus algorithm, providing distributed consensus
//! for replicated state machines. This implementation follows the Raft paper by Diego Ongaro
//! and John Ousterhout, ensuring safety and liveness properties in distributed systems.
//!
//! ## Key Components
//!
//! - **RaftNode**: Main orchestrator managing state transitions and consensus operations
//! - **State Management**: Follower, Candidate, and Leader states with proper transitions
//! - **Log Replication**: Consistent log replication across cluster members
//! - **Leader Election**: Randomized election timeouts with term-based safety
//! - **Safety Mechanisms**: Term validation, log matching, and commit safety
//!
//! ## Example Usage
//!
//! ```rust
//! use raft_core::{RaftNode, RaftConfig};
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RaftConfig {
//!     node_id: 1,
//!     peers: vec![2, 3],
//!     heartbeat_interval: std::time::Duration::from_millis(50),
//!     election_timeout_min: std::time::Duration::from_millis(150),
//!     election_timeout_max: std::time::Duration::from_millis(300),
//! };
//!
//! let node = RaftNode::new(config).await?;
//! // Start the node and begin consensus operations
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod log;
pub mod node;
pub mod state;
pub mod message;
pub mod config;

pub use error::{RaftError, RaftResult};
pub use log::{LogEntry, LogIndex, Term};
pub use node::RaftNode;
pub use state::{RaftState, NodeState};
pub use message::{
    AppendEntriesRequest, AppendEntriesResponse,
    VoteRequest, VoteResponse,
    Message, MessageType,
};
pub use config::RaftConfig;

/// Current version of the Raft implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of entries per AppendEntries request
pub const MAX_ENTRIES_PER_REQUEST: usize = 100;

/// Default heartbeat interval in milliseconds
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 50;

/// Default minimum election timeout in milliseconds
pub const DEFAULT_ELECTION_TIMEOUT_MIN_MS: u64 = 150;

/// Default maximum election timeout in milliseconds
pub const DEFAULT_ELECTION_TIMEOUT_MAX_MS: u64 = 300;