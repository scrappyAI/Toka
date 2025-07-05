//! Configuration module for Raft consensus algorithm.
//!
//! This module provides configuration structures and validation for Raft nodes,
//! including timing parameters, cluster membership, and operational settings.

use crate::error::{RaftError, RaftResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for a Raft node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    /// Unique identifier for this node
    pub node_id: u64,
    
    /// List of peer node IDs in the cluster
    pub peers: Vec<u64>,
    
    /// Interval between heartbeat messages when leader
    pub heartbeat_interval: Duration,
    
    /// Minimum election timeout duration
    pub election_timeout_min: Duration,
    
    /// Maximum election timeout duration
    pub election_timeout_max: Duration,
    
    /// Maximum number of log entries to send in a single AppendEntries request
    pub max_entries_per_request: usize,
    
    /// Whether to enable pre-vote optimization
    pub enable_pre_vote: bool,
    
    /// Maximum size of the log before triggering compaction
    pub max_log_size: usize,
    
    /// Number of applied entries to keep after compaction
    pub compaction_threshold: usize,
    
    /// Maximum time to wait for a response from a peer
    pub request_timeout: Duration,
    
    /// Whether to enable leadership transfer
    pub enable_leadership_transfer: bool,
    
    /// Whether to enable batch optimization for log entries
    pub enable_batch_optimization: bool,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            peers: vec![],
            heartbeat_interval: Duration::from_millis(50),
            election_timeout_min: Duration::from_millis(150),
            election_timeout_max: Duration::from_millis(300),
            max_entries_per_request: 100,
            enable_pre_vote: false,
            max_log_size: 10000,
            compaction_threshold: 1000,
            request_timeout: Duration::from_millis(1000),
            enable_leadership_transfer: false,
            enable_batch_optimization: true,
        }
    }
}

impl RaftConfig {
    /// Create a new RaftConfig with basic settings
    pub fn new(node_id: u64, peers: Vec<u64>) -> Self {
        Self {
            node_id,
            peers,
            ..Default::default()
        }
    }

    /// Set heartbeat interval (must be much smaller than election timeout)
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set election timeout range
    pub fn with_election_timeout(mut self, min: Duration, max: Duration) -> Self {
        self.election_timeout_min = min;
        self.election_timeout_max = max;
        self
    }

    /// Set maximum entries per request
    pub fn with_max_entries_per_request(mut self, max: usize) -> Self {
        self.max_entries_per_request = max;
        self
    }

    /// Enable pre-vote optimization
    pub fn with_pre_vote(mut self, enable: bool) -> Self {
        self.enable_pre_vote = enable;
        self
    }

    /// Set log compaction parameters
    pub fn with_log_compaction(mut self, max_size: usize, threshold: usize) -> Self {
        self.max_log_size = max_size;
        self.compaction_threshold = threshold;
        self
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Enable leadership transfer
    pub fn with_leadership_transfer(mut self, enable: bool) -> Self {
        self.enable_leadership_transfer = enable;
        self
    }

    /// Enable batch optimization
    pub fn with_batch_optimization(mut self, enable: bool) -> Self {
        self.enable_batch_optimization = enable;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> RaftResult<()> {
        // Validate node ID
        if self.node_id == 0 {
            return Err(RaftError::configuration("Node ID cannot be zero"));
        }

        // Validate peer list doesn't contain self
        if self.peers.contains(&self.node_id) {
            return Err(RaftError::configuration(
                "Peer list cannot contain the node's own ID",
            ));
        }

        // Validate timing constraints
        if self.heartbeat_interval >= self.election_timeout_min {
            return Err(RaftError::configuration(
                "Heartbeat interval must be much smaller than election timeout",
            ));
        }

        if self.election_timeout_min >= self.election_timeout_max {
            return Err(RaftError::configuration(
                "Election timeout minimum must be less than maximum",
            ));
        }

        // Validate that election timeout is reasonable
        if self.election_timeout_min < Duration::from_millis(10) {
            return Err(RaftError::configuration(
                "Election timeout minimum too small (< 10ms)",
            ));
        }

        if self.election_timeout_max > Duration::from_secs(30) {
            return Err(RaftError::configuration(
                "Election timeout maximum too large (> 30s)",
            ));
        }

        // Validate batch size
        if self.max_entries_per_request == 0 {
            return Err(RaftError::configuration(
                "Max entries per request cannot be zero",
            ));
        }

        if self.max_entries_per_request > 10000 {
            return Err(RaftError::configuration(
                "Max entries per request too large (> 10000)",
            ));
        }

        // Validate log compaction settings
        if self.compaction_threshold > self.max_log_size {
            return Err(RaftError::configuration(
                "Compaction threshold cannot be larger than max log size",
            ));
        }

        // Validate request timeout
        if self.request_timeout < Duration::from_millis(1) {
            return Err(RaftError::configuration(
                "Request timeout too small (< 1ms)",
            ));
        }

        Ok(())
    }

    /// Get the cluster size (including this node)
    pub fn cluster_size(&self) -> usize {
        self.peers.len() + 1
    }

    /// Get the quorum size needed for decisions
    pub fn quorum_size(&self) -> usize {
        (self.cluster_size() / 2) + 1
    }

    /// Get all node IDs in the cluster (including this node)
    pub fn all_nodes(&self) -> Vec<u64> {
        let mut nodes = self.peers.clone();
        nodes.push(self.node_id);
        nodes.sort();
        nodes
    }

    /// Check if a node ID is part of the cluster
    pub fn is_member(&self, node_id: u64) -> bool {
        node_id == self.node_id || self.peers.contains(&node_id)
    }

    /// Generate a random election timeout within the configured range
    pub fn random_election_timeout(&self) -> Duration {
        use rand::Rng;
        let min_ms = self.election_timeout_min.as_millis() as u64;
        let max_ms = self.election_timeout_max.as_millis() as u64;
        let timeout_ms = rand::thread_rng().gen_range(min_ms..=max_ms);
        Duration::from_millis(timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RaftConfig::default();
        assert_eq!(config.node_id, 1);
        assert!(config.peers.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = RaftConfig::new(1, vec![2, 3]);
        assert!(config.validate().is_ok());

        // Test invalid node ID
        config.node_id = 0;
        assert!(config.validate().is_err());

        // Test self in peers
        config.node_id = 1;
        config.peers = vec![1, 2];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cluster_calculations() {
        let config = RaftConfig::new(1, vec![2, 3, 4]);
        assert_eq!(config.cluster_size(), 4);
        assert_eq!(config.quorum_size(), 3);
        assert_eq!(config.all_nodes(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_membership_check() {
        let config = RaftConfig::new(1, vec![2, 3]);
        assert!(config.is_member(1));
        assert!(config.is_member(2));
        assert!(config.is_member(3));
        assert!(!config.is_member(4));
    }

    #[test]
    fn test_random_election_timeout() {
        let config = RaftConfig::new(1, vec![2, 3]);
        for _ in 0..100 {
            let timeout = config.random_election_timeout();
            assert!(timeout >= config.election_timeout_min);
            assert!(timeout <= config.election_timeout_max);
        }
    }

    #[test]
    fn test_builder_pattern() {
        let config = RaftConfig::new(1, vec![2, 3])
            .with_heartbeat_interval(Duration::from_millis(25))
            .with_election_timeout(Duration::from_millis(100), Duration::from_millis(200))
            .with_max_entries_per_request(50)
            .with_pre_vote(true)
            .with_batch_optimization(false);

        assert_eq!(config.heartbeat_interval, Duration::from_millis(25));
        assert_eq!(config.election_timeout_min, Duration::from_millis(100));
        assert_eq!(config.max_entries_per_request, 50);
        assert!(config.enable_pre_vote);
        assert!(!config.enable_batch_optimization);
        assert!(config.validate().is_ok());
    }
}