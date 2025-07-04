//! Storage trait and related types for Raft consensus.

use crate::error::{StorageError, StorageResult};
use crate::{PersistentState, StoredLogEntry};
use async_trait::async_trait;
use raft_core::{LogEntry, LogIndex, Term};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Main storage trait for Raft consensus algorithm
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store persistent state (term, voted_for, etc.)
    async fn store_persistent_state(&mut self, state: &PersistentState) -> StorageResult<()>;

    /// Load persistent state
    async fn load_persistent_state(&self) -> StorageResult<Option<PersistentState>>;

    /// Store a log entry
    async fn store_log_entry(&mut self, entry: &LogEntry) -> StorageResult<()>;

    /// Store multiple log entries atomically
    async fn store_log_entries(&mut self, entries: &[LogEntry]) -> StorageResult<()>;

    /// Get a log entry by index
    async fn get_log_entry(&self, index: LogIndex) -> StorageResult<Option<LogEntry>>;

    /// Get multiple log entries in the specified range
    async fn get_log_entries(&self, range: Range<LogIndex>) -> StorageResult<Vec<LogEntry>>;

    /// Get the first log index
    async fn first_log_index(&self) -> StorageResult<LogIndex>;

    /// Get the last log index
    async fn last_log_index(&self) -> StorageResult<LogIndex>;

    /// Get the term of a log entry
    async fn get_log_term(&self, index: LogIndex) -> StorageResult<Option<Term>>;

    /// Remove log entries from the specified index onwards
    async fn truncate_log_from(&mut self, index: LogIndex) -> StorageResult<()>;

    /// Remove log entries before the specified index (for compaction)
    async fn compact_log_to(&mut self, index: LogIndex) -> StorageResult<()>;

    /// Store a snapshot
    async fn store_snapshot(&mut self, snapshot: &[u8], last_included_index: LogIndex, last_included_term: Term) -> StorageResult<()>;

    /// Load the latest snapshot
    async fn load_snapshot(&self) -> StorageResult<Option<(Vec<u8>, LogIndex, Term)>>;

    /// Delete old snapshots, keeping only the latest
    async fn cleanup_snapshots(&mut self) -> StorageResult<()>;

    /// Get storage metrics
    async fn metrics(&self) -> StorageResult<StorageMetrics>;

    /// Sync all pending writes to disk
    async fn sync(&mut self) -> StorageResult<()>;

    /// Check storage integrity
    async fn verify_integrity(&self) -> StorageResult<IntegrityReport>;

    /// Perform storage maintenance (cleanup, optimization, etc.)
    async fn maintenance(&mut self) -> StorageResult<MaintenanceReport>;
}

/// Storage performance and usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// Total number of log entries stored
    pub total_log_entries: u64,

    /// Size of log storage in bytes
    pub log_size_bytes: u64,

    /// Number of snapshots stored
    pub snapshot_count: u64,

    /// Size of snapshot storage in bytes
    pub snapshot_size_bytes: u64,

    /// Total storage size in bytes
    pub total_size_bytes: u64,

    /// Number of read operations
    pub read_operations: u64,

    /// Number of write operations
    pub write_operations: u64,

    /// Number of sync operations
    pub sync_operations: u64,

    /// Average read latency in microseconds
    pub avg_read_latency_us: f64,

    /// Average write latency in microseconds
    pub avg_write_latency_us: f64,

    /// Number of storage errors encountered
    pub error_count: u64,

    /// Last maintenance timestamp
    pub last_maintenance: Option<chrono::DateTime<chrono::Utc>>,

    /// Storage implementation name
    pub implementation: String,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            total_log_entries: 0,
            log_size_bytes: 0,
            snapshot_count: 0,
            snapshot_size_bytes: 0,
            total_size_bytes: 0,
            read_operations: 0,
            write_operations: 0,
            sync_operations: 0,
            avg_read_latency_us: 0.0,
            avg_write_latency_us: 0.0,
            error_count: 0,
            last_maintenance: None,
            implementation: "unknown".to_string(),
        }
    }
}

/// Storage integrity verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    /// Whether the storage passed integrity checks
    pub is_valid: bool,

    /// List of issues found
    pub issues: Vec<IntegrityIssue>,

    /// Number of entries checked
    pub entries_checked: u64,

    /// Number of snapshots checked
    pub snapshots_checked: u64,

    /// Time taken for verification
    pub verification_duration: std::time::Duration,
}

/// Individual integrity issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityIssue {
    /// Type of issue
    pub issue_type: IntegrityIssueType,

    /// Description of the issue
    pub description: String,

    /// Affected log index (if applicable)
    pub log_index: Option<LogIndex>,

    /// Severity level
    pub severity: IssueSeverity,
}

/// Type of integrity issue
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntegrityIssueType {
    /// Checksum mismatch
    ChecksumMismatch,
    
    /// Missing log entry
    MissingEntry,
    
    /// Duplicate log entry
    DuplicateEntry,
    
    /// Invalid term sequence
    InvalidTermSequence,
    
    /// Corrupted snapshot
    CorruptedSnapshot,
    
    /// Invalid file format
    InvalidFormat,
    
    /// Other issue
    Other,
}

/// Severity level of an issue
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational only
    Info,
    
    /// Warning that may indicate a problem
    Warning,
    
    /// Error that affects functionality
    Error,
    
    /// Critical error that prevents operation
    Critical,
}

/// Storage maintenance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceReport {
    /// Operations performed
    pub operations_performed: Vec<MaintenanceOperation>,

    /// Storage space reclaimed in bytes
    pub space_reclaimed: u64,

    /// Time taken for maintenance
    pub maintenance_duration: std::time::Duration,

    /// Any issues encountered during maintenance
    pub issues: Vec<String>,
}

/// Type of maintenance operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceOperation {
    /// Compacted log entries
    LogCompaction { entries_removed: u64 },
    
    /// Cleaned up old snapshots
    SnapshotCleanup { snapshots_removed: u64 },
    
    /// Defragmented storage
    Defragmentation { space_reclaimed: u64 },
    
    /// Rebuilt indices
    IndexRebuild,
    
    /// Verified and repaired checksums
    ChecksumRepair { entries_repaired: u64 },
}

/// Helper trait for batch storage operations
#[async_trait]
pub trait BatchStorage: Storage {
    /// Begin a batch operation
    async fn begin_batch(&mut self) -> StorageResult<()>;

    /// Commit the current batch
    async fn commit_batch(&mut self) -> StorageResult<()>;

    /// Abort the current batch
    async fn abort_batch(&mut self) -> StorageResult<()>;
}

/// Helper trait for storage with transaction support
#[async_trait]
pub trait TransactionalStorage: Storage {
    /// Transaction handle
    type Transaction: Send + Sync;

    /// Begin a transaction
    async fn begin_transaction(&mut self) -> StorageResult<Self::Transaction>;

    /// Commit a transaction
    async fn commit_transaction(&mut self, transaction: Self::Transaction) -> StorageResult<()>;

    /// Rollback a transaction
    async fn rollback_transaction(&mut self, transaction: Self::Transaction) -> StorageResult<()>;
}

/// Configuration for storage implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of log entries to keep in memory
    pub max_memory_entries: usize,

    /// Whether to enable compression
    pub enable_compression: bool,

    /// Whether to enable checksums
    pub enable_checksums: bool,

    /// Sync mode for durability
    pub sync_mode: SyncMode,

    /// Buffer size for I/O operations
    pub io_buffer_size: usize,

    /// Whether to enable background maintenance
    pub enable_background_maintenance: bool,

    /// Interval between maintenance operations
    pub maintenance_interval: std::time::Duration,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_memory_entries: 1000,
            enable_compression: false,
            enable_checksums: true,
            sync_mode: SyncMode::Normal,
            io_buffer_size: 64 * 1024, // 64KB
            enable_background_maintenance: true,
            maintenance_interval: std::time::Duration::from_secs(300), // 5 minutes
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metrics_default() {
        let metrics = StorageMetrics::default();
        assert_eq!(metrics.total_log_entries, 0);
        assert_eq!(metrics.implementation, "unknown");
    }

    #[test]
    fn test_integrity_issue_severity_ordering() {
        assert!(IssueSeverity::Info < IssueSeverity::Warning);
        assert!(IssueSeverity::Warning < IssueSeverity::Error);
        assert!(IssueSeverity::Error < IssueSeverity::Critical);
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.max_memory_entries, 1000);
        assert!(config.enable_checksums);
        assert!(!config.enable_compression);
        assert!(matches!(config.sync_mode, SyncMode::Normal));
    }

    #[test]
    fn test_integrity_report_serialization() {
        let report = IntegrityReport {
            is_valid: true,
            issues: vec![],
            entries_checked: 100,
            snapshots_checked: 5,
            verification_duration: std::time::Duration::from_secs(10),
        };

        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: IntegrityReport = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(report.is_valid, deserialized.is_valid);
        assert_eq!(report.entries_checked, deserialized.entries_checked);
    }
}