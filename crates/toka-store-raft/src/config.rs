//! Configuration for Raft cluster setup and operation.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use raft_core::RaftConfig;
use crate::error::{RaftStorageError, RaftStorageResult};

/// Configuration for a Raft cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftClusterConfig {
    /// This node's ID in the cluster
    pub node_id: u64,
    
    /// List of peer nodes with their addresses
    pub peers: Vec<(u64, String)>,
    
    /// Heartbeat interval for leader
    pub heartbeat_interval: Duration,
    
    /// Election timeout range (min, max)
    pub election_timeout: (Duration, Duration),
    
    /// Path for storing Raft logs and snapshots
    pub storage_path: PathBuf,
    
    /// Address to bind for incoming connections
    pub bind_address: String,
    
    /// Maximum number of log entries per AppendEntries request
    pub max_entries_per_request: usize,
    
    /// Maximum size of log before triggering compaction
    pub max_log_size: u64,
    
    /// Interval between snapshot attempts
    pub snapshot_interval: Duration,
    
    /// Timeout for consensus operations
    pub consensus_timeout: Duration,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// TLS configuration (optional)
    pub tls: Option<TlsConfig>,
}

/// Network configuration for Raft cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Read timeout for network operations
    pub read_timeout: Duration,
    
    /// Write timeout for network operations
    pub write_timeout: Duration,
    
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    
    /// TCP keep-alive interval
    pub tcp_keepalive: Option<Duration>,
    
    /// TCP no-delay setting
    pub tcp_nodelay: bool,
    
    /// Maximum message size
    pub max_message_size: usize,
    
    /// Retry configuration
    pub retry: RetryConfig,
}

/// Retry configuration for network operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    
    /// Initial retry delay
    pub initial_delay: Duration,
    
    /// Maximum retry delay
    pub max_delay: Duration,
    
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Jitter to add to retry delays
    pub jitter: bool,
}

/// Storage configuration for Raft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory for storing data
    pub data_dir: PathBuf,
    
    /// Directory for storing snapshots
    pub snapshot_dir: PathBuf,
    
    /// Maximum size of a single log file
    pub max_log_file_size: u64,
    
    /// Maximum number of log files to keep
    pub max_log_files: usize,
    
    /// Enable compression for logs
    pub enable_compression: bool,
    
    /// Sync mode for durability
    pub sync_mode: SyncMode,
    
    /// Buffer size for I/O operations
    pub io_buffer_size: usize,
    
    /// Enable background compaction
    pub enable_background_compaction: bool,
    
    /// Compaction threshold (percentage of log to trigger compaction)
    pub compaction_threshold: f64,
}

/// Sync mode for storage durability
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncMode {
    /// No explicit syncing (fastest, least durable)
    None,
    
    /// Normal fsync after writes (balanced)
    Normal,
    
    /// Full sync including metadata (slowest, most durable)
    Full,
}

/// TLS configuration for secure cluster communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the certificate file
    pub cert_path: PathBuf,
    
    /// Path to the private key file
    pub key_path: PathBuf,
    
    /// Path to the CA certificate file
    pub ca_cert_path: PathBuf,
    
    /// Whether to verify peer certificates
    pub verify_peer: bool,
    
    /// Server name for certificate validation
    pub server_name: Option<String>,
}

impl Default for RaftClusterConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            peers: Vec::new(),
            heartbeat_interval: Duration::from_millis(50),
            election_timeout: (Duration::from_millis(150), Duration::from_millis(300)),
            storage_path: PathBuf::from("./data/raft"),
            bind_address: "0.0.0.0:8080".to_string(),
            max_entries_per_request: 100,
            max_log_size: 1024 * 1024 * 1024, // 1GB
            snapshot_interval: Duration::from_secs(3600), // 1 hour
            consensus_timeout: Duration::from_secs(10),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            tls: None,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_secs(10),
            max_connections: 100,
            tcp_keepalive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
            max_message_size: 1024 * 1024, // 1MB
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data/raft/logs"),
            snapshot_dir: PathBuf::from("./data/raft/snapshots"),
            max_log_file_size: 64 * 1024 * 1024, // 64MB
            max_log_files: 10,
            enable_compression: true,
            sync_mode: SyncMode::Normal,
            io_buffer_size: 64 * 1024, // 64KB
            enable_background_compaction: true,
            compaction_threshold: 0.5, // 50%
        }
    }
}

impl RaftClusterConfig {
    /// Create a new cluster configuration with the specified node ID
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            ..Default::default()
        }
    }
    
    /// Create a single-node cluster configuration (for testing)
    pub fn single_node(node_id: u64, bind_address: String) -> Self {
        Self {
            node_id,
            bind_address,
            ..Default::default()
        }
    }
    
    /// Add a peer to the cluster
    pub fn add_peer(mut self, peer_id: u64, address: String) -> Self {
        self.peers.push((peer_id, address));
        self
    }
    
    /// Set the storage path
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = path;
        self
    }
    
    /// Set the bind address
    pub fn with_bind_address(mut self, address: String) -> Self {
        self.bind_address = address;
        self
    }
    
    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }
    
    /// Set election timeout range
    pub fn with_election_timeout(mut self, min: Duration, max: Duration) -> Self {
        self.election_timeout = (min, max);
        self
    }
    
    /// Enable TLS with the specified configuration
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls = Some(tls_config);
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> RaftStorageResult<()> {
        // Validate node ID
        if self.node_id == 0 {
            return Err(RaftStorageError::configuration("Node ID cannot be 0"));
        }
        
        // Validate peers - allow single-node clusters for testing
        // In production, clusters should have at least one peer, but single-node
        // clusters are useful for development and testing
        if !self.peers.is_empty() {
            // Only validate peer-related constraints if we have peers
            
            // Check for duplicate peer IDs
            let mut seen_ids = std::collections::HashSet::new();
            for (peer_id, _) in &self.peers {
                if *peer_id == self.node_id {
                    return Err(RaftStorageError::configuration("Peer ID cannot be the same as node ID"));
                }
                if !seen_ids.insert(*peer_id) {
                    return Err(RaftStorageError::configuration(format!("Duplicate peer ID: {}", peer_id)));
                }
            }
            
            // Validate peer addresses
            for (peer_id, address) in &self.peers {
                if let Err(e) = address.parse::<SocketAddr>() {
                    return Err(RaftStorageError::configuration(format!("Invalid peer address for node {}: {}", peer_id, e)));
                }
            }
        }
        
        // Validate timeouts
        if self.election_timeout.0 >= self.election_timeout.1 {
            return Err(RaftStorageError::configuration("Election timeout min must be less than max"));
        }
        
        if self.heartbeat_interval >= self.election_timeout.0 {
            return Err(RaftStorageError::configuration("Heartbeat interval must be less than election timeout"));
        }
        
        // Validate bind address
        if let Err(e) = self.bind_address.parse::<SocketAddr>() {
            return Err(RaftStorageError::configuration(format!("Invalid bind address: {}", e)));
        }
        

        
        // Validate storage configuration
        if self.storage.max_log_file_size == 0 {
            return Err(RaftStorageError::configuration("Max log file size cannot be 0"));
        }
        
        if self.storage.max_log_files == 0 {
            return Err(RaftStorageError::configuration("Max log files cannot be 0"));
        }
        
        if self.storage.compaction_threshold <= 0.0 || self.storage.compaction_threshold >= 1.0 {
            return Err(RaftStorageError::configuration("Compaction threshold must be between 0 and 1"));
        }
        
        Ok(())
    }
    
    /// Convert to Raft core configuration
    pub fn to_raft_config(&self) -> RaftConfig {
        RaftConfig {
            node_id: self.node_id,
            peers: self.peers.iter().map(|(id, _)| *id).collect(),
            heartbeat_interval: self.heartbeat_interval,
            election_timeout_min: self.election_timeout.0,
            election_timeout_max: self.election_timeout.1,
            max_entries_per_request: self.max_entries_per_request,
            enable_pre_vote: true,
            max_log_size: self.max_log_size as usize,
            compaction_threshold: 1000, // Default threshold
            request_timeout: self.consensus_timeout,
            enable_leadership_transfer: false,
            enable_batch_optimization: true,
        }
    }
    
    /// Get peer addresses as a HashMap
    pub fn peer_addresses(&self) -> HashMap<u64, String> {
        self.peers.iter().cloned().collect()
    }
    
    /// Get all node IDs in the cluster (including this node)
    pub fn all_node_ids(&self) -> Vec<u64> {
        let mut ids = vec![self.node_id];
        ids.extend(self.peers.iter().map(|(id, _)| *id));
        ids.sort();
        ids
    }
    
    /// Get the cluster size
    pub fn cluster_size(&self) -> usize {
        self.peers.len() + 1
    }
    
    /// Get the majority threshold for consensus
    pub fn majority_threshold(&self) -> usize {
        (self.cluster_size() / 2) + 1
    }
} 