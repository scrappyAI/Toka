//! # Raft Storage
//!
//! Storage abstraction and implementations for the Raft consensus algorithm.
//! This crate provides traits and implementations for persistent storage of
//! Raft state, including log entries, snapshots, and cluster metadata.
//!
//! ## Key Components
//!
//! - **Storage Trait**: Abstract interface for Raft storage operations
//! - **Memory Storage**: In-memory implementation for testing and development
//! - **File Storage**: File-based persistent storage implementation
//! - **Snapshot Management**: Utilities for creating and managing snapshots
//!
//! ## Example Usage
//!
//! ```rust
//! use raft_storage::{MemoryStorage, Storage};
//! use raft_core::{LogEntry, Term};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut storage = MemoryStorage::new();
//! 
//! // Store a log entry
//! let entry = LogEntry::new_command(1, 1, b"SET key value".to_vec());
//! storage.store_log_entry(&entry).await?;
//! 
//! // Retrieve the entry
//! let retrieved = storage.get_log_entry(1).await?;
//! assert_eq!(retrieved.unwrap().data, b"SET key value");
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod storage;
pub mod memory;
pub mod file;
pub mod snapshot;

pub use error::{StorageError, StorageResult};
pub use storage::{Storage, StorageMetrics};
pub use memory::MemoryStorage;
pub use file::FileStorage;
pub use snapshot::{SnapshotManager, SnapshotMetadata};

use raft_core::{LogEntry, LogIndex, Term};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Persistent state that must be maintained across restarts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistentState {
    /// Current term
    pub current_term: Term,
    
    /// Node ID that received vote in current term
    pub voted_for: Option<u64>,
    
    /// Last applied log index
    pub last_applied: LogIndex,
    
    /// Cluster configuration
    pub cluster_config: ClusterConfig,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            last_applied: 0,
            cluster_config: ClusterConfig::default(),
        }
    }
}

/// Cluster configuration information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterConfig {
    /// All nodes in the cluster
    pub nodes: HashMap<u64, NodeInfo>,
    
    /// Configuration change index (for joint consensus)
    pub config_index: LogIndex,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            config_index: 0,
        }
    }
}

/// Information about a cluster node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInfo {
    /// Node ID
    pub id: u64,
    
    /// Node address for communication
    pub address: String,
    
    /// Whether this node is a voting member
    pub voting: bool,
    
    /// When this node was added to the cluster
    pub added_at: chrono::DateTime<chrono::Utc>,
}

impl NodeInfo {
    /// Create a new voting node
    pub fn voting(id: u64, address: String) -> Self {
        Self {
            id,
            address,
            voting: true,
            added_at: chrono::Utc::now(),
        }
    }
    
    /// Create a new non-voting node
    pub fn non_voting(id: u64, address: String) -> Self {
        Self {
            id,
            address,
            voting: false,
            added_at: chrono::Utc::now(),
        }
    }
}

/// Log entry with storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLogEntry {
    /// The log entry data
    pub entry: LogEntry,
    
    /// Storage checksum for integrity
    pub checksum: u32,
    
    /// When this entry was stored
    pub stored_at: chrono::DateTime<chrono::Utc>,
}

impl StoredLogEntry {
    /// Create a new stored log entry
    pub fn new(entry: LogEntry) -> Self {
        let checksum = Self::calculate_checksum(&entry);
        Self {
            entry,
            checksum,
            stored_at: chrono::Utc::now(),
        }
    }
    
    /// Verify the integrity of the stored entry
    pub fn verify_integrity(&self) -> bool {
        Self::calculate_checksum(&self.entry) == self.checksum
    }
    
    /// Calculate checksum for an entry
    fn calculate_checksum(entry: &LogEntry) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        entry.term.hash(&mut hasher);
        entry.index.hash(&mut hasher);
        entry.data.hash(&mut hasher);
        entry.entry_type.hash(&mut hasher);
        
        hasher.finish() as u32
    }
}

/// Current version of the storage format
pub const STORAGE_VERSION: u32 = 1;

/// Magic bytes for identifying Raft storage files
pub const STORAGE_MAGIC: &[u8] = b"RAFT";

#[cfg(test)]
mod tests {
    use super::*;
    use raft_core::LogEntry;

    #[test]
    fn test_stored_log_entry() {
        let entry = LogEntry::new_command(1, 1, b"test data".to_vec());
        let stored = StoredLogEntry::new(entry.clone());
        
        assert_eq!(stored.entry, entry);
        assert!(stored.verify_integrity());
        
        // Test with corrupted checksum
        let mut corrupted = stored.clone();
        corrupted.checksum = 0;
        assert!(!corrupted.verify_integrity());
    }
    
    #[test]
    fn test_node_info() {
        let voting = NodeInfo::voting(1, "localhost:8080".to_string());
        assert!(voting.voting);
        assert_eq!(voting.id, 1);
        assert_eq!(voting.address, "localhost:8080");
        
        let non_voting = NodeInfo::non_voting(2, "localhost:8081".to_string());
        assert!(!non_voting.voting);
        assert_eq!(non_voting.id, 2);
    }
    
    #[test]
    fn test_persistent_state_serialization() {
        let mut state = PersistentState::default();
        state.current_term = 5;
        state.voted_for = Some(1);
        state.last_applied = 10;
        
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: PersistentState = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(state, deserialized);
    }
}