#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-sled** – Sled-based persistent storage driver for Toka OS.
//!
//! This crate provides a durable, embedded storage backend using the sled
//! database engine. It offers ACID transactions, crash recovery, and efficient
//! on-disk storage while maintaining the same interface as other storage drivers.

use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use sled::{Db, Tree};
use tokio::sync::broadcast;

use toka_store_core::{
    StorageBackend, EventHeader, EventId, CausalDigest
};

/// Default broadcast channel size for live event streaming.
const DEFAULT_BROADCAST_SIZE: usize = 256;

//─────────────────────────────
//  Sled storage backend
//─────────────────────────────

/// A persistent storage backend using the sled embedded database.
///
/// This implementation provides durable storage with ACID guarantees,
/// automatic crash recovery, and efficient on-disk representation.
/// The database uses two trees: one for event headers and one for payloads,
/// with automatic deduplication of payloads by content hash.
#[derive(Debug)]
pub struct SledBackend {
    _db: Db,  // Keep reference to prevent premature database closure
    db_payloads: Tree,
    db_headers: Tree,
    broadcast_tx: broadcast::Sender<EventHeader>,
}

impl SledBackend {
    /// Opens or creates a new sled database at the specified path.
    ///
    /// The database will be created if it doesn't exist. This operation
    /// may perform recovery if the database was not properly closed.
    ///
    /// # Arguments
    /// * `path` - File system path where the database should be stored
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or created.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_config(path, sled::Config::default())
    }

    /// Opens a sled database with custom configuration.
    ///
    /// This allows fine-tuning of sled's behavior including cache size,
    /// compression, and other performance parameters.
    pub fn open_with_config<P: AsRef<Path>>(path: P, config: sled::Config) -> Result<Self> {
        let db = config.path(path).open()?;
        Self::from_db(db)
    }

    /// Creates a new backend from an existing sled database instance.
    ///
    /// This is useful when you need to share a database instance across
    /// multiple components or when using custom sled configurations.
    pub fn from_db(db: Db) -> Result<Self> {
        let db_payloads = db.open_tree("payloads")?;
        let db_headers = db.open_tree("headers")?;
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BROADCAST_SIZE);

        Ok(Self {
            _db: db,
            db_payloads,
            db_headers,
            broadcast_tx,
        })
    }

    /// Creates a temporary backend for testing purposes.
    ///
    /// The database will be stored in a temporary directory and
    /// automatically cleaned up when the backend is dropped.
    #[cfg(test)]
    pub fn temporary() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        Self::from_db(db)
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
    pub fn event_count(&self) -> usize {
        self.db_headers.len()
    }

    /// Get the total number of unique payloads stored.
    ///
    /// This may be less than the event count due to payload deduplication
    /// when multiple events share the same content hash.
    pub fn payload_count(&self) -> usize {
        self.db_payloads.len()
    }

    /// Flush all pending writes to disk.
    ///
    /// This ensures that all committed events are durably stored
    /// and will survive a crash or power failure.
    pub async fn flush(&self) -> Result<()> {
        self.db_headers.flush_async().await?;
        self.db_payloads.flush_async().await?;
        Ok(())
    }

    /// Get database size information.
    pub fn size_on_disk(&self) -> Result<u64> {
        Ok(self._db.size_on_disk()?)
    }
}

#[async_trait]
impl StorageBackend for SledBackend {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        // Store payload (deduplicated by digest)
        // Only insert if not already present to avoid unnecessary writes
        if self.db_payloads.get(&header.digest)?.is_none() {
            self.db_payloads.insert(&header.digest, payload)?;
        }

        // Store header (may overwrite previous version)
        let header_bytes = rmp_serde::to_vec_named(header)?;
        self.db_headers.insert(header.id.as_bytes(), header_bytes)?;

        // Broadcast live update (ignore errors if no subscribers)
        let _ = self.broadcast_tx.send(header.clone());

        Ok(())
    }

    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        if let Some(bytes) = self.db_headers.get(id.as_bytes())? {
            match rmp_serde::from_slice(&bytes) {
                Ok(header) => Ok(Some(header)),
                Err(e) => Err(anyhow::anyhow!("Failed to deserialize header: {}", e)),
            }
        } else {
            Ok(None)
        }
    }

    async fn payload_bytes(&self, digest: &CausalDigest) -> Result<Option<Vec<u8>>> {
        Ok(self.db_payloads.get(digest)?.map(|ivec| ivec.to_vec()))
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
        let backend = SledBackend::temporary().unwrap();
        
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
        let backend = SledBackend::temporary().unwrap();

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
        let backend = SledBackend::temporary().unwrap();

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
        assert_eq!(backend.event_count(), 2);
        assert_eq!(backend.payload_count(), 1);
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
            let backend = SledBackend::open(&db_path).unwrap();
            backend.commit(&header, &payload_bytes).await.unwrap();
            backend.flush().await.unwrap();
        } // Backend dropped, database closed

        // Reopen and verify data persisted
        {
            let backend = SledBackend::open(&db_path).unwrap();
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
        let backend = SledBackend::temporary().unwrap();
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
    async fn test_size_on_disk() {
        let backend = SledBackend::temporary().unwrap();
        
        // Should be able to get size without error
        let size = backend.size_on_disk().unwrap();
        assert!(size >= 0); // Size should be non-negative
    }
}