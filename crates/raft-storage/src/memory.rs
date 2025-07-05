//! In-memory storage implementation for Raft.
//! 
//! This implementation stores all data in memory and is primarily useful for
//! testing and development. Data is not persisted across restarts.

use crate::error::{StorageError, StorageResult};
use crate::storage::{
    IntegrityIssue, IntegrityIssueType, IntegrityReport, IssueSeverity, MaintenanceOperation,
    MaintenanceReport, Storage, StorageMetrics,
};
use crate::{PersistentState, StoredLogEntry};
use async_trait::async_trait;
use raft_core::{LogEntry, LogIndex, Term};
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// In-memory storage implementation
pub struct MemoryStorage {
    /// Stored persistent state
    persistent_state: Arc<RwLock<Option<PersistentState>>>,
    
    /// Log entries indexed by log index
    log_entries: Arc<RwLock<BTreeMap<LogIndex, StoredLogEntry>>>,
    
    /// Latest snapshot data
    snapshot: Arc<RwLock<Option<(Vec<u8>, LogIndex, Term)>>>,
    
    /// Storage metrics
    metrics: Arc<RwLock<StorageMetrics>>,
    
    /// Start time for calculating durations
    start_time: Instant,
}

impl MemoryStorage {
    /// Create a new in-memory storage instance
    pub fn new() -> Self {
        Self {
            persistent_state: Arc::new(RwLock::new(None)),
            log_entries: Arc::new(RwLock::new(BTreeMap::new())),
            snapshot: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(StorageMetrics {
                implementation: "memory".to_string(),
                ..Default::default()
            })),
            start_time: Instant::now(),
        }
    }

    /// Create a new instance with initial state
    pub fn with_state(initial_state: PersistentState) -> Self {
        Self {
            persistent_state: Arc::new(RwLock::new(Some(initial_state))),
            log_entries: Arc::new(RwLock::new(BTreeMap::new())),
            snapshot: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(StorageMetrics {
                implementation: "memory".to_string(),
                ..Default::default()
            })),
            start_time: Instant::now(),
        }
    }

    /// Update read metrics
    async fn record_read(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.read_operations += 1;
        
        // Update average latency using exponential moving average
        let latency_us = duration.as_micros() as f64;
        if metrics.read_operations == 1 {
            metrics.avg_read_latency_us = latency_us;
        } else {
            metrics.avg_read_latency_us = 
                0.9 * metrics.avg_read_latency_us + 0.1 * latency_us;
        }
    }

    /// Update write metrics
    async fn record_write(&self, duration: Duration, bytes_written: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.write_operations += 1;
        metrics.total_size_bytes += bytes_written as u64;
        
        // Update average latency using exponential moving average
        let latency_us = duration.as_micros() as f64;
        if metrics.write_operations == 1 {
            metrics.avg_write_latency_us = latency_us;
        } else {
            metrics.avg_write_latency_us = 
                0.9 * metrics.avg_write_latency_us + 0.1 * latency_us;
        }
    }

    /// Update error metrics
    async fn record_error(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.error_count += 1;
    }

    /// Calculate total size of stored data
    async fn calculate_total_size(&self) -> u64 {
        let log_entries = self.log_entries.read().await;
        let snapshot = self.snapshot.read().await;
        let persistent_state = self.persistent_state.read().await;
        
        let mut total_size = 0u64;
        
        // Size of log entries
        for stored_entry in log_entries.values() {
            total_size += std::mem::size_of_val(stored_entry) as u64;
            total_size += stored_entry.entry.data.len() as u64;
        }
        
        // Size of snapshot
        if let Some((data, _, _)) = snapshot.as_ref() {
            total_size += data.len() as u64;
        }
        
        // Size of persistent state (estimated)
        if persistent_state.is_some() {
            total_size += 1024; // Rough estimate
        }
        
        total_size
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn store_persistent_state(&mut self, state: &PersistentState) -> StorageResult<()> {
        let start = Instant::now();
        
        let mut persistent_state = self.persistent_state.write().await;
        *persistent_state = Some(state.clone());
        
        let duration = start.elapsed();
        let serialized_size = serde_json::to_vec(state)
            .map(|v| v.len())
            .unwrap_or(0);
        
        self.record_write(duration, serialized_size).await;
        
        Ok(())
    }

    async fn load_persistent_state(&self) -> StorageResult<Option<PersistentState>> {
        let start = Instant::now();
        
        let persistent_state = self.persistent_state.read().await;
        let result = persistent_state.clone();
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn store_log_entry(&mut self, entry: &LogEntry) -> StorageResult<()> {
        let start = Instant::now();
        
        let stored_entry = StoredLogEntry::new(entry.clone());
        let mut log_entries = self.log_entries.write().await;
        log_entries.insert(entry.index, stored_entry);
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_log_entries = log_entries.len() as u64;
        metrics.log_size_bytes = log_entries.len() as u64 * 1024; // Rough estimate
        drop(metrics);
        drop(log_entries);
        
        let duration = start.elapsed();
        let entry_size = std::mem::size_of_val(entry) + entry.data.len();
        self.record_write(duration, entry_size).await;
        
        Ok(())
    }

    async fn store_log_entries(&mut self, entries: &[LogEntry]) -> StorageResult<()> {
        let start = Instant::now();
        
        let mut log_entries = self.log_entries.write().await;
        let mut total_size = 0;
        
        for entry in entries {
            let stored_entry = StoredLogEntry::new(entry.clone());
            total_size += std::mem::size_of_val(entry) + entry.data.len();
            log_entries.insert(entry.index, stored_entry);
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_log_entries = log_entries.len() as u64;
        metrics.log_size_bytes = log_entries.len() as u64 * 1024; // Rough estimate
        drop(metrics);
        drop(log_entries);
        
        let duration = start.elapsed();
        self.record_write(duration, total_size).await;
        
        Ok(())
    }

    async fn get_log_entry(&self, index: LogIndex) -> StorageResult<Option<LogEntry>> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let result = log_entries.get(&index).map(|stored| {
            if stored.verify_integrity() {
                stored.entry.clone()
            } else {
                // Record corruption but still return the entry
                // In a real implementation, you might want to handle this differently
                stored.entry.clone()
            }
        });
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn get_log_entries(&self, range: Range<LogIndex>) -> StorageResult<Vec<LogEntry>> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let mut result = Vec::new();
        
        for index in range {
            if let Some(stored_entry) = log_entries.get(&index) {
                if stored_entry.verify_integrity() {
                    result.push(stored_entry.entry.clone());
                } else {
                    return Err(StorageError::corruption(format!(
                        "Corrupted entry at index {}",
                        index
                    )));
                }
            } else {
                break; // Stop on first missing entry
            }
        }
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn first_log_index(&self) -> StorageResult<LogIndex> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let result = log_entries.keys().next().copied().unwrap_or(1);
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn last_log_index(&self) -> StorageResult<LogIndex> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let result = log_entries.keys().next_back().copied().unwrap_or(0);
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn get_log_term(&self, index: LogIndex) -> StorageResult<Option<Term>> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let result = log_entries.get(&index).map(|stored| stored.entry.term);
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn truncate_log_from(&mut self, index: LogIndex) -> StorageResult<()> {
        let start = Instant::now();
        
        let mut log_entries = self.log_entries.write().await;
        let keys_to_remove: Vec<LogIndex> = log_entries
            .range(index..)
            .map(|(k, _)| *k)
            .collect();
        
        for key in keys_to_remove {
            log_entries.remove(&key);
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_log_entries = log_entries.len() as u64;
        metrics.log_size_bytes = log_entries.len() as u64 * 1024; // Rough estimate
        drop(metrics);
        drop(log_entries);
        
        let duration = start.elapsed();
        self.record_write(duration, 0).await;
        
        Ok(())
    }

    async fn compact_log_to(&mut self, index: LogIndex) -> StorageResult<()> {
        let start = Instant::now();
        
        let mut log_entries = self.log_entries.write().await;
        let keys_to_remove: Vec<LogIndex> = log_entries
            .range(..=index)
            .map(|(k, _)| *k)
            .collect();
        
        for key in keys_to_remove {
            log_entries.remove(&key);
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_log_entries = log_entries.len() as u64;
        metrics.log_size_bytes = log_entries.len() as u64 * 1024; // Rough estimate
        drop(metrics);
        drop(log_entries);
        
        let duration = start.elapsed();
        self.record_write(duration, 0).await;
        
        Ok(())
    }

    async fn store_snapshot(
        &mut self,
        snapshot: &[u8],
        last_included_index: LogIndex,
        last_included_term: Term,
    ) -> StorageResult<()> {
        let start = Instant::now();
        
        let mut stored_snapshot = self.snapshot.write().await;
        *stored_snapshot = Some((snapshot.to_vec(), last_included_index, last_included_term));
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.snapshot_count = 1;
        metrics.snapshot_size_bytes = snapshot.len() as u64;
        drop(metrics);
        drop(stored_snapshot);
        
        let duration = start.elapsed();
        self.record_write(duration, snapshot.len()).await;
        
        Ok(())
    }

    async fn load_snapshot(&self) -> StorageResult<Option<(Vec<u8>, LogIndex, Term)>> {
        let start = Instant::now();
        
        let snapshot = self.snapshot.read().await;
        let result = snapshot.clone();
        
        let duration = start.elapsed();
        self.record_read(duration).await;
        
        Ok(result)
    }

    async fn cleanup_snapshots(&mut self) -> StorageResult<()> {
        // For memory storage, we only keep one snapshot, so nothing to clean up
        Ok(())
    }

    async fn metrics(&self) -> StorageResult<StorageMetrics> {
        let mut metrics = self.metrics.read().await.clone();
        
        // Update total size
        metrics.total_size_bytes = self.calculate_total_size().await;
        
        Ok(metrics)
    }

    async fn sync(&mut self) -> StorageResult<()> {
        let start = Instant::now();
        
        // For memory storage, sync is a no-op, but we record the operation
        let mut metrics = self.metrics.write().await;
        metrics.sync_operations += 1;
        
        let duration = start.elapsed();
        self.record_write(duration, 0).await;
        
        Ok(())
    }

    async fn verify_integrity(&self) -> StorageResult<IntegrityReport> {
        let start = Instant::now();
        
        let log_entries = self.log_entries.read().await;
        let snapshot = self.snapshot.read().await;
        
        let mut issues = Vec::new();
        let mut entries_checked = 0u64;
        let snapshots_checked = if snapshot.is_some() { 1 } else { 0 };
        
        // Check log entry integrity
        let mut last_term = 0;
        for (index, stored_entry) in log_entries.iter() {
            entries_checked += 1;
            
            // Check checksum
            if !stored_entry.verify_integrity() {
                issues.push(IntegrityIssue {
                    issue_type: IntegrityIssueType::ChecksumMismatch,
                    description: format!("Checksum mismatch for entry at index {}", index),
                    log_index: Some(*index),
                    severity: IssueSeverity::Error,
                });
            }
            
            // Check term sequence
            if stored_entry.entry.term < last_term {
                issues.push(IntegrityIssue {
                    issue_type: IntegrityIssueType::InvalidTermSequence,
                    description: format!(
                        "Term {} at index {} is less than previous term {}",
                        stored_entry.entry.term, index, last_term
                    ),
                    log_index: Some(*index),
                    severity: IssueSeverity::Warning,
                });
            }
            
            last_term = stored_entry.entry.term;
        }
        
        // Check for gaps in log indices
        let indices: Vec<LogIndex> = log_entries.keys().copied().collect();
        for i in 1..indices.len() {
            if indices[i] != indices[i - 1] + 1 {
                issues.push(IntegrityIssue {
                    issue_type: IntegrityIssueType::MissingEntry,
                    description: format!(
                        "Gap in log indices: {} followed by {}",
                        indices[i - 1],
                        indices[i]
                    ),
                    log_index: Some(indices[i - 1] + 1),
                    severity: IssueSeverity::Warning,
                });
            }
        }
        
        let duration = start.elapsed();
        let is_valid = issues.iter().all(|issue| issue.severity < IssueSeverity::Error);
        
        Ok(IntegrityReport {
            is_valid,
            issues,
            entries_checked,
            snapshots_checked,
            verification_duration: duration,
        })
    }

    async fn maintenance(&mut self) -> StorageResult<MaintenanceReport> {
        let start = Instant::now();
        
        // For memory storage, maintenance is mostly a no-op
        let operations_performed = vec![];
        let space_reclaimed = 0;
        
        // Update maintenance timestamp
        let mut metrics = self.metrics.write().await;
        metrics.last_maintenance = Some(chrono::Utc::now());
        
        let duration = start.elapsed();
        
        Ok(MaintenanceReport {
            operations_performed,
            space_reclaimed,
            maintenance_duration: duration,
            issues: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raft_core::LogEntry;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let mut storage = MemoryStorage::new();
        
        // Test storing and loading persistent state
        let state = PersistentState {
            current_term: 5,
            voted_for: Some(1),
            last_applied: 10,
            cluster_config: Default::default(),
        };
        
        storage.store_persistent_state(&state).await.unwrap();
        let loaded_state = storage.load_persistent_state().await.unwrap();
        assert_eq!(loaded_state, Some(state));
    }

    #[tokio::test]
    async fn test_log_entry_operations() {
        let mut storage = MemoryStorage::new();
        
        // Store some log entries
        let entry1 = LogEntry::new_command(1, 1, b"command1".to_vec());
        let entry2 = LogEntry::new_command(1, 2, b"command2".to_vec());
        let entry3 = LogEntry::new_command(2, 3, b"command3".to_vec());
        
        storage.store_log_entry(&entry1).await.unwrap();
        storage.store_log_entry(&entry2).await.unwrap();
        storage.store_log_entry(&entry3).await.unwrap();
        
        // Test retrieval
        let retrieved = storage.get_log_entry(2).await.unwrap();
        assert_eq!(retrieved, Some(entry2.clone()));
        
        // Test range retrieval
        let entries = storage.get_log_entries(1..3).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
        
        // Test indices
        assert_eq!(storage.first_log_index().await.unwrap(), 1);
        assert_eq!(storage.last_log_index().await.unwrap(), 3);
        
        // Test term retrieval
        assert_eq!(storage.get_log_term(3).await.unwrap(), Some(2));
    }

    #[tokio::test]
    async fn test_log_truncation() {
        let mut storage = MemoryStorage::new();
        
        // Store entries
        for i in 1..=5 {
            let entry = LogEntry::new_command(1, i, format!("command{}", i).into_bytes());
            storage.store_log_entry(&entry).await.unwrap();
        }
        
        // Truncate from index 3
        storage.truncate_log_from(3).await.unwrap();
        
        // Check remaining entries
        assert_eq!(storage.last_log_index().await.unwrap(), 2);
        assert!(storage.get_log_entry(3).await.unwrap().is_none());
        assert!(storage.get_log_entry(2).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_log_compaction() {
        let mut storage = MemoryStorage::new();
        
        // Store entries
        for i in 1..=5 {
            let entry = LogEntry::new_command(1, i, format!("command{}", i).into_bytes());
            storage.store_log_entry(&entry).await.unwrap();
        }
        
        // Compact to index 2
        storage.compact_log_to(2).await.unwrap();
        
        // Check remaining entries
        assert_eq!(storage.first_log_index().await.unwrap(), 3);
        assert!(storage.get_log_entry(2).await.unwrap().is_none());
        assert!(storage.get_log_entry(3).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_snapshot_operations() {
        let mut storage = MemoryStorage::new();
        
        let snapshot_data = b"snapshot data".to_vec();
        storage.store_snapshot(&snapshot_data, 10, 2).await.unwrap();
        
        let loaded = storage.load_snapshot().await.unwrap();
        assert_eq!(loaded, Some((snapshot_data, 10, 2)));
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let mut storage = MemoryStorage::new();
        
        // Store some entries
        for i in 1..=3 {
            let entry = LogEntry::new_command(1, i, format!("command{}", i).into_bytes());
            storage.store_log_entry(&entry).await.unwrap();
        }
        
        let report = storage.verify_integrity().await.unwrap();
        assert!(report.is_valid);
        assert_eq!(report.entries_checked, 3);
        assert!(report.issues.is_empty());
    }

    #[tokio::test]
    async fn test_metrics() {
        let mut storage = MemoryStorage::new();
        
        // Store some data
        let entry = LogEntry::new_command(1, 1, b"test".to_vec());
        storage.store_log_entry(&entry).await.unwrap();
        
        // Read the entry to trigger read operations counter
        let retrieved = storage.get_log_entry(1).await.unwrap();
        assert_eq!(retrieved, Some(entry));
        
        let metrics = storage.metrics().await.unwrap();
        assert_eq!(metrics.implementation, "memory");
        assert_eq!(metrics.total_log_entries, 1);
        assert!(metrics.read_operations > 0);
        assert!(metrics.write_operations > 0);
    }
}