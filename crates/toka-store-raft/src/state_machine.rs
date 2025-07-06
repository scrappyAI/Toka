//! Raft state machine implementation for Toka operations.
//!
//! This module provides the `TokaStateMachine` which implements the Raft `StateMachine` trait
//! to handle consensus operations and apply them to the underlying Toka storage backend.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use raft_core::{LogEntry, StateMachine, RaftResult, RaftError};
use toka_store_core::{StorageBackend, EventHeader, EventId, CausalDigest};
use toka_bus_core::{EventBus, KernelEvent};

use crate::{TokaOperation, TokaOperationResult, RaftMetrics};
use crate::error::{RaftStorageError, RaftStorageResult};

/// Raft state machine for Toka operations
///
/// This state machine handles the application of Raft log entries to the underlying
/// Toka storage backend. It ensures that all operations are applied consistently
/// across the cluster and provides mechanisms for snapshotting and recovery.
pub struct TokaStateMachine {
    /// Underlying storage backend
    storage: Arc<dyn StorageBackend>,
    
    /// Event bus for publishing events
    event_bus: Arc<dyn EventBus>,
    
    /// Last applied log index
    last_applied: Arc<RwLock<u64>>,
    
    /// Snapshot of committed events (for recovery)
    committed_events: Arc<RwLock<HashMap<EventId, EventHeader>>>,
    
    /// Metrics tracking
    metrics: Arc<RwLock<RaftMetrics>>,
    
    /// Performance tracking
    operation_times: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    
    /// Node ID for logging
    node_id: u64,
}

impl TokaStateMachine {
    /// Create a new Toka state machine
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        event_bus: Arc<dyn EventBus>,
        node_id: u64,
    ) -> Self {
        Self {
            storage,
            event_bus,
            last_applied: Arc::new(RwLock::new(0)),
            committed_events: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(RaftMetrics::default())),
            operation_times: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        }
    }
    
    /// Get the last applied log index
    pub async fn last_applied_index(&self) -> u64 {
        *self.last_applied.read().await
    }
    
    /// Get current metrics
    pub async fn metrics(&self) -> RaftMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Get committed events snapshot
    pub async fn committed_events_snapshot(&self) -> HashMap<EventId, EventHeader> {
        self.committed_events.read().await.clone()
    }
    
    /// Apply a Toka operation and return the result
    async fn apply_toka_operation(&self, operation: TokaOperation) -> RaftStorageResult<TokaOperationResult> {
        let start_time = Instant::now();
        
        let result = match operation {
            TokaOperation::CommitEvent { header, payload } => {
                self.apply_commit_event(header, payload).await?
            }
            TokaOperation::CompactLog { before_index } => {
                self.apply_compact_log(before_index).await?
            }
            TokaOperation::TakeSnapshot => {
                self.apply_take_snapshot().await?
            }
            TokaOperation::InstallSnapshot { data, last_included_index, last_included_term } => {
                self.apply_install_snapshot(data, last_included_index, last_included_term).await?
            }
        };
        
        // Record operation timing
        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
        let operation_name = match &result {
            TokaOperationResult::EventCommitted { .. } => "commit_event",
            TokaOperationResult::LogCompacted { .. } => "compact_log",
            TokaOperationResult::SnapshotTaken { .. } => "take_snapshot",
            TokaOperationResult::SnapshotInstalled { .. } => "install_snapshot",
            TokaOperationResult::Failed { .. } => "failed_operation",
        };
        
        let mut times = self.operation_times.write().await;
        times.entry(operation_name.to_string()).or_insert_with(Vec::new).push(elapsed);
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.consensus_throughput = self.calculate_throughput(&times).await;
        
        debug!("Applied operation {} in {:.2}ms", operation_name, elapsed);
        
        Ok(result)
    }
    
    /// Apply a commit event operation
    async fn apply_commit_event(&self, header: EventHeader, payload: Vec<u8>) -> RaftStorageResult<TokaOperationResult> {
        // Store the event in the backend
        self.storage.commit(&header, &payload).await
            .map_err(|e| RaftStorageError::StorageBackend(e))?;
        
        // Update our tracking
        {
            let mut events = self.committed_events.write().await;
            events.insert(header.event_id, header.clone());
        }
        
        // Publish event to the bus
        let event = KernelEvent::EventCommitted {
            event_id: header.event_id,
        };
        
        if let Err(e) = self.event_bus.publish(&event) {
            warn!("Failed to publish event to bus: {}", e);
        }
        
        info!("Node {} committed event {}", self.node_id, header.event_id);
        
        Ok(TokaOperationResult::EventCommitted {
            event_id: header.event_id,
        })
    }
    
    /// Apply a log compaction operation
    async fn apply_compact_log(&self, before_index: u64) -> RaftStorageResult<TokaOperationResult> {
        // For now, we'll just clean up old events from our tracking
        // In a full implementation, this would involve actual log compaction
        let mut events = self.committed_events.write().await;
        let initial_count = events.len();
        
        // Remove events that are old (this is a simplified implementation)
        // In practice, you'd want more sophisticated compaction logic
        events.retain(|_, header| {
            // Keep recent events (this is a placeholder logic)
            header.timestamp > chrono::Utc::now() - chrono::Duration::hours(1)
        });
        
        let final_count = events.len();
        let entries_removed = initial_count - final_count;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.log_size = final_count as u64;
        }
        
        info!("Node {} compacted {} log entries", self.node_id, entries_removed);
        
        Ok(TokaOperationResult::LogCompacted {
            entries_removed: entries_removed as u64,
        })
    }
    
    /// Apply a take snapshot operation
    async fn apply_take_snapshot(&self) -> RaftStorageResult<TokaOperationResult> {
        // Create a snapshot of the current state
        let events = self.committed_events.read().await;
        let snapshot_data = bincode::serialize(&*events)
            .map_err(|e| RaftStorageError::Serialization(e))?;
        
        let snapshot_size = snapshot_data.len();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.snapshots_taken += 1;
        }
        
        info!("Node {} took snapshot of {} bytes", self.node_id, snapshot_size);
        
        Ok(TokaOperationResult::SnapshotTaken {
            snapshot_size,
        })
    }
    
    /// Apply an install snapshot operation
    async fn apply_install_snapshot(
        &self,
        data: Vec<u8>,
        last_included_index: u64,
        _last_included_term: u64,
    ) -> RaftStorageResult<TokaOperationResult> {
        // Deserialize the snapshot
        let events: HashMap<EventId, EventHeader> = bincode::deserialize(&data)
            .map_err(|e| RaftStorageError::Serialization(e))?;
        
        // Replace our current state with the snapshot
        {
            let mut current_events = self.committed_events.write().await;
            *current_events = events;
        }
        
        // Update last applied index
        {
            let mut last_applied = self.last_applied.write().await;
            *last_applied = last_included_index;
        }
        
        info!("Node {} installed snapshot up to index {}", self.node_id, last_included_index);
        
        Ok(TokaOperationResult::SnapshotInstalled {
            last_included_index,
        })
    }
    
    /// Calculate current throughput based on operation times
    async fn calculate_throughput(&self, times: &HashMap<String, Vec<f64>>) -> f64 {
        let total_operations: usize = times.values().map(|v| v.len()).sum();
        let total_time: f64 = times.values().flat_map(|v| v.iter()).sum();
        
        if total_time > 0.0 {
            (total_operations as f64) / (total_time / 1000.0) // ops per second
        } else {
            0.0
        }
    }
    
    /// Serialize operation for storage
    fn serialize_operation(&self, operation: &TokaOperation) -> RaftStorageResult<Vec<u8>> {
        bincode::serialize(operation)
            .map_err(|e| RaftStorageError::Serialization(e))
    }
    
    /// Deserialize operation from storage
    fn deserialize_operation(&self, data: &[u8]) -> RaftStorageResult<TokaOperation> {
        bincode::deserialize(data)
            .map_err(|e| RaftStorageError::Serialization(e))
    }
}

#[async_trait]
impl StateMachine for TokaStateMachine {
    async fn apply(&mut self, entry: &LogEntry) -> RaftResult<Vec<u8>> {
        debug!("Applying log entry {} on node {}", entry.index, self.node_id);
        
        // Update last applied index
        {
            let mut last_applied = self.last_applied.write().await;
            *last_applied = entry.index;
        }
        
        // Deserialize the operation
        let operation = self.deserialize_operation(&entry.data)
            .map_err(|e| RaftError::other(format!("Failed to deserialize operation: {}", e)))?;
        
        // Apply the operation
        let result = self.apply_toka_operation(operation).await
            .map_err(|e| RaftError::other(format!("Failed to apply operation: {}", e)))?;
        
        // Serialize the result
        let result_bytes = bincode::serialize(&result)
            .map_err(|e| RaftError::other(format!("Failed to serialize result: {}", e)))?;
        
        debug!("Successfully applied log entry {} on node {}", entry.index, self.node_id);
        
        Ok(result_bytes)
    }
    
    async fn take_snapshot(&mut self) -> RaftResult<Vec<u8>> {
        info!("Taking snapshot on node {}", self.node_id);
        
        // Create snapshot of current state
        let events = self.committed_events.read().await;
        let last_applied = *self.last_applied.read().await;
        
        let snapshot = SnapshotData {
            committed_events: events.clone(),
            last_applied,
            timestamp: chrono::Utc::now(),
        };
        
        let snapshot_bytes = bincode::serialize(&snapshot)
            .map_err(|e| RaftError::other(format!("Failed to serialize snapshot: {}", e)))?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.snapshots_taken += 1;
        }
        
        info!("Snapshot taken on node {}: {} bytes", self.node_id, snapshot_bytes.len());
        
        Ok(snapshot_bytes)
    }
    
    async fn restore_from_snapshot(&mut self, snapshot: &[u8]) -> RaftResult<()> {
        info!("Restoring from snapshot on node {}", self.node_id);
        
        // Deserialize snapshot
        let snapshot_data: SnapshotData = bincode::deserialize(snapshot)
            .map_err(|e| RaftError::other(format!("Failed to deserialize snapshot: {}", e)))?;
        
        // Restore state
        {
            let mut events = self.committed_events.write().await;
            *events = snapshot_data.committed_events;
        }
        
        {
            let mut last_applied = self.last_applied.write().await;
            *last_applied = snapshot_data.last_applied;
        }
        
        info!("Restored from snapshot on node {}: {} events, last applied: {}", 
               self.node_id, 
               snapshot_data.committed_events.len(), 
               snapshot_data.last_applied);
        
        Ok(())
    }
}

/// Data structure for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotData {
    /// Map of committed events
    committed_events: HashMap<EventId, EventHeader>,
    /// Last applied log index
    last_applied: u64,
    /// Timestamp when snapshot was taken
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use toka_store_core::EventHeader;
    use toka_bus_core::InMemoryBus;
    use toka_store_memory::MemoryStorage;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_state_machine_creation() {
        let storage = Arc::new(MemoryStorage::new());
        let event_bus = Arc::new(InMemoryBus::new(100));
        let state_machine = TokaStateMachine::new(storage, event_bus, 1);
        
        assert_eq!(state_machine.last_applied_index().await, 0);
        assert_eq!(state_machine.committed_events_snapshot().await.len(), 0);
    }
    
    #[tokio::test]
    async fn test_commit_event_operation() {
        let storage = Arc::new(MemoryStorage::new());
        let event_bus = Arc::new(InMemoryBus::new(100));
        let state_machine = TokaStateMachine::new(storage, event_bus, 1);
        
        let event_id = Uuid::new_v4();
        let header = EventHeader {
            event_id,
            intent_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            causal_digest: [0u8; 32],
        };
        
        let operation = TokaOperation::CommitEvent {
            header: header.clone(),
            payload: b"test payload".to_vec(),
        };
        
        let result = state_machine.apply_toka_operation(operation).await.unwrap();
        
        match result {
            TokaOperationResult::EventCommitted { event_id: result_id } => {
                assert_eq!(result_id, event_id);
            }
            _ => panic!("Expected EventCommitted result"),
        }
        
        let events = state_machine.committed_events_snapshot().await;
        assert_eq!(events.len(), 1);
        assert!(events.contains_key(&event_id));
    }
    
    #[tokio::test]
    async fn test_snapshot_operations() {
        let storage = Arc::new(MemoryStorage::new());
        let event_bus = Arc::new(InMemoryBus::new(100));
        let mut state_machine = TokaStateMachine::new(storage, event_bus, 1);
        
        // Add some events first
        let event_id = Uuid::new_v4();
        let header = EventHeader {
            event_id,
            intent_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            causal_digest: [0u8; 32],
        };
        
        let operation = TokaOperation::CommitEvent {
            header: header.clone(),
            payload: b"test payload".to_vec(),
        };
        
        state_machine.apply_toka_operation(operation).await.unwrap();
        
        // Take snapshot
        let snapshot = state_machine.take_snapshot().await.unwrap();
        assert!(!snapshot.is_empty());
        
        // Clear state
        {
            let mut events = state_machine.committed_events.write().await;
            events.clear();
        }
        
        // Restore from snapshot
        state_machine.restore_from_snapshot(&snapshot).await.unwrap();
        
        let events = state_machine.committed_events_snapshot().await;
        assert_eq!(events.len(), 1);
        assert!(events.contains_key(&event_id));
    }
} 