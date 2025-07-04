//! Log module for Raft consensus algorithm.
//!
//! This module provides log entry structures and operations for the Raft replicated log.
//! The log is the core data structure that maintains the sequence of commands and ensures
//! consistency across all nodes in the cluster.

use crate::error::{RaftError, RaftResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type alias for log indices
pub type LogIndex = u64;

/// Type alias for Raft terms
pub type Term = u64;

/// A single entry in the Raft log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    /// The term when this entry was created
    pub term: Term,
    
    /// The index of this entry in the log
    pub index: LogIndex,
    
    /// The command data for this entry
    pub data: Vec<u8>,
    
    /// Timestamp when this entry was created
    pub timestamp: DateTime<Utc>,
    
    /// Type of the log entry
    pub entry_type: LogEntryType,
}

/// Type of log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogEntryType {
    /// Regular application command
    Command,
    
    /// Configuration change entry
    Configuration,
    
    /// No-op entry (used for establishing leadership)
    NoOp,
    
    /// Snapshot marker
    Snapshot,
}

impl LogEntry {
    /// Create a new command log entry
    pub fn new_command(term: Term, index: LogIndex, data: Vec<u8>) -> Self {
        Self {
            term,
            index,
            data,
            timestamp: Utc::now(),
            entry_type: LogEntryType::Command,
        }
    }

    /// Create a new configuration change entry
    pub fn new_configuration(term: Term, index: LogIndex, data: Vec<u8>) -> Self {
        Self {
            term,
            index,
            data,
            timestamp: Utc::now(),
            entry_type: LogEntryType::Configuration,
        }
    }

    /// Create a new no-op entry
    pub fn new_noop(term: Term, index: LogIndex) -> Self {
        Self {
            term,
            index,
            data: vec![],
            timestamp: Utc::now(),
            entry_type: LogEntryType::NoOp,
        }
    }

    /// Create a new snapshot marker entry
    pub fn new_snapshot(term: Term, index: LogIndex, data: Vec<u8>) -> Self {
        Self {
            term,
            index,
            data,
            timestamp: Utc::now(),
            entry_type: LogEntryType::Snapshot,
        }
    }

    /// Check if this entry is a command
    pub fn is_command(&self) -> bool {
        matches!(self.entry_type, LogEntryType::Command)
    }

    /// Check if this entry is a configuration change
    pub fn is_configuration(&self) -> bool {
        matches!(self.entry_type, LogEntryType::Configuration)
    }

    /// Check if this entry is a no-op
    pub fn is_noop(&self) -> bool {
        matches!(self.entry_type, LogEntryType::NoOp)
    }

    /// Check if this entry is a snapshot
    pub fn is_snapshot(&self) -> bool {
        matches!(self.entry_type, LogEntryType::Snapshot)
    }
}

/// In-memory log implementation
#[derive(Debug, Clone)]
pub struct Log {
    /// The log entries
    entries: VecDeque<LogEntry>,
    
    /// Index of the first entry in the log
    first_index: LogIndex,
    
    /// Index of the last committed entry
    commit_index: LogIndex,
    
    /// Index of the last applied entry
    last_applied: LogIndex,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            first_index: 1,
            commit_index: 0,
            last_applied: 0,
        }
    }
}

impl Log {
    /// Create a new empty log
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the last log index
    pub fn last_log_index(&self) -> LogIndex {
        if self.entries.is_empty() {
            self.first_index - 1
        } else {
            self.first_index + self.entries.len() as LogIndex - 1
        }
    }

    /// Get the last log term
    pub fn last_log_term(&self) -> Term {
        self.entries.back().map(|e| e.term).unwrap_or(0)
    }

    /// Get the first log index
    pub fn first_log_index(&self) -> LogIndex {
        self.first_index
    }

    /// Get the commit index
    pub fn commit_index(&self) -> LogIndex {
        self.commit_index
    }

    /// Get the last applied index
    pub fn last_applied(&self) -> LogIndex {
        self.last_applied
    }

    /// Set the commit index
    pub fn set_commit_index(&mut self, index: LogIndex) -> RaftResult<()> {
        if index > self.last_log_index() {
            return Err(RaftError::LogEntryNotFound { index });
        }
        
        if index < self.commit_index {
            return Err(RaftError::internal(
                "Cannot decrease commit index"
            ));
        }
        
        self.commit_index = index;
        Ok(())
    }

    /// Set the last applied index
    pub fn set_last_applied(&mut self, index: LogIndex) -> RaftResult<()> {
        if index > self.commit_index {
            return Err(RaftError::internal(
                "Cannot apply entries beyond commit index"
            ));
        }
        
        self.last_applied = index;
        Ok(())
    }

    /// Get a log entry by index
    pub fn get_entry(&self, index: LogIndex) -> RaftResult<&LogEntry> {
        if index < self.first_index {
            return Err(RaftError::LogEntryNotFound { index });
        }
        
        let offset = (index - self.first_index) as usize;
        self.entries.get(offset).ok_or(RaftError::LogEntryNotFound { index })
    }

    /// Get the term of a log entry by index
    pub fn get_term(&self, index: LogIndex) -> RaftResult<Term> {
        if index == 0 {
            return Ok(0);
        }
        
        self.get_entry(index).map(|e| e.term)
    }

    /// Append a new entry to the log
    pub fn append_entry(&mut self, entry: LogEntry) -> RaftResult<()> {
        let expected_index = self.last_log_index() + 1;
        if entry.index != expected_index {
            return Err(RaftError::internal(
                format!("Entry index {} does not match expected {}", entry.index, expected_index)
            ));
        }
        
        self.entries.push_back(entry);
        Ok(())
    }

    /// Append multiple entries to the log
    pub fn append_entries(&mut self, entries: Vec<LogEntry>) -> RaftResult<()> {
        for entry in entries {
            self.append_entry(entry)?;
        }
        Ok(())
    }

    /// Remove entries from the given index onwards
    pub fn truncate_from(&mut self, index: LogIndex) -> RaftResult<()> {
        if index < self.first_index {
            return Err(RaftError::LogEntryNotFound { index });
        }
        
        let offset = (index - self.first_index) as usize;
        self.entries.truncate(offset);
        
        // Update commit index if necessary
        if self.commit_index >= index {
            self.commit_index = if index > 0 { index - 1 } else { 0 };
        }
        
        // Update last applied if necessary
        if self.last_applied >= index {
            self.last_applied = if index > 0 { index - 1 } else { 0 };
        }
        
        Ok(())
    }

    /// Get entries from start_index to end_index (inclusive)
    pub fn get_entries(&self, start_index: LogIndex, end_index: LogIndex) -> RaftResult<Vec<LogEntry>> {
        if start_index > end_index {
            return Ok(vec![]);
        }
        
        if start_index < self.first_index {
            return Err(RaftError::LogEntryNotFound { index: start_index });
        }
        
        let start_offset = (start_index - self.first_index) as usize;
        let end_offset = (end_index - self.first_index) as usize;
        
        if start_offset >= self.entries.len() {
            return Ok(vec![]);
        }
        
        let actual_end_offset = std::cmp::min(end_offset + 1, self.entries.len());
        
        Ok(self.entries.range(start_offset..actual_end_offset).cloned().collect())
    }

    /// Get entries from start_index onwards, up to max_entries
    pub fn get_entries_from(&self, start_index: LogIndex, max_entries: usize) -> RaftResult<Vec<LogEntry>> {
        if start_index < self.first_index {
            return Err(RaftError::LogEntryNotFound { index: start_index });
        }
        
        let start_offset = (start_index - self.first_index) as usize;
        
        if start_offset >= self.entries.len() {
            return Ok(vec![]);
        }
        
        let end_offset = std::cmp::min(start_offset + max_entries, self.entries.len());
        
        Ok(self.entries.range(start_offset..end_offset).cloned().collect())
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries in the log
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get uncommitted entries
    pub fn uncommitted_entries(&self) -> RaftResult<Vec<LogEntry>> {
        if self.commit_index >= self.last_log_index() {
            return Ok(vec![]);
        }
        
        self.get_entries(self.commit_index + 1, self.last_log_index())
    }

    /// Get unapplied entries
    pub fn unapplied_entries(&self) -> RaftResult<Vec<LogEntry>> {
        if self.last_applied >= self.commit_index {
            return Ok(vec![]);
        }
        
        self.get_entries(self.last_applied + 1, self.commit_index)
    }

    /// Compact the log by removing entries up to the given index
    pub fn compact(&mut self, index: LogIndex) -> RaftResult<()> {
        if index < self.first_index {
            return Ok(());
        }
        
        if index > self.last_log_index() {
            return Err(RaftError::LogEntryNotFound { index });
        }
        
        let remove_count = (index - self.first_index + 1) as usize;
        for _ in 0..remove_count {
            self.entries.pop_front();
        }
        
        self.first_index = index + 1;
        Ok(())
    }

    /// Check if the log matches at the given index and term
    pub fn matches(&self, index: LogIndex, term: Term) -> bool {
        if index == 0 {
            return term == 0;
        }
        
        match self.get_term(index) {
            Ok(t) => t == term,
            Err(_) => false,
        }
    }

    /// Find the index of the last entry with the given term
    pub fn find_last_entry_with_term(&self, term: Term) -> Option<LogIndex> {
        self.entries
            .iter()
            .rposition(|e| e.term == term)
            .map(|offset| self.first_index + offset as LogIndex)
    }

    /// Get log statistics
    pub fn stats(&self) -> LogStats {
        LogStats {
            total_entries: self.len(),
            first_index: self.first_index,
            last_index: self.last_log_index(),
            commit_index: self.commit_index,
            last_applied: self.last_applied,
            last_term: self.last_log_term(),
        }
    }
}

/// Statistics about the log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    /// Total number of entries in the log
    pub total_entries: usize,
    
    /// Index of the first entry
    pub first_index: LogIndex,
    
    /// Index of the last entry
    pub last_index: LogIndex,
    
    /// Index of the last committed entry
    pub commit_index: LogIndex,
    
    /// Index of the last applied entry
    pub last_applied: LogIndex,
    
    /// Term of the last entry
    pub last_term: Term,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new_command(1, 1, b"test".to_vec());
        assert_eq!(entry.term, 1);
        assert_eq!(entry.index, 1);
        assert_eq!(entry.data, b"test");
        assert!(entry.is_command());
        assert!(!entry.is_noop());
    }

    #[test]
    fn test_empty_log() {
        let log = Log::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert_eq!(log.last_log_index(), 0);
        assert_eq!(log.last_log_term(), 0);
        assert_eq!(log.first_log_index(), 1);
    }

    #[test]
    fn test_log_append() {
        let mut log = Log::new();
        let entry1 = LogEntry::new_command(1, 1, b"cmd1".to_vec());
        let entry2 = LogEntry::new_command(1, 2, b"cmd2".to_vec());
        
        assert!(log.append_entry(entry1).is_ok());
        assert!(log.append_entry(entry2).is_ok());
        
        assert_eq!(log.len(), 2);
        assert_eq!(log.last_log_index(), 2);
        assert_eq!(log.last_log_term(), 1);
    }

    #[test]
    fn test_log_get_entry() {
        let mut log = Log::new();
        let entry = LogEntry::new_command(1, 1, b"test".to_vec());
        log.append_entry(entry.clone()).unwrap();
        
        let retrieved = log.get_entry(1).unwrap();
        assert_eq!(retrieved, &entry);
        
        assert!(log.get_entry(2).is_err());
    }

    #[test]
    fn test_log_truncate() {
        let mut log = Log::new();
        for i in 1..=5 {
            let entry = LogEntry::new_command(1, i, format!("cmd{}", i).into_bytes());
            log.append_entry(entry).unwrap();
        }
        
        assert_eq!(log.len(), 5);
        
        log.truncate_from(3).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log.last_log_index(), 2);
    }

    #[test]
    fn test_log_get_entries() {
        let mut log = Log::new();
        for i in 1..=5 {
            let entry = LogEntry::new_command(1, i, format!("cmd{}", i).into_bytes());
            log.append_entry(entry).unwrap();
        }
        
        let entries = log.get_entries(2, 4).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 2);
        assert_eq!(entries[2].index, 4);
    }

    #[test]
    fn test_log_commit_index() {
        let mut log = Log::new();
        for i in 1..=3 {
            let entry = LogEntry::new_command(1, i, format!("cmd{}", i).into_bytes());
            log.append_entry(entry).unwrap();
        }
        
        assert_eq!(log.commit_index(), 0);
        log.set_commit_index(2).unwrap();
        assert_eq!(log.commit_index(), 2);
        
        // Can't commit beyond last log index
        assert!(log.set_commit_index(5).is_err());
    }

    #[test]
    fn test_log_matches() {
        let mut log = Log::new();
        let entry1 = LogEntry::new_command(1, 1, b"cmd1".to_vec());
        let entry2 = LogEntry::new_command(2, 2, b"cmd2".to_vec());
        
        log.append_entry(entry1).unwrap();
        log.append_entry(entry2).unwrap();
        
        assert!(log.matches(1, 1));
        assert!(log.matches(2, 2));
        assert!(!log.matches(1, 2));
        assert!(!log.matches(2, 1));
        assert!(log.matches(0, 0));
    }

    #[test]
    fn test_log_compact() {
        let mut log = Log::new();
        for i in 1..=10 {
            let entry = LogEntry::new_command(1, i, format!("cmd{}", i).into_bytes());
            log.append_entry(entry).unwrap();
        }
        
        log.compact(5).unwrap();
        assert_eq!(log.first_log_index(), 6);
        assert_eq!(log.last_log_index(), 10);
        assert_eq!(log.len(), 5);
        
        // Can still get entries after compaction
        let entry = log.get_entry(6).unwrap();
        assert_eq!(entry.index, 6);
        
        // Can't get compacted entries
        assert!(log.get_entry(5).is_err());
    }
}