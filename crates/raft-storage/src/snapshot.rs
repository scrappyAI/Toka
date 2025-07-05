//! Snapshot management utilities for Raft storage.
//!
//! This module provides utilities for creating, managing, and verifying snapshots
//! in the Raft consensus algorithm. Snapshots are used for log compaction and
//! bringing new nodes up to date quickly.

use crate::error::{StorageError, StorageResult};
use raft_core::{LogIndex, Term};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Metadata about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotMetadata {
    /// Index of the last log entry included in the snapshot
    pub last_included_index: LogIndex,
    
    /// Term of the last log entry included in the snapshot
    pub last_included_term: Term,
    
    /// Size of the snapshot data in bytes
    pub size: u64,
    
    /// Checksum of the snapshot data
    pub checksum: u32,
    
    /// When the snapshot was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Configuration index at the time of snapshot
    pub config_index: LogIndex,
    
    /// Cluster configuration at the time of snapshot
    pub cluster_config: Vec<u64>,
}

impl SnapshotMetadata {
    /// Create new snapshot metadata
    pub fn new(
        last_included_index: LogIndex,
        last_included_term: Term,
        size: u64,
        checksum: u32,
        config_index: LogIndex,
        cluster_config: Vec<u64>,
    ) -> Self {
        Self {
            last_included_index,
            last_included_term,
            size,
            checksum,
            created_at: chrono::Utc::now(),
            config_index,
            cluster_config,
        }
    }
}

/// Snapshot manager for handling snapshot operations
pub struct SnapshotManager {
    /// Base directory for snapshots
    snapshots_dir: PathBuf,
    
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
    
    /// Whether to compress snapshots
    compress: bool,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub async fn new<P: AsRef<Path>>(snapshots_dir: P) -> StorageResult<Self> {
        Self::with_config(snapshots_dir, 5, false).await
    }

    /// Create a new snapshot manager with configuration
    pub async fn with_config<P: AsRef<Path>>(
        snapshots_dir: P,
        max_snapshots: usize,
        compress: bool,
    ) -> StorageResult<Self> {
        let snapshots_dir = snapshots_dir.as_ref().to_path_buf();
        
        // Create snapshots directory if it doesn't exist
        if !snapshots_dir.exists() {
            fs::create_dir_all(&snapshots_dir).await?;
        }
        
        Ok(Self {
            snapshots_dir,
            max_snapshots,
            compress,
        })
    }

    /// Store a snapshot with metadata
    pub async fn store_snapshot(
        &self,
        snapshot_data: &[u8],
        metadata: &SnapshotMetadata,
    ) -> StorageResult<()> {
        let snapshot_path = self.snapshot_path(metadata.last_included_index);
        let metadata_path = self.metadata_path(metadata.last_included_index);
        
        // Write snapshot data
        let mut snapshot_file = fs::File::create(&snapshot_path).await?;
        if self.compress {
            // TODO: Implement compression
            snapshot_file.write_all(snapshot_data).await?;
        } else {
            snapshot_file.write_all(snapshot_data).await?;
        }
        snapshot_file.sync_all().await?;
        
        // Write metadata
        let metadata_json = serde_json::to_vec_pretty(metadata)?;
        let mut metadata_file = fs::File::create(&metadata_path).await?;
        metadata_file.write_all(&metadata_json).await?;
        metadata_file.sync_all().await?;
        
        // Cleanup old snapshots
        self.cleanup_old_snapshots().await?;
        
        Ok(())
    }

    /// Load the latest snapshot
    pub async fn load_latest_snapshot(&self) -> StorageResult<Option<(Vec<u8>, SnapshotMetadata)>> {
        let snapshots = self.list_snapshots().await?;
        if snapshots.is_empty() {
            return Ok(None);
        }
        
        // Get the latest snapshot
        let latest_index = snapshots.iter().map(|(index, _)| *index).max().unwrap();
        self.load_snapshot(latest_index).await.map(Some)
    }

    /// Load a specific snapshot by index
    pub async fn load_snapshot(&self, index: LogIndex) -> StorageResult<(Vec<u8>, SnapshotMetadata)> {
        let snapshot_path = self.snapshot_path(index);
        let metadata_path = self.metadata_path(index);
        
        // Load metadata
        let metadata_data = fs::read(&metadata_path).await?;
        let metadata: SnapshotMetadata = serde_json::from_slice(&metadata_data)?;
        
        // Load snapshot data
        let mut snapshot_data = fs::read(&snapshot_path).await?;
        
        if self.compress {
            // TODO: Implement decompression
        }
        
        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&snapshot_data);
        if calculated_checksum != metadata.checksum {
            return Err(StorageError::corruption(format!(
                "Snapshot checksum mismatch: expected {}, got {}",
                metadata.checksum, calculated_checksum
            )));
        }
        
        Ok((snapshot_data, metadata))
    }

    /// List all available snapshots
    pub async fn list_snapshots(&self) -> StorageResult<Vec<(LogIndex, SnapshotMetadata)>> {
        let mut entries = fs::read_dir(&self.snapshots_dir).await?;
        let mut snapshots = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.starts_with("snapshot_") && filename.ends_with(".snap") {
                    if let Some(index_str) = filename
                        .strip_prefix("snapshot_")
                        .and_then(|s| s.strip_suffix(".snap"))
                    {
                        if let Ok(index) = index_str.parse::<LogIndex>() {
                            // Load metadata
                            let metadata_path = self.metadata_path(index);
                            if metadata_path.exists() {
                                let metadata_data = fs::read(&metadata_path).await?;
                                let metadata: SnapshotMetadata = serde_json::from_slice(&metadata_data)?;
                                snapshots.push((index, metadata));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by index
        snapshots.sort_by_key(|(index, _)| *index);
        
        Ok(snapshots)
    }

    /// Delete a specific snapshot
    pub async fn delete_snapshot(&self, index: LogIndex) -> StorageResult<()> {
        let snapshot_path = self.snapshot_path(index);
        let metadata_path = self.metadata_path(index);
        
        if snapshot_path.exists() {
            fs::remove_file(snapshot_path).await?;
        }
        
        if metadata_path.exists() {
            fs::remove_file(metadata_path).await?;
        }
        
        Ok(())
    }

    /// Cleanup old snapshots, keeping only the most recent ones
    async fn cleanup_old_snapshots(&self) -> StorageResult<()> {
        let snapshots = self.list_snapshots().await?;
        
        if snapshots.len() > self.max_snapshots {
            // Sort by index and remove older snapshots
            let mut sorted_snapshots = snapshots;
            sorted_snapshots.sort_by_key(|(index, _)| *index);
            
            let to_remove = sorted_snapshots.len() - self.max_snapshots;
            for (index, _) in sorted_snapshots.iter().take(to_remove) {
                self.delete_snapshot(*index).await?;
            }
        }
        
        Ok(())
    }

    /// Verify the integrity of a snapshot
    pub async fn verify_snapshot(&self, index: LogIndex) -> StorageResult<bool> {
        match self.load_snapshot(index).await {
            Ok(_) => Ok(true),
            Err(e) if e.is_corruption() => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get snapshot file path
    fn snapshot_path(&self, index: LogIndex) -> PathBuf {
        self.snapshots_dir.join(format!("snapshot_{}.snap", index))
    }

    /// Get metadata file path
    fn metadata_path(&self, index: LogIndex) -> PathBuf {
        self.snapshots_dir.join(format!("snapshot_{}.meta", index))
    }

    /// Calculate checksum for data
    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Get storage statistics
    pub async fn storage_stats(&self) -> StorageResult<SnapshotStats> {
        let snapshots = self.list_snapshots().await?;
        let mut total_size = 0;
        let mut oldest_snapshot = None;
        let mut newest_snapshot = None;
        
        for (index, metadata) in &snapshots {
            total_size += metadata.size;
            
            if oldest_snapshot.is_none() || Some(*index) < oldest_snapshot {
                oldest_snapshot = Some(*index);
            }
            
            if newest_snapshot.is_none() || Some(*index) > newest_snapshot {
                newest_snapshot = Some(*index);
            }
        }
        
        Ok(SnapshotStats {
            count: snapshots.len(),
            total_size,
            oldest_snapshot,
            newest_snapshot,
        })
    }

    /// Create a snapshot from state machine data
    pub async fn create_snapshot(
        &self,
        state_machine_data: &[u8],
        last_included_index: LogIndex,
        last_included_term: Term,
        config_index: LogIndex,
        cluster_config: Vec<u64>,
    ) -> StorageResult<SnapshotMetadata> {
        let checksum = self.calculate_checksum(state_machine_data);
        let size = state_machine_data.len() as u64;
        
        let metadata = SnapshotMetadata::new(
            last_included_index,
            last_included_term,
            size,
            checksum,
            config_index,
            cluster_config,
        );
        
        self.store_snapshot(state_machine_data, &metadata).await?;
        
        Ok(metadata)
    }
}

/// Statistics about stored snapshots
#[derive(Debug, Clone)]
pub struct SnapshotStats {
    /// Number of snapshots stored
    pub count: usize,
    
    /// Total size of all snapshots in bytes
    pub total_size: u64,
    
    /// Index of the oldest snapshot
    pub oldest_snapshot: Option<LogIndex>,
    
    /// Index of the newest snapshot
    pub newest_snapshot: Option<LogIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_snapshot_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().join("snapshots"))
            .await
            .unwrap();
        
        assert!(temp_dir.path().join("snapshots").exists());
    }

    #[tokio::test]
    async fn test_snapshot_storage_and_retrieval() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().join("snapshots"))
            .await
            .unwrap();
        
        let snapshot_data = b"test snapshot data".to_vec();
        let checksum = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            snapshot_data.hash(&mut hasher);
            hasher.finish() as u32
        };
        
        let metadata = SnapshotMetadata::new(
            100,
            5,
            snapshot_data.len() as u64,
            checksum,
            50,
            vec![1, 2, 3],
        );
        
        // Store snapshot
        manager.store_snapshot(&snapshot_data, &metadata).await.unwrap();
        
        // Load snapshot
        let (loaded_data, loaded_metadata) = manager.load_snapshot(100).await.unwrap();
        
        assert_eq!(loaded_data, snapshot_data);
        assert_eq!(loaded_metadata.last_included_index, 100);
        assert_eq!(loaded_metadata.last_included_term, 5);
    }

    #[tokio::test]
    async fn test_snapshot_listing() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().join("snapshots"))
            .await
            .unwrap();
        
        // Create multiple snapshots
        for i in 1..=3 {
            let snapshot_data = format!("snapshot data {}", i).into_bytes();
            let checksum = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                snapshot_data.hash(&mut hasher);
                hasher.finish() as u32
            };
            
            let metadata = SnapshotMetadata::new(
                (i * 100) as u64,
                i as u64,
                snapshot_data.len() as u64,
                checksum,
                (i * 50) as u64,
                vec![1, 2, 3],
            );
            
            manager.store_snapshot(&snapshot_data, &metadata).await.unwrap();
        }
        
        let snapshots = manager.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 3);
        
        // Should be sorted by index
        assert_eq!(snapshots[0].0, 100);
        assert_eq!(snapshots[1].0, 200);
        assert_eq!(snapshots[2].0, 300);
    }

    #[tokio::test]
    async fn test_snapshot_cleanup() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::with_config(
            temp_dir.path().join("snapshots"),
            2, // Keep only 2 snapshots
            false,
        )
        .await
        .unwrap();
        
        // Create 4 snapshots
        for i in 1..=4 {
            let snapshot_data = format!("snapshot data {}", i).into_bytes();
            let checksum = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                snapshot_data.hash(&mut hasher);
                hasher.finish() as u32
            };
            
            let metadata = SnapshotMetadata::new(
                (i * 100) as u64,
                i as u64,
                snapshot_data.len() as u64,
                checksum,
                (i * 50) as u64,
                vec![1, 2, 3],
            );
            
            manager.store_snapshot(&snapshot_data, &metadata).await.unwrap();
        }
        
        // Should only have 2 snapshots (the latest ones)
        let snapshots = manager.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].0, 300);
        assert_eq!(snapshots[1].0, 400);
    }

    #[tokio::test]
    async fn test_latest_snapshot() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().join("snapshots"))
            .await
            .unwrap();
        
        // No snapshots initially
        let latest = manager.load_latest_snapshot().await.unwrap();
        assert!(latest.is_none());
        
        // Create a snapshot
        let snapshot_data = b"latest snapshot".to_vec();
        let checksum = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            snapshot_data.hash(&mut hasher);
            hasher.finish() as u32
        };
        
        let metadata = SnapshotMetadata::new(
            500,
            10,
            snapshot_data.len() as u64,
            checksum,
            250,
            vec![1, 2, 3],
        );
        
        manager.store_snapshot(&snapshot_data, &metadata).await.unwrap();
        
        // Load latest snapshot
        let (loaded_data, loaded_metadata) = manager.load_latest_snapshot().await.unwrap().unwrap();
        assert_eq!(loaded_data, snapshot_data);
        assert_eq!(loaded_metadata.last_included_index, 500);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().join("snapshots"))
            .await
            .unwrap();
        
        // Create snapshots
        for i in 1..=3 {
            let snapshot_data = vec![0u8; i * 100]; // Different sizes
            let checksum = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                snapshot_data.hash(&mut hasher);
                hasher.finish() as u32
            };
            
            let metadata = SnapshotMetadata::new(
                (i * 100) as u64,
                i as u64,
                snapshot_data.len() as u64,
                checksum,
                (i * 50) as u64,
                vec![1, 2, 3],
            );
            
            manager.store_snapshot(&snapshot_data, &metadata).await.unwrap();
        }
        
        let stats = manager.storage_stats().await.unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_size, 100 + 200 + 300); // Sum of sizes
        assert_eq!(stats.oldest_snapshot, Some(100));
        assert_eq!(stats.newest_snapshot, Some(300));
    }
}