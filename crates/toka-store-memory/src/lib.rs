#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-memory** – In-memory storage driver for Toka OS.
//!
//! This crate provides a fast, non-persistent storage backend suitable for
//! testing, development, and scenarios where event persistence is not required.
//! All data is stored in memory and will be lost when the process terminates.
//!
//! This implementation now includes Write-Ahead Logging (WAL) support for enhanced
//! testing capabilities and consistent API with persistent backends.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use toka_store_core::{
    StorageBackend, EventHeader, EventId, CausalDigest,
    WriteAheadLog, WalEntry, WalOperation, WalEntryState, WalRecoveryResult,
    TransactionId, SequenceNumber,
};

/// Default buffer size for the live event broadcast channel.
const DEFAULT_BUFFER: usize = 1024;

//─────────────────────────────
//  In-memory storage backend with WAL
//─────────────────────────────

/// An in-memory, non-persistent event store with WAL support.
///
/// This implementation stores all events in memory using HashMap collections.
/// It provides excellent performance for read and write operations but offers
/// no persistence guarantees. All data is lost when the process terminates.
///
/// The storage backend also provides a live event stream via broadcast channels,
/// allowing subscribers to receive real-time notifications of committed events.
/// WAL support is provided for testing and API consistency with persistent backends.
#[derive(Debug, Clone)]
pub struct MemoryBackend {
    headers: Arc<RwLock<HashMap<EventId, EventHeader>>>,
    payloads: Arc<RwLock<HashMap<CausalDigest, Vec<u8>>>>,
    broadcast_tx: broadcast::Sender<EventHeader>,
    // WAL state management
    wal_entries: Arc<RwLock<HashMap<SequenceNumber, WalEntry>>>,
    wal_sequence: Arc<RwLock<SequenceNumber>>,
    active_transactions: Arc<RwLock<HashMap<TransactionId, WalTransactionState>>>,
}

/// State tracking for active WAL transactions.
#[derive(Debug, Clone)]
struct WalTransactionState {
    /// Transaction identifier
    transaction_id: TransactionId,
    /// Current state of the transaction
    state: WalTransactionStateType,
    /// Operations logged in this transaction
    operations: Vec<WalOperation>,
    /// Sequence numbers for this transaction's entries
    sequences: Vec<SequenceNumber>,
}

/// State types for WAL transactions.
#[derive(Debug, Clone, PartialEq)]
enum WalTransactionStateType {
    /// Transaction is active and accepting operations
    Active,
    /// Transaction is being committed
    Committing,
    /// Transaction has been committed
    Committed,
    /// Transaction is being rolled back
    RollingBack,
    /// Transaction has been rolled back
    RolledBack,
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBackend {
    /// Creates a new, empty memory storage backend.
    ///
    /// The backend starts with empty storage and a broadcast channel with
    /// the default buffer size for live event streaming.
    pub fn new() -> Self {
        Self::with_buffer_size(DEFAULT_BUFFER)
    }

    /// Creates a new memory backend with a custom broadcast buffer size.
    ///
    /// The buffer size determines how many events can be queued for slow
    /// subscribers before older events are dropped from the live stream.
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(buffer_size);
        Self {
            headers: Arc::new(RwLock::new(HashMap::new())),
            payloads: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            wal_entries: Arc::new(RwLock::new(HashMap::new())),
            wal_sequence: Arc::new(RwLock::new(0)),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the next sequence number for WAL entries.
    async fn next_sequence(&self) -> SequenceNumber {
        let mut seq = self.wal_sequence.write().await;
        *seq += 1;
        *seq
    }

    /// Subscribe to the live event stream.
    ///
    /// Returns a receiver that will receive copies of all event headers
    /// as they are committed to storage. Subscribers that fall behind
    /// may miss events if the broadcast buffer overflows.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.broadcast_tx.subscribe()
    }

    /// Get the current number of stored events.
    pub async fn event_count(&self) -> usize {
        self.headers.read().await.len()
    }

    /// Get the current number of unique payloads stored.
    ///
    /// This may be less than the event count due to payload deduplication
    /// when multiple events share the same content hash.
    pub async fn payload_count(&self) -> usize {
        self.payloads.read().await.len()
    }

    /// Get the current number of WAL entries.
    pub async fn wal_entry_count(&self) -> usize {
        self.wal_entries.read().await.len()
    }

    /// Clear all stored events and payloads.
    ///
    /// This operation is useful for testing and development scenarios
    /// where you need to reset the storage state.
    pub async fn clear(&self) {
        self.headers.write().await.clear();
        self.payloads.write().await.clear();
        self.wal_entries.write().await.clear();
        *self.wal_sequence.write().await = 0;
        self.active_transactions.write().await.clear();
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        // Store payload (deduplicated by digest)
        // Multiple headers can reference the same payload via shared digest
        self.payloads
            .write()
            .await
            .entry(header.digest)
            .or_insert_with(|| payload.to_vec());

        // Store header
        self.headers
            .write()
            .await
            .insert(header.id, header.clone());

        // Broadcast live update (ignore errors if no subscribers)
        let _ = self.broadcast_tx.send(header.clone());

        Ok(())
    }

    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        Ok(self.headers.read().await.get(id).cloned())
    }

    async fn payload_bytes(&self, digest: &CausalDigest) -> Result<Option<Vec<u8>>> {
        Ok(self.payloads.read().await.get(digest).cloned())
    }
}

#[async_trait]
impl WriteAheadLog for MemoryBackend {
    async fn begin_transaction(&self) -> Result<TransactionId> {
        let transaction_id = Uuid::new_v4();
        let sequence = self.next_sequence().await;
        
        // Create WAL entry for transaction begin
        let wal_entry = WalEntry {
            id: Uuid::new_v4(),
            transaction_id,
            sequence,
            timestamp: chrono::Utc::now(),
            operation: WalOperation::BeginTransaction { transaction_id },
            state: WalEntryState::Pending,
        };

        // Store WAL entry
        self.wal_entries.write().await.insert(sequence, wal_entry.clone());

        // Track transaction state
        let transaction_state = WalTransactionState {
            transaction_id,
            state: WalTransactionStateType::Active,
            operations: vec![wal_entry.operation],
            sequences: vec![sequence],
        };

        self.active_transactions
            .write()
            .await
            .insert(transaction_id, transaction_state);

        Ok(transaction_id)
    }

    async fn write_entry(
        &self,
        transaction_id: TransactionId,
        operation: WalOperation,
    ) -> Result<()> {
        // Check if transaction is active
        {
            let transactions = self.active_transactions.read().await;
            if let Some(tx_state) = transactions.get(&transaction_id) {
                if tx_state.state != WalTransactionStateType::Active {
                    return Err(anyhow::anyhow!(
                        "Transaction {} is not active (state: {:?})",
                        transaction_id,
                        tx_state.state
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Transaction {} not found",
                    transaction_id
                ));
            }
        }

        let sequence = self.next_sequence().await;
        
        // Create WAL entry
        let wal_entry = WalEntry {
            id: Uuid::new_v4(),
            transaction_id,
            sequence,
            timestamp: chrono::Utc::now(),
            operation: operation.clone(),
            state: WalEntryState::Pending,
        };

        // Store WAL entry
        self.wal_entries.write().await.insert(sequence, wal_entry);

        // Update transaction state
        {
            let mut transactions = self.active_transactions.write().await;
            if let Some(tx_state) = transactions.get_mut(&transaction_id) {
                tx_state.operations.push(operation);
                tx_state.sequences.push(sequence);
            }
        }

        Ok(())
    }

    async fn commit_transaction(&self, transaction_id: TransactionId) -> Result<()> {
        // Update transaction state to committing and get operations
        let operations = {
            let mut transactions = self.active_transactions.write().await;
            if let Some(tx_state) = transactions.get_mut(&transaction_id) {
                if tx_state.state != WalTransactionStateType::Active {
                    return Err(anyhow::anyhow!(
                        "Transaction {} is not active (state: {:?})",
                        transaction_id,
                        tx_state.state
                    ));
                }
                tx_state.state = WalTransactionStateType::Committing;
                tx_state.operations.clone()
            } else {
                return Err(anyhow::anyhow!(
                    "Transaction {} not found",
                    transaction_id
                ));
            }
        };

        // Apply all operations in this transaction to storage
        for operation in operations {
            match operation {
                WalOperation::CommitEvent { header, payload } => {
                    // Apply the event to storage
                    self.commit(&header, &payload).await?;
                }
                _ => {
                    // Other operations don't need to be applied to storage
                }
            }
        }

        // Log the commit transaction operation
        let commit_sequence = self.next_sequence().await;
        let commit_wal_entry = WalEntry {
            id: Uuid::new_v4(),
            transaction_id,
            sequence: commit_sequence,
            timestamp: chrono::Utc::now(),
            operation: WalOperation::CommitTransaction { transaction_id },
            state: WalEntryState::Committed,
        };
        self.wal_entries.write().await.insert(commit_sequence, commit_wal_entry);

        // Mark all WAL entries for this transaction as committed
        {
            let mut wal_entries = self.wal_entries.write().await;
            for entry in wal_entries.values_mut() {
                if entry.transaction_id == transaction_id {
                    entry.state = WalEntryState::Committed;
                }
            }
        }

        // Update transaction state to committed
        {
            let mut transactions = self.active_transactions.write().await;
            if let Some(tx_state) = transactions.get_mut(&transaction_id) {
                tx_state.state = WalTransactionStateType::Committed;
            }
        }

        Ok(())
    }

    async fn rollback_transaction(&self, transaction_id: TransactionId) -> Result<()> {
        // Update transaction state to rolling back
        {
            let mut transactions = self.active_transactions.write().await;
            if let Some(tx_state) = transactions.get_mut(&transaction_id) {
                if tx_state.state != WalTransactionStateType::Active {
                    return Err(anyhow::anyhow!(
                        "Transaction {} is not active (state: {:?})",
                        transaction_id,
                        tx_state.state
                    ));
                }
                tx_state.state = WalTransactionStateType::RollingBack;
            } else {
                return Err(anyhow::anyhow!(
                    "Transaction {} not found",
                    transaction_id
                ));
            }
        }

        // Log the rollback transaction operation
        let rollback_sequence = self.next_sequence().await;
        let rollback_wal_entry = WalEntry {
            id: Uuid::new_v4(),
            transaction_id,
            sequence: rollback_sequence,
            timestamp: chrono::Utc::now(),
            operation: WalOperation::RollbackTransaction { transaction_id },
            state: WalEntryState::RolledBack,
        };
        self.wal_entries.write().await.insert(rollback_sequence, rollback_wal_entry);

        // Mark all WAL entries for this transaction as rolled back
        {
            let mut wal_entries = self.wal_entries.write().await;
            for entry in wal_entries.values_mut() {
                if entry.transaction_id == transaction_id {
                    entry.state = WalEntryState::RolledBack;
                }
            }
        }

        // Update transaction state to rolled back
        {
            let mut transactions = self.active_transactions.write().await;
            if let Some(tx_state) = transactions.get_mut(&transaction_id) {
                tx_state.state = WalTransactionStateType::RolledBack;
            }
        }

        Ok(())
    }

    async fn recover(&self) -> Result<WalRecoveryResult> {
        let mut result = WalRecoveryResult {
            entries_recovered: 0,
            transactions_rolled_back: 0,
            transactions_committed: 0,
            entries_checkpointed: 0,
            recovery_errors: Vec::new(),
        };

        // Get all WAL entries ordered by sequence number (scope the read lock)
        let entries = {
            let wal_entries = self.wal_entries.read().await;
            let mut entries: Vec<_> = wal_entries.values().cloned().collect();
            entries.sort_by_key(|entry| entry.sequence);
            entries
        }; // Read lock is released here

        let mut transaction_states: HashMap<TransactionId, Vec<WalEntry>> = HashMap::new();

        // Group entries by transaction
        for entry in entries {
            transaction_states
                .entry(entry.transaction_id)
                .or_insert_with(Vec::new)
                .push(entry);
            result.entries_recovered += 1;
        }

        // Process transactions for recovery
        for (transaction_id, entries) in transaction_states {
            // Check if transaction has a commit entry
            let has_commit = entries.iter().any(|e| {
                matches!(e.operation, WalOperation::CommitTransaction { .. })
                    && e.state == WalEntryState::Committed
            });

            if has_commit {
                // Apply all committed operations
                for entry in entries {
                    if entry.state == WalEntryState::Committed {
                        match &entry.operation {
                            WalOperation::CommitEvent { header, payload } => {
                                if let Err(e) = self.commit(header, payload).await {
                                    result.recovery_errors.push(format!(
                                        "Failed to apply committed event: {}", e
                                    ));
                                }
                            }
                            _ => {} // Other operations don't need reapplication
                        }
                    }
                }
                result.transactions_committed += 1;
            } else {
                // Check if transaction is incomplete (no commit or rollback)
                let has_rollback = entries.iter().any(|e| {
                    matches!(e.operation, WalOperation::RollbackTransaction { .. })
                });

                if !has_rollback {
                    // Incomplete transaction - roll it back
                    if let Err(e) = self.rollback_transaction(transaction_id).await {
                        result.recovery_errors.push(format!(
                            "Failed to rollback transaction {}: {}", transaction_id, e
                        ));
                    } else {
                        result.transactions_rolled_back += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    async fn checkpoint(&self, sequence: SequenceNumber) -> Result<()> {
        // Mark entries up to sequence as checkpointed
        let mut wal_entries = self.wal_entries.write().await;
        let mut checkpointed_count = 0;
        
        for entry in wal_entries.values_mut() {
            if entry.sequence <= sequence && entry.state == WalEntryState::Committed {
                entry.state = WalEntryState::Checkpointed;
                checkpointed_count += 1;
            }
        }

        // Optionally remove old checkpointed entries to free memory
        // For now we keep them for consistency with SQLite backend
        
        Ok(())
    }

    async fn current_sequence(&self) -> Result<SequenceNumber> {
        Ok(*self.wal_sequence.read().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use toka_store_core::{create_event_header, prelude::*};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        message: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_basic_storage_operations() {
        let backend = MemoryBackend::new();
        
        let event = TestEvent {
            message: "test".to_string(),
            value: 42,
        };

        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.event".to_string(),
            &event,
        ).unwrap();

        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();

        // Commit event
        backend.commit(&header, &payload_bytes).await.unwrap();

        // Retrieve header
        let retrieved_header = backend.header(&header.id).await.unwrap().unwrap();
        assert_eq!(retrieved_header, header);

        // Retrieve payload
        let payload_bytes = backend
            .payload_bytes(&header.digest)
            .await
            .unwrap()
            .unwrap();
        let retrieved_event: TestEvent = rmp_serde::from_slice(&payload_bytes).unwrap();
        assert_eq!(retrieved_event, event);
    }

    #[tokio::test]
    async fn test_missing_events() {
        let backend = MemoryBackend::new();

        // Try to get non-existent header
        let result = backend.header(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());

        // Try to get non-existent payload
        let result = backend
            .payload_bytes(&[0u8; 32])
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_payload_deduplication() {
        let backend = MemoryBackend::new();

        let event = TestEvent {
            message: "duplicate".to_string(),
            value: 123,
        };

        // Create two headers with same payload content (will have same digest)
        let header1 = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.event".to_string(),
            &event,
        ).unwrap();

        let header2 = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.event".to_string(),
            &event,
        ).unwrap();

        // Same payload, same digest
        assert_eq!(header1.digest, header2.digest);

        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();

        // Commit both events
        backend.commit(&header1, &payload_bytes).await.unwrap();
        backend.commit(&header2, &payload_bytes).await.unwrap();

        // Should have 2 events but only 1 unique payload
        assert_eq!(backend.event_count().await, 2);
        assert_eq!(backend.payload_count().await, 1);
    }

    #[tokio::test]
    async fn test_live_event_stream() {
        let backend = MemoryBackend::new();
        let mut rx = backend.subscribe();

        let event = TestEvent {
            message: "live".to_string(),
            value: 999,
        };

        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.live".to_string(),
            &event,
        ).unwrap();

        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();

        // Commit event
        backend.commit(&header, &payload_bytes).await.unwrap();

        // Should receive the header via live stream
        let received = rx.recv().await.unwrap();
        assert_eq!(received, header);
    }

    #[tokio::test]
    async fn test_clear_storage() {
        let backend = MemoryBackend::new();

        let event = TestEvent {
            message: "to_be_cleared".to_string(),
            value: 1,
        };

        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.clear".to_string(),
            &event,
        ).unwrap();

        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();

        // Commit and verify storage
        backend.commit(&header, &payload_bytes).await.unwrap();
        assert_eq!(backend.event_count().await, 1);
        assert_eq!(backend.payload_count().await, 1);

        // Clear and verify empty
        backend.clear().await;
        assert_eq!(backend.event_count().await, 0);
        assert_eq!(backend.payload_count().await, 0);
        assert_eq!(backend.wal_entry_count().await, 0);

        // Verify event is gone
        let result = backend.header(&header.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_wal_basic_transaction() {
        let backend = MemoryBackend::new();
        
        // Begin a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        
        // Should have 1 WAL entry (BeginTransaction)
        assert_eq!(backend.wal_entry_count().await, 1);
        
        // Create an event
        let event = TestEvent {
            message: "wal test".to_string(),
            value: 42,
        };
        
        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.wal".to_string(),
            &event,
        ).unwrap();
        
        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();
        
        // Write the event to WAL
        backend.write_entry(
            tx_id,
            WalOperation::CommitEvent {
                header: header.clone(),
                payload: payload_bytes.clone(),
            },
        ).await.unwrap();
        
        // Should have 2 WAL entries now
        assert_eq!(backend.wal_entry_count().await, 2);
        
        // Commit the transaction
        backend.commit_transaction(tx_id).await.unwrap();
        
        // Should have 3 WAL entries (BeginTransaction + CommitEvent + CommitTransaction)
        assert_eq!(backend.wal_entry_count().await, 3);
        
        // Should have 1 event in storage (the CommitEvent was applied)
        assert_eq!(backend.event_count().await, 1);
    }

    #[tokio::test]
    async fn test_wal_rollback() {
        let backend = MemoryBackend::new();
        
        // Begin a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        
        // Create an event
        let event = TestEvent {
            message: "rollback test".to_string(),
            value: 99,
        };
        
        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.rollback".to_string(),
            &event,
        ).unwrap();
        
        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();
        
        // Write the event to WAL
        backend.write_entry(
            tx_id,
            WalOperation::CommitEvent {
                header: header.clone(),
                payload: payload_bytes.clone(),
            },
        ).await.unwrap();
        
        // Rollback the transaction
        backend.rollback_transaction(tx_id).await.unwrap();
        
        // Should have 3 WAL entries (BeginTransaction + CommitEvent + RollbackTransaction)
        assert_eq!(backend.wal_entry_count().await, 3);
        
        // Should have 0 events in storage (nothing was committed)
        assert_eq!(backend.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_wal_commit_with_wal() {
        let backend = MemoryBackend::new();
        
        // Begin a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        
        // Create an event
        let event = TestEvent {
            message: "commit with wal".to_string(),
            value: 123,
        };
        
        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.commit_wal".to_string(),
            &event,
        ).unwrap();
        
        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();
        
        // Use the WalStorageBackend trait method
        backend.commit_with_wal(tx_id, &header, &payload_bytes).await.unwrap();
        
        // Commit the transaction
        backend.commit_transaction(tx_id).await.unwrap();
        
        // Should have the event in storage
        assert_eq!(backend.event_count().await, 1);
        
        // Should be able to retrieve it
        let retrieved_header = backend.header(&header.id).await.unwrap().unwrap();
        assert_eq!(retrieved_header, header);
    }

    #[tokio::test]
    async fn test_wal_sequence_numbers() {
        let backend = MemoryBackend::new();
        
        // Current sequence should start at 0
        assert_eq!(backend.current_sequence().await.unwrap(), 0);
        
        // Begin a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        
        // Sequence should now be 1
        assert_eq!(backend.current_sequence().await.unwrap(), 1);
        
        // Write an entry
        backend.write_entry(
            tx_id,
            WalOperation::CommitEvent {
                header: create_event_header(
                    &[],
                    Uuid::new_v4(),
                    "test.sequence".to_string(),
                    &TestEvent { message: "seq".to_string(), value: 1 },
                ).unwrap(),
                payload: rmp_serde::to_vec_named(&TestEvent { message: "seq".to_string(), value: 1 }).unwrap(),
            },
        ).await.unwrap();
        
        // Sequence should now be 2
        assert_eq!(backend.current_sequence().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_wal_checkpoint() {
        let backend = MemoryBackend::new();
        
        // Begin and commit a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        backend.write_entry(
            tx_id,
            WalOperation::CommitEvent {
                header: create_event_header(
                    &[],
                    Uuid::new_v4(),
                    "test.checkpoint".to_string(),
                    &TestEvent { message: "checkpoint".to_string(), value: 42 },
                ).unwrap(),
                payload: rmp_serde::to_vec_named(&TestEvent { message: "checkpoint".to_string(), value: 42 }).unwrap(),
            },
        ).await.unwrap();
        backend.commit_transaction(tx_id).await.unwrap();
        
        // Get current sequence
        let current_seq = backend.current_sequence().await.unwrap();
        
        // Checkpoint up to current sequence
        backend.checkpoint(current_seq).await.unwrap();
        
        // Should still have WAL entries (we keep them for consistency)
        assert!(backend.wal_entry_count().await > 0);
    }

    #[tokio::test]
    async fn test_wal_recovery() {
        let backend = MemoryBackend::new();
        
        // Begin a transaction but don't commit
        let tx_id = backend.begin_transaction().await.unwrap();
        backend.write_entry(
            tx_id,
            WalOperation::CommitEvent {
                header: create_event_header(
                    &[],
                    Uuid::new_v4(),
                    "test.recovery".to_string(),
                    &TestEvent { message: "recovery".to_string(), value: 99 },
                ).unwrap(),
                payload: rmp_serde::to_vec_named(&TestEvent { message: "recovery".to_string(), value: 99 }).unwrap(),
            },
        ).await.unwrap();
        
        // Begin another transaction and commit it
        let tx_id2 = backend.begin_transaction().await.unwrap();
        backend.write_entry(
            tx_id2,
            WalOperation::CommitEvent {
                header: create_event_header(
                    &[],
                    Uuid::new_v4(),
                    "test.recovery2".to_string(),
                    &TestEvent { message: "recovery2".to_string(), value: 100 },
                ).unwrap(),
                payload: rmp_serde::to_vec_named(&TestEvent { message: "recovery2".to_string(), value: 100 }).unwrap(),
            },
        ).await.unwrap();
        backend.commit_transaction(tx_id2).await.unwrap();
        
        // Run recovery
        let recovery_result = backend.recover().await.unwrap();
        
        // Should have recovered some entries
        assert!(recovery_result.entries_recovered > 0);
        
        // Should have rolled back the uncommitted transaction
        assert_eq!(recovery_result.transactions_rolled_back, 1);
        
        // Should have committed the committed transaction
        assert_eq!(recovery_result.transactions_committed, 1);
    }
}