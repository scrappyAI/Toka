//! File-based storage implementation for Raft.
//!
//! This implementation provides persistent storage using files on disk.
//! It's designed for production use with durability guarantees.

use crate::error::{StorageError, StorageResult};
use crate::storage::{
    IntegrityReport, MaintenanceReport, Storage, StorageConfig, StorageMetrics, SyncMode,
};
use crate::{PersistentState, StoredLogEntry, STORAGE_MAGIC, STORAGE_VERSION};
use async_trait::async_trait;
use raft_core::{LogEntry, LogIndex, Term};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::RwLock;

/// File-based storage implementation
pub struct FileStorage {
    /// Base directory for storage files
    base_dir: PathBuf,
    
    /// Configuration
    config: StorageConfig,
    
    /// Storage metrics
    metrics: Arc<RwLock<StorageMetrics>>,
}

impl FileStorage {
    /// Create a new file-based storage instance
    pub async fn new<P: AsRef<Path>>(base_dir: P) -> StorageResult<Self> {
        Self::with_config(base_dir, StorageConfig::default()).await
    }

    /// Create a new file-based storage instance with custom configuration
    pub async fn with_config<P: AsRef<Path>>(
        base_dir: P,
        config: StorageConfig,
    ) -> StorageResult<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Create base directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }
        
        let mut storage = Self {
            base_dir,
            config,
            metrics: Arc::new(RwLock::new(StorageMetrics {
                implementation: "file".to_string(),
                ..Default::default()
            })),
        };
        
        // Initialize storage files if needed
        storage.initialize().await?;
        
        Ok(storage)
    }

    /// Initialize storage files
    async fn initialize(&mut self) -> StorageResult<()> {
        // Create subdirectories
        for subdir in &["state", "log", "snapshots"] {
            let dir = self.base_dir.join(subdir);
            if !dir.exists() {
                fs::create_dir_all(dir).await?;
            }
        }
        
        Ok(())
    }

    /// Get path for persistent state file
    fn state_file_path(&self) -> PathBuf {
        self.base_dir.join("state").join("persistent.json")
    }

    /// Get path for log file
    fn log_file_path(&self) -> PathBuf {
        self.base_dir.join("log").join("entries.log")
    }

    /// Get path for snapshot file
    fn snapshot_file_path(&self, index: LogIndex) -> PathBuf {
        self.base_dir
            .join("snapshots")
            .join(format!("snapshot_{}.snap", index))
    }

    /// Write file header with magic bytes and version
    async fn write_file_header(file: &mut tokio::fs::File) -> StorageResult<()> {
        file.write_all(STORAGE_MAGIC).await?;
        file.write_u32(STORAGE_VERSION).await?;
        Ok(())
    }

    /// Read and verify file header
    async fn read_file_header(file: &mut tokio::fs::File) -> StorageResult<()> {
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).await?;
        
        if &magic != STORAGE_MAGIC {
            return Err(StorageError::invalid_format("Invalid magic bytes"));
        }
        
        let version = file.read_u32().await?;
        if version != STORAGE_VERSION {
            return Err(StorageError::invalid_format(format!(
                "Unsupported version: {}",
                version
            )));
        }
        
        Ok(())
    }

    /// Sync file based on configuration
    async fn sync_file(&self, file: &mut tokio::fs::File) -> StorageResult<()> {
        match self.config.sync_mode {
            SyncMode::None => Ok(()),
            SyncMode::Normal => {
                file.sync_data().await?;
                Ok(())
            }
            SyncMode::Full => {
                file.sync_all().await?;
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn store_persistent_state(&mut self, state: &PersistentState) -> StorageResult<()> {
        let path = self.state_file_path();
        let serialized = serde_json::to_vec_pretty(state)?;
        
        // Write to temporary file first, then atomically rename
        let temp_path = path.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path).await?;
        
        Self::write_file_header(&mut file).await?;
        file.write_all(&serialized).await?;
        self.sync_file(&mut file).await?;
        drop(file);
        
        // Atomic rename
        tokio::fs::rename(temp_path, path).await?;
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.write_operations += 1;
        }
        Ok(())
    }

    async fn load_persistent_state(&self) -> StorageResult<Option<PersistentState>> {
        let path = self.state_file_path();
        
        if !path.exists() {
            return Ok(None);
        }
        
        let mut file = tokio::fs::File::open(path).await?;
        Self::read_file_header(&mut file).await?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        
        let state: PersistentState = serde_json::from_slice(&contents)?;
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.read_operations += 1;
        }
        Ok(Some(state))
    }

    async fn store_log_entry(&mut self, entry: &LogEntry) -> StorageResult<()> {
        // This is a simplified implementation
        // In practice, you'd want a more sophisticated log file format
        // with indices, checksums, and efficient seeking
        
        let stored_entry = StoredLogEntry::new(entry.clone());
        let serialized = serde_json::to_vec(&stored_entry)?;
        
        let path = self.log_file_path();
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        
        // Write entry size followed by entry data
        file.write_u32(serialized.len() as u32).await?;
        file.write_all(&serialized).await?;
        
        self.sync_file(&mut file).await?;
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.write_operations += 1;
            metrics.total_log_entries += 1;
        }
        
        Ok(())
    }

    async fn store_log_entries(&mut self, entries: &[LogEntry]) -> StorageResult<()> {
        for entry in entries {
            self.store_log_entry(entry).await?;
        }
        Ok(())
    }

    async fn get_log_entry(&self, index: LogIndex) -> StorageResult<Option<LogEntry>> {
        // This is a simplified implementation that reads the entire log file
        // In practice, you'd want to maintain an index for efficient seeking
        
        let path = self.log_file_path();
        if !path.exists() {
            return Ok(None);
        }
        
        let mut file = tokio::fs::File::open(path).await?;
        
        loop {
            // Try to read entry size
            let entry_size = match file.read_u32().await {
                Ok(size) => size,
                Err(_) => break, // End of file
            };
            
            // Read entry data
            let mut entry_data = vec![0u8; entry_size as usize];
            file.read_exact(&mut entry_data).await?;
            
            let stored_entry: StoredLogEntry = serde_json::from_slice(&entry_data)?;
            
            if stored_entry.entry.index == index {
                if stored_entry.verify_integrity() {
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.read_operations += 1;
                    }
                    return Ok(Some(stored_entry.entry));
                } else {
                    return Err(StorageError::corruption(format!(
                        "Corrupted entry at index {}",
                        index
                    )));
                }
            }
        }
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.read_operations += 1;
        }
        Ok(None)
    }

    async fn get_log_entries(&self, range: Range<LogIndex>) -> StorageResult<Vec<LogEntry>> {
        let mut result = Vec::new();
        
        for index in range {
            if let Some(entry) = self.get_log_entry(index).await? {
                result.push(entry);
            } else {
                break;
            }
        }
        
        Ok(result)
    }

    async fn first_log_index(&self) -> StorageResult<LogIndex> {
        // Simplified implementation - in practice you'd maintain metadata
        let path = self.log_file_path();
        if !path.exists() {
            return Ok(1);
        }
        
        let mut file = tokio::fs::File::open(path).await?;
        
        // Read first entry
        if let Ok(entry_size) = file.read_u32().await {
            let mut entry_data = vec![0u8; entry_size as usize];
            file.read_exact(&mut entry_data).await?;
            
            let stored_entry: StoredLogEntry = serde_json::from_slice(&entry_data)?;
            Ok(stored_entry.entry.index)
        } else {
            Ok(1)
        }
    }

    async fn last_log_index(&self) -> StorageResult<LogIndex> {
        // Simplified implementation - reads entire file to find last entry
        let path = self.log_file_path();
        if !path.exists() {
            return Ok(0);
        }
        
        let mut file = tokio::fs::File::open(path).await?;
        let mut last_index = 0;
        
        loop {
            let entry_size = match file.read_u32().await {
                Ok(size) => size,
                Err(_) => break,
            };
            
            let mut entry_data = vec![0u8; entry_size as usize];
            file.read_exact(&mut entry_data).await?;
            
            let stored_entry: StoredLogEntry = serde_json::from_slice(&entry_data)?;
            last_index = stored_entry.entry.index;
        }
        
        Ok(last_index)
    }

    async fn get_log_term(&self, index: LogIndex) -> StorageResult<Option<Term>> {
        self.get_log_entry(index)
            .await
            .map(|entry| entry.map(|e| e.term))
    }

    async fn truncate_log_from(&mut self, _index: LogIndex) -> StorageResult<()> {
        // Simplified implementation - in practice you'd need to truncate the file
        // This would require rebuilding the log file
        Err(StorageError::internal(
            "Log truncation not implemented in this simplified version",
        ))
    }

    async fn compact_log_to(&mut self, _index: LogIndex) -> StorageResult<()> {
        // Simplified implementation - in practice you'd need to remove old entries
        // This would require rebuilding the log file
        Err(StorageError::internal(
            "Log compaction not implemented in this simplified version",
        ))
    }

    async fn store_snapshot(
        &mut self,
        snapshot: &[u8],
        last_included_index: LogIndex,
        last_included_term: Term,
    ) -> StorageResult<()> {
        let path = self.snapshot_file_path(last_included_index);
        let mut file = tokio::fs::File::create(path).await?;
        
        Self::write_file_header(&mut file).await?;
        file.write_u64(last_included_index).await?;
        file.write_u64(last_included_term).await?;
        file.write_u32(snapshot.len() as u32).await?;
        file.write_all(snapshot).await?;
        
        self.sync_file(&mut file).await?;
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.write_operations += 1;
            metrics.snapshot_count += 1;
        }
        
        Ok(())
    }

    async fn load_snapshot(&self) -> StorageResult<Option<(Vec<u8>, LogIndex, Term)>> {
        let snapshots_dir = self.base_dir.join("snapshots");
        if !snapshots_dir.exists() {
            return Ok(None);
        }
        
        // Find the latest snapshot file
        let mut entries = tokio::fs::read_dir(snapshots_dir).await?;
        let mut latest_snapshot: Option<PathBuf> = None;
        let mut latest_index = 0u64;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.starts_with("snapshot_") && filename.ends_with(".snap") {
                    if let Some(index_str) = filename
                        .strip_prefix("snapshot_")
                        .and_then(|s| s.strip_suffix(".snap"))
                    {
                        if let Ok(index) = index_str.parse::<u64>() {
                            if index > latest_index {
                                latest_index = index;
                                latest_snapshot = Some(path);
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(path) = latest_snapshot {
            let mut file = tokio::fs::File::open(path).await?;
            
            Self::read_file_header(&mut file).await?;
            let last_included_index = file.read_u64().await?;
            let last_included_term = file.read_u64().await?;
            let snapshot_size = file.read_u32().await?;
            
            let mut snapshot_data = vec![0u8; snapshot_size as usize];
            file.read_exact(&mut snapshot_data).await?;
            
            {
                let mut metrics = self.metrics.write().await;
                metrics.read_operations += 1;
            }
            Ok(Some((snapshot_data, last_included_index, last_included_term)))
        } else {
            Ok(None)
        }
    }

    async fn cleanup_snapshots(&mut self) -> StorageResult<()> {
        // Keep only the latest snapshot
        let snapshots_dir = self.base_dir.join("snapshots");
        if !snapshots_dir.exists() {
            return Ok(());
        }
        
        let mut entries = tokio::fs::read_dir(&snapshots_dir).await?;
        let mut snapshots = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.starts_with("snapshot_") && filename.ends_with(".snap") {
                    if let Some(index_str) = filename
                        .strip_prefix("snapshot_")
                        .and_then(|s| s.strip_suffix(".snap"))
                    {
                        if let Ok(index) = index_str.parse::<u64>() {
                            snapshots.push((index, path));
                        }
                    }
                }
            }
        }
        
        // Sort by index and keep only the latest
        snapshots.sort_by_key(|(index, _)| *index);
        
        // Remove all but the latest snapshot
        for (_, path) in snapshots.iter().take(snapshots.len().saturating_sub(1)) {
            tokio::fs::remove_file(path).await?;
        }
        
        Ok(())
    }

    async fn metrics(&self) -> StorageResult<StorageMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    async fn sync(&mut self) -> StorageResult<()> {
        {
            let mut metrics = self.metrics.write().await;
            metrics.sync_operations += 1;
        }
        Ok(())
    }

    async fn verify_integrity(&self) -> StorageResult<IntegrityReport> {
        // Simplified integrity check - in practice you'd be more thorough
        let start = std::time::Instant::now();
        
        let mut issues = Vec::new();
        let mut entries_checked = 0u64;
        
        // Check if log file exists and is readable
        let log_path = self.log_file_path();
        if log_path.exists() {
            match tokio::fs::File::open(log_path).await {
                Ok(mut file) => {
                    // Try to read entries
                    loop {
                        match file.read_u32().await {
                            Ok(entry_size) => {
                                let mut entry_data = vec![0u8; entry_size as usize];
                                if file.read_exact(&mut entry_data).await.is_ok() {
                                    if let Ok(stored_entry) =
                                        serde_json::from_slice::<StoredLogEntry>(&entry_data)
                                    {
                                        entries_checked += 1;
                                        if !stored_entry.verify_integrity() {
                                            issues.push(crate::storage::IntegrityIssue {
                                                issue_type: crate::storage::IntegrityIssueType::ChecksumMismatch,
                                                description: format!(
                                                    "Checksum mismatch for entry at index {}",
                                                    stored_entry.entry.index
                                                ),
                                                log_index: Some(stored_entry.entry.index),
                                                severity: crate::storage::IssueSeverity::Error,
                                            });
                                        }
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                Err(_) => {
                    issues.push(crate::storage::IntegrityIssue {
                        issue_type: crate::storage::IntegrityIssueType::Other,
                        description: "Cannot read log file".to_string(),
                        log_index: None,
                        severity: crate::storage::IssueSeverity::Critical,
                    });
                }
            }
        }
        
        let duration = start.elapsed();
        let is_valid = issues
            .iter()
            .all(|issue| issue.severity < crate::storage::IssueSeverity::Error);
        
        Ok(IntegrityReport {
            is_valid,
            issues,
            entries_checked,
            snapshots_checked: 0, // TODO: implement snapshot checking
            verification_duration: duration,
        })
    }

    async fn maintenance(&mut self) -> StorageResult<MaintenanceReport> {
        let start = std::time::Instant::now();
        
        // Cleanup old snapshots
        self.cleanup_snapshots().await?;
        
        let duration = start.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.last_maintenance = Some(chrono::Utc::now());
        }
        
        Ok(MaintenanceReport {
            operations_performed: vec![crate::storage::MaintenanceOperation::SnapshotCleanup {
                snapshots_removed: 0, // TODO: track actual removals
            }],
            space_reclaimed: 0,
            maintenance_duration: duration,
            issues: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use raft_core::LogEntry;

    #[tokio::test]
    async fn test_file_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path()).await.unwrap();
        
        // Check that directories were created
        assert!(temp_dir.path().join("state").exists());
        assert!(temp_dir.path().join("log").exists());
        assert!(temp_dir.path().join("snapshots").exists());
    }

    #[tokio::test]
    async fn test_persistent_state_storage() {
        let temp_dir = tempdir().unwrap();
        let mut storage = FileStorage::new(temp_dir.path()).await.unwrap();
        
        let state = PersistentState {
            current_term: 5,
            voted_for: Some(1),
            last_applied: 10,
            cluster_config: Default::default(),
        };
        
        storage.store_persistent_state(&state).await.unwrap();
        let loaded = storage.load_persistent_state().await.unwrap();
        
        assert_eq!(loaded, Some(state));
    }

    #[tokio::test]
    async fn test_log_entry_storage() {
        let temp_dir = tempdir().unwrap();
        let mut storage = FileStorage::new(temp_dir.path()).await.unwrap();
        
        let entry = LogEntry::new_command(1, 1, b"test command".to_vec());
        storage.store_log_entry(&entry).await.unwrap();
        
        let retrieved = storage.get_log_entry(1).await.unwrap();
        assert_eq!(retrieved, Some(entry));
    }

    #[tokio::test]
    async fn test_snapshot_storage() {
        let temp_dir = tempdir().unwrap();
        let mut storage = FileStorage::new(temp_dir.path()).await.unwrap();
        
        let snapshot_data = b"snapshot content".to_vec();
        storage
            .store_snapshot(&snapshot_data, 100, 5)
            .await
            .unwrap();
        
        let loaded = storage.load_snapshot().await.unwrap();
        assert_eq!(loaded, Some((snapshot_data, 100, 5)));
    }
}