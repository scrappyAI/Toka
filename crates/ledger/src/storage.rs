//! Storage abstractions for the ledger.
//!
//! This module provides pluggable storage backends for ledger persistence
//! with Write-Ahead Logging (WAL) and atomic commit support.

use std::fs;
use std::io;
use std::path::Path;
use crate::ledger::Ledger;
use crate::event::{LedgerEvent, WALEntry};

/// Error type for storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Storage not available")]
    NotAvailable,
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("WAL corruption detected")]
    WALCorruption,
    #[error("Atomic commit failed: {0}")]
    CommitFailed(String),
}

/// Trait for ledger storage backends with WAL and atomic commit support.
/// 
/// Implementations should handle serialization, deserialization, persistence,
/// and atomic operations for ledger state including accounts and events.
pub trait Storage {
    /// Saves the entire ledger state.
    fn save_ledger(&mut self, ledger: &Ledger) -> Result<(), StorageError>;
    
    /// Loads the entire ledger state.
    fn load_ledger(&self) -> Result<Ledger, StorageError>;
    
    /// Checks if the storage is available and writable.
    fn is_available(&self) -> bool;
    
    /// Clears all stored data (useful for testing).
    fn clear(&mut self) -> Result<(), StorageError>;
}

/// Extended trait for storage backends that support WAL and atomic operations.
pub trait WALStorage: Storage {
    /// Appends an event to the Write-Ahead Log.
    fn append_to_wal(&mut self, entry: WALEntry) -> Result<(), StorageError>;
    
    /// Commits all staged events in the WAL atomically.
    fn commit_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError>;
    
    /// Rolls back all staged events in the WAL.
    fn rollback_wal(&mut self) -> Result<(), StorageError>;
    
    /// Recovers from WAL on startup.
    fn recover_from_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError>;
    
    /// Gets the current WAL sequence number.
    fn get_wal_sequence(&self) -> u64;
    
    /// Truncates the WAL (removes committed entries).
    fn truncate_wal(&mut self) -> Result<(), StorageError>;
}

/// In-memory storage backend for testing and development with WAL support.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    data: Option<String>,
    wal: Vec<WALEntry>,
    wal_sequence: u64,
}

impl MemoryStorage {
    /// Creates a new in-memory storage backend.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn save_ledger(&mut self, ledger: &Ledger) -> Result<(), StorageError> {
        let serialized = serde_json::to_string(ledger)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.data = Some(serialized);
        Ok(())
    }
    
    fn load_ledger(&self) -> Result<Ledger, StorageError> {
        match &self.data {
            Some(data) => {
                serde_json::from_str(data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))
            },
            None => Ok(Ledger::default()),
        }
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn clear(&mut self) -> Result<(), StorageError> {
        self.data = None;
        self.wal.clear();
        self.wal_sequence = 0;
        Ok(())
    }
}

impl WALStorage for MemoryStorage {
    fn append_to_wal(&mut self, entry: WALEntry) -> Result<(), StorageError> {
        self.wal.push(entry);
        self.wal_sequence += 1;
        Ok(())
    }
    
    fn commit_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        let committed_events: Vec<LedgerEvent> = self.wal
            .iter()
            .filter(|entry| entry.event.is_staged())
            .map(|entry| entry.event.clone().commit())
            .collect();
        
        // Clear staged entries from WAL
        self.wal.retain(|entry| !entry.event.is_staged());
        
        Ok(committed_events)
    }
    
    fn rollback_wal(&mut self) -> Result<(), StorageError> {
        // Remove all staged entries
        self.wal.retain(|entry| !entry.event.is_staged());
        Ok(())
    }
    
    fn recover_from_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        let recovery_events: Vec<LedgerEvent> = self.wal
            .iter()
            .filter(|entry| entry.event.is_committed())
            .map(|entry| entry.event.clone())
            .collect();
        
        Ok(recovery_events)
    }
    
    fn get_wal_sequence(&self) -> u64 {
        self.wal_sequence
    }
    
    fn truncate_wal(&mut self) -> Result<(), StorageError> {
        self.wal.retain(|entry| entry.event.is_staged());
        Ok(())
    }
}

/// File-based storage backend with WAL support for persistent storage.
#[derive(Debug)]
pub struct FileStorage {
    file_path: String,
    wal_path: String,
    wal_sequence: u64,
}

impl FileStorage {
    /// Creates a new file storage backend with the specified path.
    pub fn new(file_path: impl Into<String>) -> Self {
        let file_path = file_path.into();
        let wal_path = format!("{}.wal", file_path);
        
        Self {
            file_path,
            wal_path,
            wal_sequence: 0,
        }
    }
    
    /// Loads WAL sequence from file system.
    fn load_wal_sequence(&mut self) -> Result<(), StorageError> {
        if Path::new(&self.wal_path).exists() {
            let wal_data = fs::read_to_string(&self.wal_path)?;
            if !wal_data.trim().is_empty() {
                let entries: Vec<WALEntry> = serde_json::from_str(&wal_data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                self.wal_sequence = entries.iter().map(|e| e.sequence).max().unwrap_or(0);
            }
        }
        Ok(())
    }
}

impl Storage for FileStorage {
    fn save_ledger(&mut self, ledger: &Ledger) -> Result<(), StorageError> {
        let serialized = serde_json::to_string_pretty(ledger)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        // Atomic write: write to temp file, then rename
        let temp_path = format!("{}.tmp", self.file_path);
        fs::write(&temp_path, serialized)?;
        fs::rename(&temp_path, &self.file_path)?;
        
        Ok(())
    }
    
    fn load_ledger(&self) -> Result<Ledger, StorageError> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Ledger::default());
        }
        
        let data = fs::read_to_string(&self.file_path)?;
        serde_json::from_str(&data)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
    
    fn is_available(&self) -> bool {
        if let Some(parent) = Path::new(&self.file_path).parent() {
            parent.exists() && fs::metadata(parent).map(|m| !m.permissions().readonly()).unwrap_or(false)
        } else {
            false
        }
    }
    
    fn clear(&mut self) -> Result<(), StorageError> {
        if Path::new(&self.file_path).exists() {
            fs::remove_file(&self.file_path)?;
        }
        if Path::new(&self.wal_path).exists() {
            fs::remove_file(&self.wal_path)?;
        }
        self.wal_sequence = 0;
        Ok(())
    }
}

impl WALStorage for FileStorage {
    fn append_to_wal(&mut self, entry: WALEntry) -> Result<(), StorageError> {
        self.load_wal_sequence()?;
        
        let mut entries = if Path::new(&self.wal_path).exists() {
            let wal_data = fs::read_to_string(&self.wal_path)?;
            if wal_data.trim().is_empty() {
                Vec::new()
            } else {
                serde_json::from_str::<Vec<WALEntry>>(&wal_data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?
            }
        } else {
            Vec::new()
        };
        
        entries.push(entry);
        self.wal_sequence += 1;
        
        let serialized = serde_json::to_string_pretty(&entries)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        // Atomic write to WAL
        let temp_wal_path = format!("{}.tmp", self.wal_path);
        fs::write(&temp_wal_path, serialized)?;
        fs::rename(&temp_wal_path, &self.wal_path)?;
        
        Ok(())
    }
    
    fn commit_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        if !Path::new(&self.wal_path).exists() {
            return Ok(Vec::new());
        }
        
        let wal_data = fs::read_to_string(&self.wal_path)?;
        if wal_data.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut entries: Vec<WALEntry> = serde_json::from_str(&wal_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let committed_events: Vec<LedgerEvent> = entries
            .iter_mut()
            .filter(|entry| entry.event.is_staged())
            .map(|entry| {
                let committed_event = entry.event.clone().commit();
                entry.event = committed_event.clone();
                committed_event
            })
            .collect();
        
        // Write back updated WAL
        let serialized = serde_json::to_string_pretty(&entries)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&self.wal_path, serialized)?;
        
        Ok(committed_events)
    }
    
    fn rollback_wal(&mut self) -> Result<(), StorageError> {
        if !Path::new(&self.wal_path).exists() {
            return Ok(());
        }
        
        let wal_data = fs::read_to_string(&self.wal_path)?;
        if wal_data.trim().is_empty() {
            return Ok(());
        }
        
        let entries: Vec<WALEntry> = serde_json::from_str(&wal_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        // Keep only committed entries, remove staged ones
        let remaining_entries: Vec<WALEntry> = entries
            .into_iter()
            .filter(|entry| !entry.event.is_staged())
            .collect();
        
        let serialized = serde_json::to_string_pretty(&remaining_entries)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&self.wal_path, serialized)?;
        
        Ok(())
    }
    
    fn recover_from_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        if !Path::new(&self.wal_path).exists() {
            return Ok(Vec::new());
        }
        
        let wal_data = fs::read_to_string(&self.wal_path)?;
        if wal_data.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let entries: Vec<WALEntry> = serde_json::from_str(&wal_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let recovery_events: Vec<LedgerEvent> = entries
            .into_iter()
            .filter(|entry| entry.event.is_committed())
            .map(|entry| entry.event)
            .collect();
        
        Ok(recovery_events)
    }
    
    fn get_wal_sequence(&self) -> u64 {
        self.wal_sequence
    }
    
    fn truncate_wal(&mut self) -> Result<(), StorageError> {
        if !Path::new(&self.wal_path).exists() {
            return Ok(());
        }
        
        let wal_data = fs::read_to_string(&self.wal_path)?;
        if wal_data.trim().is_empty() {
            return Ok(());
        }
        
        let entries: Vec<WALEntry> = serde_json::from_str(&wal_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        // Keep only staged entries
        let staged_entries: Vec<WALEntry> = entries
            .into_iter()
            .filter(|entry| entry.event.is_staged())
            .collect();
        
        let serialized = serde_json::to_string_pretty(&staged_entries)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&self.wal_path, serialized)?;
        
        Ok(())
    }
}

/// No-op storage backend that doesn't persist anything.
/// Useful for testing or when persistence is not needed.
#[derive(Debug, Default)]
pub struct NoOpStorage {
    wal_sequence: u64,
}

impl NoOpStorage {
    /// Creates a new no-op storage backend.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for NoOpStorage {
    fn save_ledger(&mut self, _ledger: &Ledger) -> Result<(), StorageError> {
        Ok(())
    }
    
    fn load_ledger(&self) -> Result<Ledger, StorageError> {
        Ok(Ledger::default())
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn clear(&mut self) -> Result<(), StorageError> {
        self.wal_sequence = 0;
        Ok(())
    }
}

impl WALStorage for NoOpStorage {
    fn append_to_wal(&mut self, _entry: WALEntry) -> Result<(), StorageError> {
        self.wal_sequence += 1;
        Ok(())
    }
    
    fn commit_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        Ok(Vec::new())
    }
    
    fn rollback_wal(&mut self) -> Result<(), StorageError> {
        Ok(())
    }
    
    fn recover_from_wal(&mut self) -> Result<Vec<LedgerEvent>, StorageError> {
        Ok(Vec::new())
    }
    
    fn get_wal_sequence(&self) -> u64 {
        self.wal_sequence
    }
    
    fn truncate_wal(&mut self) -> Result<(), StorageError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{LedgerEvent, LedgerEventKind, ReasonCode};

    #[test]
    fn test_memory_wal_storage() {
        let mut storage = MemoryStorage::new();
        
        // Create a test event
        let event = LedgerEvent::new(
            1,
            LedgerEventKind::Mint {
                credits: 100,
                reason: ReasonCode::Custom("test".to_string()),
                memo: None,
            },
            None,
            None,
        );
        
        let wal_entry = WALEntry::new(1, event);
        
        // Test WAL operations
        storage.append_to_wal(wal_entry).unwrap();
        assert_eq!(storage.get_wal_sequence(), 1);
        
        let committed = storage.commit_wal().unwrap();
        assert_eq!(committed.len(), 1);
        assert!(committed[0].is_committed());
        
        storage.truncate_wal().unwrap();
        assert!(storage.is_available());
    }
} 