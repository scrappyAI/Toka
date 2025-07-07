#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-core** – Core storage abstractions for Toka OS.
//!
//! This crate provides the fundamental storage traits and event primitives used
//! throughout the Toka ecosystem. It sits at the core layer and defines the
//! contracts for event persistence without providing concrete implementations.
//!
//! Storage drivers (sled, SQLite, in-memory, etc.) implement these traits in
//! separate crates that depend on this core abstraction.

use std::vec::Vec;
use core::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

//─────────────────────────────
//  Core type aliases
//─────────────────────────────

/// Unique identifier for a committed event (UUID v4).
pub type EventId = Uuid;

/// Semantic identifier representing a high-level intent or task cluster.
pub type IntentId = Uuid;

/// Blake3 digest representing the causal hash chain of an event.
pub type CausalDigest = [u8; 32];

/// Unique identifier for a WAL transaction (UUID v4).
pub type TransactionId = Uuid;

/// Sequence number for WAL entries to ensure ordering.
pub type SequenceNumber = u64;

//─────────────────────────────
//  Event payload trait
//─────────────────────────────

/// Marker trait implemented by all serializable event payloads.
///
/// This trait is automatically implemented for any type that satisfies the
/// required bounds. It serves as a type-safe marker to ensure only appropriate
/// types can be used as event payloads.
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

//─────────────────────────────
//  Event header
//─────────────────────────────

/// Minimal header stored inline with every event.
///
/// The header contains all the metadata needed to identify, order, and verify
/// an event without needing to deserialize its payload. This supports efficient
/// queries and indexing operations.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventHeader {
    /// Event identifier (UUID v4)
    pub id: EventId,
    /// Parent event IDs this event causally depends on (can be empty)
    pub parents: SmallVec<[EventId; 4]>,
    /// Wall-clock timestamp when the event was committed
    pub timestamp: DateTime<Utc>,
    /// Blake3 digest of the event payload and its causal parent digests
    pub digest: CausalDigest,
    /// Semantic intent bucket this event belongs to
    pub intent: IntentId,
    /// Application-defined kind, e.g. `ledger.mint` or `agent.spawn`
    pub kind: String,
}

//─────────────────────────────
//  Write-Ahead Logging (WAL) Support
//─────────────────────────────

/// Represents a single entry in the Write-Ahead Log.
///
/// WAL entries track all operations that modify the storage state,
/// enabling crash recovery and providing durability guarantees.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalEntry {
    /// Unique identifier for this WAL entry
    pub id: Uuid,
    /// Transaction this entry belongs to
    pub transaction_id: TransactionId,
    /// Sequence number for ordering within the transaction
    pub sequence: SequenceNumber,
    /// Timestamp when this entry was created
    pub timestamp: DateTime<Utc>,
    /// The operation being logged
    pub operation: WalOperation,
    /// Current state of this entry
    pub state: WalEntryState,
}

/// Types of operations that can be logged in the WAL.
///
/// Each operation type corresponds to a specific storage modification
/// and includes all necessary information for recovery.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WalOperation {
    /// Begin a new transaction
    BeginTransaction {
        /// Transaction identifier
        transaction_id: TransactionId,
    },
    /// Commit an event (header + payload)
    CommitEvent {
        /// Event header to be committed
        header: EventHeader,
        /// Serialized payload bytes
        payload: Vec<u8>,
    },
    /// Commit a transaction (make all changes durable)
    CommitTransaction {
        /// Transaction identifier
        transaction_id: TransactionId,
    },
    /// Rollback a transaction (discard all changes)
    RollbackTransaction {
        /// Transaction identifier
        transaction_id: TransactionId,
    },
    /// Mark a WAL entry as checkpointed (can be safely removed)
    Checkpoint {
        /// Sequence number up to which entries are checkpointed
        sequence: SequenceNumber,
    },
}

/// State of a WAL entry during processing.
///
/// This tracks the lifecycle of each entry and enables proper
/// recovery behavior during crash scenarios.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WalEntryState {
    /// Entry is pending (not yet committed)
    Pending,
    /// Entry has been committed to storage
    Committed,
    /// Entry has been rolled back
    RolledBack,
    /// Entry has been checkpointed and can be removed
    Checkpointed,
}

/// Result of a WAL recovery operation.
///
/// Contains information about what was recovered and what actions
/// were taken during the recovery process.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalRecoveryResult {
    /// Number of entries that were recovered
    pub entries_recovered: usize,
    /// Number of transactions that were rolled back
    pub transactions_rolled_back: usize,
    /// Number of transactions that were committed
    pub transactions_committed: usize,
    /// Number of entries that were checkpointed
    pub entries_checkpointed: usize,
    /// Any errors encountered during recovery
    pub recovery_errors: Vec<String>,
}

/// Abstraction over a Write-Ahead Log for storage backends.
///
/// This trait provides durability guarantees by ensuring all operations
/// are logged before being applied to the main storage. In case of crashes,
/// the WAL can be replayed to restore the system to a consistent state.
#[async_trait]
pub trait WriteAheadLog: Send + Sync {
    /// Begin a new transaction and return its identifier.
    ///
    /// All subsequent operations within this transaction will be logged
    /// and can be atomically committed or rolled back.
    async fn begin_transaction(&self) -> anyhow::Result<TransactionId>;

    /// Write an entry to the WAL for the given transaction.
    ///
    /// The entry is logged but not yet committed. The operation will
    /// only take effect when the transaction is committed.
    async fn write_entry(
        &self,
        transaction_id: TransactionId,
        operation: WalOperation,
    ) -> anyhow::Result<()>;

    /// Commit a transaction, making all logged operations durable.
    ///
    /// All WAL entries for this transaction are applied to the main
    /// storage and marked as committed.
    async fn commit_transaction(&self, transaction_id: TransactionId) -> anyhow::Result<()>;

    /// Rollback a transaction, discarding all logged operations.
    ///
    /// All WAL entries for this transaction are marked as rolled back
    /// and will not be applied to the main storage.
    async fn rollback_transaction(&self, transaction_id: TransactionId) -> anyhow::Result<()>;

    /// Recover from a previous crash by replaying the WAL.
    ///
    /// This method examines all WAL entries and applies any committed
    /// but not yet applied operations. Uncommitted transactions are
    /// rolled back.
    async fn recover(&self) -> anyhow::Result<WalRecoveryResult>;

    /// Create a checkpoint up to the given sequence number.
    ///
    /// Entries up to this sequence number are considered durably stored
    /// and can be safely removed from the WAL to free space.
    async fn checkpoint(&self, sequence: SequenceNumber) -> anyhow::Result<()>;

    /// Get the current WAL sequence number.
    ///
    /// This is useful for determining checkpoint positions and
    /// monitoring WAL growth.
    async fn current_sequence(&self) -> anyhow::Result<SequenceNumber>;
}

//─────────────────────────────
//  Causal hashing utilities
//─────────────────────────────

/// Compute Blake3 causal hash for an event payload.
///
/// The hash includes both the payload bytes and all parent event digests,
/// providing a cryptographic guarantee of the event's position in the
/// causal chain. Parent digests are sorted to ensure deterministic hashing
/// regardless of input order.
pub fn causal_hash(payload_bytes: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(payload_bytes);

    // Sort parent digests to ensure deterministic hashing
    let mut sorted_parents = parent_digests.to_vec();
    sorted_parents.sort_unstable();

    for parent_digest in sorted_parents {
        hasher.update(&parent_digest);
    }

    hasher.finalize().into()
}

/// Utility to build an [`EventHeader`] from a payload and parent events.
///
/// This function handles serialization of the payload, computation of the
/// causal hash, and generation of a unique event ID. It ensures all events
/// have proper causal ordering and integrity verification.
pub fn create_event_header<P: EventPayload>(
    parents: &[EventHeader],
    intent: IntentId,
    kind: String,
    payload: &P,
) -> Result<EventHeader, rmp_serde::encode::Error> {
    let parent_ids: SmallVec<[EventId; 4]> = parents.iter().map(|h| h.id).collect();
    let parent_digests: Vec<CausalDigest> = parents.iter().map(|h| h.digest).collect();

    let payload_bytes = rmp_serde::to_vec_named(payload)?;
    let digest = causal_hash(&payload_bytes, &parent_digests);

    Ok(EventHeader {
        id: Uuid::new_v4(),
        parents: parent_ids,
        timestamp: Utc::now(),
        digest,
        intent,
        kind,
    })
}

/// Deserialize a payload from raw bytes.
///
/// This is a convenience function for deserializing payloads retrieved
/// from storage backends using the `payload_bytes` method.
pub fn deserialize_payload<P: EventPayload>(bytes: &[u8]) -> Result<P, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}

//─────────────────────────────
//  Storage backend traits
//─────────────────────────────

/// Abstraction over an append-only event sink.
///
/// Storage backends implement this trait to provide event persistence.
/// The trait is designed to be simple and efficient, requiring only
/// the ability to store event headers and their associated payload bytes.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Persist an [`EventHeader`] together with its serialized payload bytes.
    ///
    /// This operation should be atomic - either both the header and payload
    /// are stored successfully, or neither is stored. Implementations may
    /// batch writes for performance but must maintain event ordering.
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> anyhow::Result<()>;

    /// Fetch an [`EventHeader`] by identifier.
    ///
    /// Returns `None` if no event with the given ID exists. This operation
    /// should be fast as it only requires header retrieval.
    async fn header(&self, id: &EventId) -> anyhow::Result<Option<EventHeader>>;

    /// Get the raw payload bytes for a given digest.
    ///
    /// Returns `None` if no event with the given digest exists. The digest
    /// serves as both a lookup key and integrity verification. Callers must
    /// deserialize the bytes themselves using the appropriate type.
    async fn payload_bytes(&self, digest: &CausalDigest) -> anyhow::Result<Option<Vec<u8>>>;
}

/// Enhanced storage backend with Write-Ahead Logging support.
///
/// This trait extends the basic storage backend with WAL capabilities,
/// providing durability guarantees and crash recovery.
#[async_trait]
pub trait WalStorageBackend: StorageBackend + WriteAheadLog {
    /// Commit an event within a WAL transaction.
    ///
    /// This method combines event commitment with WAL logging to ensure
    /// durability. The operation is logged but not immediately applied to storage.
    /// The actual storage commitment happens when the transaction is committed.
    async fn commit_with_wal(
        &self,
        transaction_id: TransactionId,
        header: &EventHeader,
        payload: &[u8],
    ) -> anyhow::Result<()> {
        // Log the operation - it will be applied when transaction is committed
        self.write_entry(
            transaction_id,
            WalOperation::CommitEvent {
                header: header.clone(),
                payload: payload.to_vec(),
            },
        )
        .await
    }
}

// Automatic implementation for types that implement both traits
impl<T> WalStorageBackend for T where T: StorageBackend + WriteAheadLog {}

//─────────────────────────────
//  Error types
//─────────────────────────────

/// Errors that can occur during storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Event serialization failed
    #[error("failed to serialize event: {0}")]
    SerializationFailed(String),
    /// Event deserialization failed
    #[error("failed to deserialize event: {0}")]
    DeserializationFailed(String),
    /// Storage backend operation failed
    #[error("storage operation failed: {0}")]
    BackendError(String),
    /// Event not found
    #[error("event not found: {0}")]
    EventNotFound(String),
    /// Invalid causal hash
    #[error("invalid causal hash: expected {expected}, got {actual}")]
    InvalidCausalHash {
        /// Expected hash
        expected: String,
        /// Actual hash
        actual: String,
    },
    /// WAL operation failed
    #[error("WAL operation failed: {0}")]
    WalOperationFailed(String),
    /// Transaction not found
    #[error("transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    /// Transaction already committed
    #[error("transaction already committed: {0}")]
    TransactionAlreadyCommitted(TransactionId),
    /// Transaction already rolled back
    #[error("transaction already rolled back: {0}")]
    TransactionAlreadyRolledBack(TransactionId),
    /// Recovery failed
    #[error("WAL recovery failed: {0}")]
    RecoveryFailed(String),
}

//─────────────────────────────
//  Semantic analysis support
//─────────────────────────────

/// Semantic analysis plugin interface for event content analysis.
pub mod semantic;

//─────────────────────────────
//  Convenience re-exports
//─────────────────────────────

/// Convenient prelude for importing the most common types.
pub mod prelude {
    pub use super::{
        CausalDigest, EventHeader, EventId, EventPayload, IntentId,
        StorageBackend, StorageError,
        causal_hash, create_event_header, deserialize_payload,
        // WAL types
        TransactionId, SequenceNumber, WalEntry, WalOperation, WalEntryState,
        WalRecoveryResult, WriteAheadLog, WalStorageBackend,
        // Semantic analysis types
        semantic::{
            PluginId, SemanticResult, SemanticError, PluginMetadata, PluginConfig,
            ClassificationResult, EventRelationship, RelationshipGraph, AnomalyReport,
            ContentClassifier, RelationshipExtractor, AnomalyDetector,
            PluginRegistry, SemanticEngine, SemanticAnalysisResult,
            SemanticConfigBuilder, PluginType,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        message: String,
        value: i32,
    }

    #[test]
    fn test_causal_hash_deterministic() {
        let payload = b"test_payload";
        let parent1 = [1u8; 32];
        let parent2 = [2u8; 32];

        let hash1 = causal_hash(payload, &[parent1, parent2]);
        let hash2 = causal_hash(payload, &[parent2, parent1]); // Different order

        // Should be the same due to sorting
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_create_event_header() {
        let event = TestEvent {
            message: "test".to_string(),
            value: 42,
        };

        let header = create_event_header(
            &[],
            Uuid::nil(),
            "test.event".to_string(),
            &event,
        ).unwrap();

        assert_eq!(header.kind, "test.event");
        assert_eq!(header.parents.len(), 0);
        assert_eq!(header.intent, Uuid::nil());
    }

    #[test]
    fn test_causal_hash_with_parents() {
        let payload = b"child_event";
        let parent1 = [1u8; 32];
        let parent2 = [2u8; 32];

        let hash_with_parents = causal_hash(payload, &[parent1, parent2]);
        let hash_without_parents = causal_hash(payload, &[]);

        // Hashes should be different
        assert_ne!(hash_with_parents, hash_without_parents);
    }

    #[test]
    fn test_event_header_serialization() {
        let header = EventHeader {
            id: Uuid::new_v4(),
            parents: SmallVec::new(),
            timestamp: Utc::now(),
            digest: [0u8; 32],
            intent: Uuid::new_v4(),
            kind: "test.event".to_string(),
        };

        let serialized = serde_json::to_string(&header).unwrap();
        let deserialized: EventHeader = serde_json::from_str(&serialized).unwrap();

        assert_eq!(header, deserialized);
    }
}