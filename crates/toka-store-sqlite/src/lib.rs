#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-sqlite** – SQLite-based persistent storage driver for Toka OS.
//!
//! This crate provides a reliable, portable storage backend using SQLite
//! database engine via sqlx. It offers ACID transactions, excellent portability,
//! and efficient storage while maintaining the same interface as other storage drivers.
//!
//! This implementation now includes Write-Ahead Logging (WAL) support for enhanced
//! durability and crash recovery capabilities.

use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{SqlitePool, Sqlite, Row};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use toka_store_core::{
    StorageBackend, EventHeader, EventId, CausalDigest,
    WriteAheadLog, WalEntry, WalOperation, WalEntryState, WalRecoveryResult,
    TransactionId, SequenceNumber, StorageError,
};

/// Default broadcast channel size for live event streaming.
const DEFAULT_BROADCAST_SIZE: usize = 256;

//─────────────────────────────
//  SQLite storage backend with WAL
//─────────────────────────────

/// A persistent storage backend using SQLite database with WAL support.
///
/// This implementation provides durable storage with ACID guarantees,
/// excellent portability, and efficient on-disk representation.
/// The database uses multiple tables: event headers, payloads, and WAL entries
/// with automatic deduplication of payloads by content hash.
#[derive(Debug)]
pub struct SqliteBackend {
    pool: SqlitePool,
    broadcast_tx: broadcast::Sender<EventHeader>,
    // WAL state management
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

impl SqliteBackend {
    /// Opens or creates a new SQLite database at the specified path.
    ///
    /// The database will be created if it doesn't exist. This operation
    /// will also run database migrations to ensure the schema is up to date.
    ///
    /// # Arguments
    /// * `path` - File system path where the database should be stored
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or created.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        use sqlx::sqlite::SqliteConnectOptions;
        
        let opts = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            // Enable WAL mode for better concurrency
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        
        let pool = SqlitePool::connect_with(opts).await?;
        Self::from_pool(pool).await
    }

    /// Opens an in-memory SQLite database.
    ///
    /// This creates a database that exists only in memory and will be
    /// lost when the connection is closed. Useful for testing and
    /// temporary storage scenarios.
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        Self::from_pool(pool).await
    }

    /// Creates a new backend from an existing SQLite pool.
    ///
    /// This allows sharing a database connection pool across multiple
    /// components or when using custom sqlx configurations.
    pub async fn from_pool(pool: SqlitePool) -> Result<Self> {
        let backend = Self {
            pool,
            broadcast_tx: broadcast::channel(DEFAULT_BROADCAST_SIZE).0,
            wal_sequence: Arc::new(RwLock::new(0)),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
        };

        backend.migrate().await?;
        backend.initialize_wal_sequence().await?;
        Ok(backend)
    }

    /// Run database migrations to ensure schema is current.
    async fn migrate(&self) -> Result<()> {
        // Create headers table
        sqlx::query::<Sqlite>(
            r#"
            CREATE TABLE IF NOT EXISTS event_headers (
                id BLOB PRIMARY KEY,
                header_data BLOB NOT NULL,
                timestamp TEXT NOT NULL,
                intent TEXT NOT NULL,
                kind TEXT NOT NULL
            ) STRICT
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create payloads table with deduplication by digest
        sqlx::query::<Sqlite>(
            r#"
            CREATE TABLE IF NOT EXISTS event_payloads (
                digest BLOB PRIMARY KEY,
                payload_data BLOB NOT NULL
            ) STRICT
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create WAL entries table
        sqlx::query::<Sqlite>(
            r#"
            CREATE TABLE IF NOT EXISTS wal_entries (
                id BLOB PRIMARY KEY,
                transaction_id BLOB NOT NULL,
                sequence_number INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                operation_data BLOB NOT NULL,
                state INTEGER NOT NULL,
                UNIQUE(sequence_number)
            ) STRICT
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_headers_timestamp ON event_headers(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_headers_intent ON event_headers(intent)")
            .execute(&self.pool)
            .await?;

        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_headers_kind ON event_headers(kind)")
            .execute(&self.pool)
            .await?;

        // WAL-specific indexes
        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_wal_transaction ON wal_entries(transaction_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_wal_sequence ON wal_entries(sequence_number)")
            .execute(&self.pool)
            .await?;

        sqlx::query::<Sqlite>("CREATE INDEX IF NOT EXISTS idx_wal_state ON wal_entries(state)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Initialize the WAL sequence number from the database.
    async fn initialize_wal_sequence(&self) -> Result<()> {
        let row = sqlx::query::<Sqlite>(
            "SELECT COALESCE(MAX(sequence_number), 0) as max_seq FROM wal_entries"
        )
        .fetch_one(&self.pool)
        .await?;

        let max_seq: i64 = row.get("max_seq");
        *self.wal_sequence.write().await = max_seq as SequenceNumber;
        Ok(())
    }

    /// Get the next sequence number for WAL entries.
    async fn next_sequence(&self) -> SequenceNumber {
        let mut seq = self.wal_sequence.write().await;
        *seq += 1;
        *seq
    }

    /// Convert WalEntryState to integer for storage.
    fn state_to_int(state: WalEntryState) -> i32 {
        match state {
            WalEntryState::Pending => 0,
            WalEntryState::Committed => 1,
            WalEntryState::RolledBack => 2,
            WalEntryState::Checkpointed => 3,
        }
    }

    /// Convert integer to WalEntryState for retrieval.
    fn int_to_state(value: i32) -> WalEntryState {
        match value {
            0 => WalEntryState::Pending,
            1 => WalEntryState::Committed,
            2 => WalEntryState::RolledBack,
            3 => WalEntryState::Checkpointed,
            _ => WalEntryState::Pending, // Default fallback
        }
    }

    /// Subscribe to the live event stream.
    ///
    /// Returns a receiver that will receive copies of all event headers
    /// as they are committed to storage. Subscribers that fall behind
    /// may miss events if the broadcast buffer overflows.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.broadcast_tx.subscribe()
    }

    /// Get the total number of events stored in the database.
    pub async fn event_count(&self) -> Result<i64> {
        let row = sqlx::query::<Sqlite>("SELECT COUNT(*) as count FROM event_headers")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    /// Get the total number of unique payloads stored.
    ///
    /// This may be less than the event count due to payload deduplication
    /// when multiple events share the same content hash.
    pub async fn payload_count(&self) -> Result<i64> {
        let row = sqlx::query::<Sqlite>("SELECT COUNT(*) as count FROM event_payloads")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    /// Get the total number of WAL entries.
    pub async fn wal_entry_count(&self) -> Result<i64> {
        let row = sqlx::query::<Sqlite>("SELECT COUNT(*) as count FROM wal_entries")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    /// Close the database connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[async_trait]
impl StorageBackend for SqliteBackend {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Store payload (deduplicated by digest)
        // Use INSERT OR IGNORE to avoid errors on duplicate digests
        sqlx::query::<Sqlite>(
            "INSERT OR IGNORE INTO event_payloads (digest, payload_data) VALUES (?, ?)"
        )
        .bind(&header.digest[..])
        .bind(payload)
        .execute(&mut *tx)
        .await?;

        // Store header (may overwrite previous version with same ID)
        let header_bytes = rmp_serde::to_vec_named(header)?;
        sqlx::query::<Sqlite>(
            r#"
            INSERT OR REPLACE INTO event_headers 
            (id, header_data, timestamp, intent, kind) 
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(header.id)
        .bind(&header_bytes)
        .bind(header.timestamp.to_rfc3339())
        .bind(header.intent.to_string())
        .bind(&header.kind)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Broadcast live update (ignore errors if no subscribers)
        let _ = self.broadcast_tx.send(header.clone());

        Ok(())
    }

    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        let row = sqlx::query::<Sqlite>(
            "SELECT header_data FROM event_headers WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let header_bytes: Vec<u8> = row.get("header_data");
                let header: EventHeader = rmp_serde::from_slice(&header_bytes)?;
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    async fn payload_bytes(&self, digest: &CausalDigest) -> Result<Option<Vec<u8>>> {
        let row = sqlx::query::<Sqlite>(
            "SELECT payload_data FROM event_payloads WHERE digest = ?"
        )
        .bind(&digest[..])
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row.get("payload_data"))),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl WriteAheadLog for SqliteBackend {
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
        let operation_bytes = rmp_serde::to_vec_named(&wal_entry.operation)?;
        sqlx::query::<Sqlite>(
            r#"
            INSERT INTO wal_entries 
            (id, transaction_id, sequence_number, timestamp, operation_data, state) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(wal_entry.id)
        .bind(transaction_id)
        .bind(sequence as i64)
        .bind(wal_entry.timestamp.to_rfc3339())
        .bind(&operation_bytes)
        .bind(Self::state_to_int(wal_entry.state))
        .execute(&self.pool)
        .await?;

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
        let operation_bytes = rmp_serde::to_vec_named(&wal_entry.operation)?;
        sqlx::query::<Sqlite>(
            r#"
            INSERT INTO wal_entries 
            (id, transaction_id, sequence_number, timestamp, operation_data, state) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(wal_entry.id)
        .bind(transaction_id)
        .bind(sequence as i64)
        .bind(wal_entry.timestamp.to_rfc3339())
        .bind(&operation_bytes)
        .bind(Self::state_to_int(wal_entry.state))
        .execute(&self.pool)
        .await?;

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
        // Update transaction state to committing
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
        let commit_operation_bytes = rmp_serde::to_vec_named(&WalOperation::CommitTransaction { transaction_id })?;
        sqlx::query::<Sqlite>(
            r#"
            INSERT INTO wal_entries 
            (id, transaction_id, sequence_number, timestamp, operation_data, state) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(transaction_id)
        .bind(commit_sequence as i64)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&commit_operation_bytes)
        .bind(Self::state_to_int(WalEntryState::Committed))
        .execute(&self.pool)
        .await?;

        // Mark all WAL entries for this transaction as committed
        sqlx::query::<Sqlite>(
            "UPDATE wal_entries SET state = ? WHERE transaction_id = ?"
        )
        .bind(Self::state_to_int(WalEntryState::Committed))
        .bind(transaction_id)
        .execute(&self.pool)
        .await?;

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
        let rollback_operation_bytes = rmp_serde::to_vec_named(&WalOperation::RollbackTransaction { transaction_id })?;
        sqlx::query::<Sqlite>(
            r#"
            INSERT INTO wal_entries 
            (id, transaction_id, sequence_number, timestamp, operation_data, state) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(transaction_id)
        .bind(rollback_sequence as i64)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&rollback_operation_bytes)
        .bind(Self::state_to_int(WalEntryState::RolledBack))
        .execute(&self.pool)
        .await?;

        // Mark all WAL entries for this transaction as rolled back
        sqlx::query::<Sqlite>(
            "UPDATE wal_entries SET state = ? WHERE transaction_id = ?"
        )
        .bind(Self::state_to_int(WalEntryState::RolledBack))
        .bind(transaction_id)
        .execute(&self.pool)
        .await?;

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

        // Get all WAL entries ordered by sequence number
        let rows = sqlx::query::<Sqlite>(
            r#"
            SELECT id, transaction_id, sequence_number, timestamp, operation_data, state
            FROM wal_entries 
            ORDER BY sequence_number ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut transaction_states: HashMap<TransactionId, Vec<WalEntry>> = HashMap::new();

        // Parse all WAL entries
        for row in rows {
            let entry_id: Uuid = row.get("id");
            let transaction_id: Uuid = row.get("transaction_id");
            let sequence_number: i64 = row.get("sequence_number");
            let timestamp_str: String = row.get("timestamp");
            let operation_bytes: Vec<u8> = row.get("operation_data");
            let state_int: i32 = row.get("state");

            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| anyhow::anyhow!("Invalid timestamp: {}", e))?
                .with_timezone(&chrono::Utc);

            let operation: WalOperation = match rmp_serde::from_slice(&operation_bytes) {
                Ok(op) => op,
                Err(e) => {
                    result.recovery_errors.push(format!("Failed to deserialize operation: {}", e));
                    continue;
                }
            };

            let state = Self::int_to_state(state_int);

            let wal_entry = WalEntry {
                id: entry_id,
                transaction_id,
                sequence: sequence_number as SequenceNumber,
                timestamp,
                operation,
                state,
            };

            transaction_states
                .entry(transaction_id)
                .or_insert_with(Vec::new)
                .push(wal_entry);

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
                // Roll back uncommitted transactions
                if let Err(e) = self.rollback_transaction(transaction_id).await {
                    result.recovery_errors.push(format!(
                        "Failed to rollback transaction {}: {}", transaction_id, e
                    ));
                } else {
                    result.transactions_rolled_back += 1;
                }
            }
        }

        Ok(result)
    }

    async fn checkpoint(&self, sequence: SequenceNumber) -> Result<()> {
        // Mark entries up to sequence as checkpointed
        let rows_affected = sqlx::query::<Sqlite>(
            "UPDATE wal_entries SET state = ? WHERE sequence_number <= ? AND state = ?"
        )
        .bind(Self::state_to_int(WalEntryState::Checkpointed))
        .bind(sequence as i64)
        .bind(Self::state_to_int(WalEntryState::Committed))
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Optionally remove old checkpointed entries to free space
        // This is a policy decision - for now we keep them for audit purposes
        
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
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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
        let backend = SqliteBackend::in_memory().await.unwrap();

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
        let backend = SqliteBackend::in_memory().await.unwrap();

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
        assert_eq!(backend.event_count().await.unwrap(), 2);
        assert_eq!(backend.payload_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let event = TestEvent {
            message: "persistent".to_string(),
            value: 999,
        };

        let header = create_event_header(
            &[],
            Uuid::new_v4(),
            "test.persist".to_string(),
            &event,
        ).unwrap();

        let payload_bytes = rmp_serde::to_vec_named(&event).unwrap();

        // Store in first backend instance
        {
            let backend = SqliteBackend::open(&db_path).await.unwrap();
            backend.commit(&header, &payload_bytes).await.unwrap();
            backend.close().await;
        }

        // Reopen and verify data persisted
        {
            let backend = SqliteBackend::open(&db_path).await.unwrap();
            let retrieved_header = backend.header(&header.id).await.unwrap().unwrap();
            assert_eq!(retrieved_header, header);

            let payload_bytes = backend
                .payload_bytes(&header.digest)
                .await
                .unwrap()
                .unwrap();
            let retrieved_event: TestEvent = rmp_serde::from_slice(&payload_bytes).unwrap();
            assert_eq!(retrieved_event, event);
        }
    }

    #[tokio::test]
    async fn test_live_event_stream() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        let mut rx = backend.subscribe();

        let event = TestEvent {
            message: "live".to_string(),
            value: 777,
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
    async fn test_counts() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
        // Should start with zero counts
        assert_eq!(backend.event_count().await.unwrap(), 0);
        assert_eq!(backend.payload_count().await.unwrap(), 0);
        assert_eq!(backend.wal_entry_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_wal_basic_transaction() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
        // Begin a transaction
        let tx_id = backend.begin_transaction().await.unwrap();
        
        // Should have 1 WAL entry (BeginTransaction)
        assert_eq!(backend.wal_entry_count().await.unwrap(), 1);
        
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
        assert_eq!(backend.wal_entry_count().await.unwrap(), 2);
        
        // Commit the transaction
        backend.commit_transaction(tx_id).await.unwrap();
        
        // Should have 3 WAL entries (BeginTransaction + CommitEvent + CommitTransaction)
        assert_eq!(backend.wal_entry_count().await.unwrap(), 3);
        
        // Should have 1 event in storage (the CommitEvent was applied)
        assert_eq!(backend.event_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_wal_rollback() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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
        assert_eq!(backend.wal_entry_count().await.unwrap(), 3);
        
        // Should have 0 events in storage (nothing was committed)
        assert_eq!(backend.event_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_wal_commit_with_wal() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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
        assert_eq!(backend.event_count().await.unwrap(), 1);
        
        // Should be able to retrieve it
        let retrieved_header = backend.header(&header.id).await.unwrap().unwrap();
        assert_eq!(retrieved_header, header);
    }

    #[tokio::test]
    async fn test_wal_sequence_numbers() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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
        
        // Should still have WAL entries (we keep them for audit)
        assert!(backend.wal_entry_count().await.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_wal_recovery() {
        let backend = SqliteBackend::in_memory().await.unwrap();
        
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