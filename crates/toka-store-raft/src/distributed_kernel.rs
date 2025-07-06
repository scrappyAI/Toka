//! Distributed kernel coordinator using Raft consensus.
//!
//! This module provides a wrapper around the Toka kernel that ensures
//! consistent message processing across a distributed cluster using Raft
//! consensus. All kernel operations are coordinated through the Raft
//! leader to maintain deterministic execution.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::{RwLock, oneshot};
use tokio::time::timeout;
use uuid::Uuid;
use chrono::Utc;

use toka_types::{Message, EntityId};
use toka_bus_core::{EventBus, KernelEvent};
use toka_kernel::{Kernel, WorldState};
use toka_auth::TokenValidator;

use crate::config::RaftClusterConfig;
use crate::storage::RaftStorage;
use crate::error::{RaftStorageError, RaftStorageResult};
use crate::{TokaOperation, TokaOperationResult, ClusterTopology, NodeInfo, NodeStatus};

/// Distributed kernel coordinator that uses Raft consensus.
///
/// This coordinator wraps the regular Toka kernel and ensures that all
/// message processing is coordinated through Raft consensus. Only the
/// leader node processes messages, while follower nodes stay in sync
/// through the Raft protocol.
pub struct DistributedKernel {
    /// Local kernel instance
    kernel: Arc<Kernel>,
    
    /// Raft storage backend for consensus
    raft_storage: Arc<RaftStorage>,
    
    /// Cluster configuration
    cluster_config: RaftClusterConfig,
    
    /// Node ID in the cluster
    node_id: u64,
    
    /// Current leader information
    current_leader: Arc<RwLock<Option<u64>>>,
    
    /// Cluster topology tracking
    cluster_topology: Arc<RwLock<ClusterTopology>>,
    
    /// Shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl DistributedKernel {
    /// Create a new distributed kernel coordinator.
    pub async fn new(
        world_state: WorldState,
        auth: Arc<dyn TokenValidator>,
        event_bus: Arc<dyn EventBus>,
        cluster_config: RaftClusterConfig,
    ) -> RaftStorageResult<Self> {
        // Validate cluster configuration
        cluster_config.validate()?;
        
        let node_id = cluster_config.node_id;
        
        // Create local kernel
        let kernel = Arc::new(Kernel::new(world_state, auth, event_bus.clone()));
        
        // Create Raft storage backend
        let raft_storage = Arc::new(RaftStorage::new(cluster_config.clone(), event_bus.clone()).await?);
        
        // Initialize cluster topology
        let mut topology = ClusterTopology {
            nodes: std::collections::HashMap::new(),
            leader: None,
            term: 0,
        };
        
        // Add all cluster nodes to topology
        for (peer_id, peer_address) in &cluster_config.peers {
            topology.nodes.insert(*peer_id, NodeInfo {
                id: *peer_id,
                address: peer_address.clone(),
                status: NodeStatus::Unknown,
                last_seen: Utc::now(),
            });
        }
        
        // Add self to topology
        topology.nodes.insert(node_id, NodeInfo {
            id: node_id,
            address: cluster_config.bind_address.clone(),
            status: NodeStatus::Active,
            last_seen: Utc::now(),
        });
        
        Ok(Self {
            kernel,
            raft_storage,
            cluster_config,
            node_id,
            current_leader: Arc::new(RwLock::new(None)),
            cluster_topology: Arc::new(RwLock::new(topology)),
            shutdown_tx: None,
        })
    }
    
    /// Start the distributed kernel coordinator.
    ///
    /// This will start the Raft consensus protocol and begin accepting
    /// messages for processing.
    pub async fn start(&mut self) -> RaftStorageResult<()> {
        tracing::info!("Starting distributed kernel on node {}", self.node_id);
        
        // Start Raft storage backend
        self.raft_storage.start().await?;
        
        // Start background tasks for leader monitoring and health checks
        self.start_background_tasks().await?;
        
        tracing::info!("Distributed kernel started successfully on node {}", self.node_id);
        Ok(())
    }
    
    /// Submit a message for distributed processing.
    ///
    /// This method ensures the message is processed consistently across
    /// the cluster using Raft consensus.
    pub async fn submit_message(&self, message: Message) -> RaftStorageResult<KernelEvent> {
        // Check if we're the leader or need to forward
        let leader_id = {
            let leader = self.current_leader.read().await;
            *leader
        };
        
        if leader_id != Some(self.node_id) {
            // We're not the leader, forward to leader or return error
            if let Some(leader) = leader_id {
                return self.forward_to_leader(leader, message).await;
            } else {
                return Err(RaftStorageError::NoLeader);
            }
        }
        
        // We're the leader, process through Raft consensus
        tracing::debug!("Processing message from {} as leader", message.origin.0);
        
        // Convert message to Raft operation
        let operation = TokaOperation::ProcessMessage {
            message: message.clone(),
            request_id: Uuid::new_v4(),
        };
        
        // Submit operation through Raft consensus
        let result = self.raft_storage.consensus_submit(operation).await?;
        
        // Extract the kernel event from the result
        match result {
            TokaOperationResult::MessageProcessed { event } => Ok(event),
            _ => Err(RaftStorageError::UnexpectedResult),
        }
    }
    
    /// Get the current cluster topology.
    pub async fn get_cluster_topology(&self) -> ClusterTopology {
        self.cluster_topology.read().await.clone()
    }
    
    /// Check if this node is the current leader.
    pub async fn is_leader(&self) -> bool {
        let leader = self.current_leader.read().await;
        *leader == Some(self.node_id)
    }
    
    /// Get the current leader node ID.
    pub async fn get_leader(&self) -> Option<u64> {
        *self.current_leader.read().await
    }
    
    /// Get access to the underlying kernel for read-only operations.
    pub fn kernel(&self) -> Arc<Kernel> {
        self.kernel.clone()
    }
    
    /// Shutdown the distributed kernel gracefully.
    pub async fn shutdown(mut self) -> RaftStorageResult<()> {
        tracing::info!("Shutting down distributed kernel on node {}", self.node_id);
        
        // Send shutdown signal to background tasks
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        
        // Shutdown Raft storage
        self.raft_storage.shutdown().await?;
        
        tracing::info!("Distributed kernel shutdown complete on node {}", self.node_id);
        Ok(())
    }
    
    /// Process a message locally (called by Raft state machine).
    pub(crate) async fn process_message_locally(&self, message: Message) -> RaftStorageResult<KernelEvent> {
        tracing::debug!("Processing message locally: {:?}", message);
        
        // Process the message through the local kernel
        let event = self.kernel.submit(message).await
            .map_err(|e| RaftStorageError::KernelOperation(e.to_string()))?;
        
        Ok(event)
    }
    
    /// Forward a message to the leader node.
    async fn forward_to_leader(&self, leader_id: u64, message: Message) -> RaftStorageResult<KernelEvent> {
        // Get leader address from topology
        let leader_address = {
            let topology = self.cluster_topology.read().await;
            topology.nodes.get(&leader_id)
                .map(|node| node.address.clone())
                .ok_or(RaftStorageError::LeaderNotFound)?
        };
        
        tracing::debug!("Forwarding message to leader {} at {}", leader_id, leader_address);
        
        // For now, return an error - actual forwarding would require network implementation
        // TODO: Implement message forwarding to leader
        Err(RaftStorageError::MessageForwarding("Message forwarding not yet implemented".to_string()))
    }
    
    /// Start background tasks for monitoring and maintenance.
    async fn start_background_tasks(&mut self) -> RaftStorageResult<()> {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        
        let current_leader = self.current_leader.clone();
        let cluster_topology = self.cluster_topology.clone();
        let raft_storage = self.raft_storage.clone();
        let node_id = self.node_id;
        
        // Start leader monitoring task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check current leader from Raft storage
                        if let Ok(leader) = raft_storage.current_leader().await {
                            let mut current = current_leader.write().await;
                            if *current != leader {
                                tracing::info!("Leader changed from {:?} to {:?}", *current, leader);
                                *current = leader;
                                
                                // Update topology
                                let mut topology = cluster_topology.write().await;
                                topology.leader = leader;
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        tracing::debug!("Leader monitoring task shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
}

/// Configuration for distributed kernel setup.
#[derive(Debug, Clone)]
pub struct DistributedKernelConfig {
    /// Cluster configuration for Raft
    pub cluster: RaftClusterConfig,
    
    /// Timeout for message processing
    pub message_timeout: Duration,
    
    /// Maximum number of pending operations
    pub max_pending_operations: usize,
    
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for DistributedKernelConfig {
    fn default() -> Self {
        Self {
            cluster: RaftClusterConfig::default(),
            message_timeout: Duration::from_secs(30),
            max_pending_operations: 1000,
            health_check_interval: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_auth::MockTokenValidator;
    use toka_bus_core::InMemoryBus;
    use toka_types::{Operation, TaskSpec};
    
    #[tokio::test]
    async fn test_distributed_kernel_creation() {
        let world_state = WorldState::default();
        let auth = Arc::new(MockTokenValidator::new());
        let event_bus = Arc::new(InMemoryBus::new(100));
        let cluster_config = RaftClusterConfig::new(1)
            .add_peer(2, "127.0.0.1:8081".to_string())
            .add_peer(3, "127.0.0.1:8082".to_string());
        
        let _kernel = DistributedKernel::new(
            world_state,
            auth,
            event_bus,
            cluster_config,
        ).await.unwrap();
        
        // Basic creation test passes
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_cluster_topology() {
        let world_state = WorldState::default();
        let auth = Arc::new(MockTokenValidator::new());
        let event_bus = Arc::new(InMemoryBus::new(100));
        let cluster_config = RaftClusterConfig::new(1)
            .add_peer(2, "127.0.0.1:8081".to_string())
            .add_peer(3, "127.0.0.1:8082".to_string());
        
        let kernel = DistributedKernel::new(
            world_state,
            auth,
            event_bus,
            cluster_config,
        ).await.unwrap();
        
        let topology = kernel.get_cluster_topology().await;
        assert_eq!(topology.nodes.len(), 3); // Self + 2 peers
        assert!(topology.nodes.contains_key(&1));
        assert!(topology.nodes.contains_key(&2));
        assert!(topology.nodes.contains_key(&3));
    }
} 