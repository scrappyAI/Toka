#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-sqlite** – SQLite-based persistent storage driver for Toka OS.
//!
//! This crate provides a reliable, portable storage backend using SQLite
//! database engine via sqlx. It offers ACID transactions, excellent portability,
//! and efficient storage while maintaining the same interface as other storage drivers.

use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use tokio::sync::broadcast;

use toka_store_core::{
    StorageBackend, EventHeader, EventId, CausalDigest
};

/// Default broadcast channel size for live event streaming.
const DEFAULT_BROADCAST_SIZE: usize = 256;

//─────────────────────────────
//  SQLite storage backend
//─────────────────────────────

/// A persistent storage backend using SQLite database.
///
/// This implementation provides durable storage with ACID guarantees,
/// excellent portability, and efficient on-disk representation.
/// The database uses two tables: one for event headers and one for payloads,
/// with automatic deduplication of payloads by content hash.
#[derive(Debug)]
pub struct SqliteBackend {
    pool: SqlitePool,
    broadcast_tx: broadcast::Sender<EventHeader>,
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
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let pool = SqlitePool::connect(&database_url).await?;
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
        };

        backend.migrate().await?;
        Ok(backend)
    }

    /// Run database migrations to ensure schema is current.
    async fn migrate(&self) -> Result<()> {
        // Create headers table
        sqlx::query(
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
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_payloads (
                digest BLOB PRIMARY KEY,
                payload_data BLOB NOT NULL
            ) STRICT
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_headers_timestamp ON event_headers(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_headers_intent ON event_headers(intent)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_headers_kind ON event_headers(kind)")
            .execute(&self.pool)
            .await?;

        Ok(())
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
        let row = sqlx::query("SELECT COUNT(*) as count FROM event_headers")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    /// Get the total number of unique payloads stored.
    ///
    /// This may be less than the event count due to payload deduplication
    /// when multiple events share the same content hash.
    pub async fn payload_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM event_payloads")
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
        sqlx::query(
            "INSERT OR IGNORE INTO event_payloads (digest, payload_data) VALUES (?, ?)"
        )
        .bind(&header.digest[..])
        .bind(payload)
        .execute(&mut *tx)
        .await?;

        // Store header (may overwrite previous version with same ID)
        let header_bytes = rmp_serde::to_vec_named(header)?;
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO event_headers 
            (id, header_data, timestamp, intent, kind) 
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(header.id.as_bytes())
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
        let row = sqlx::query(
            "SELECT header_data FROM event_headers WHERE id = ?"
        )
        .bind(id.as_bytes())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let header_bytes: Vec<u8> = row.get("header_data");
            match rmp_serde::from_slice(&header_bytes) {
                Ok(header) => Ok(Some(header)),
                Err(e) => Err(anyhow::anyhow!("Failed to deserialize header: {}", e)),
            }
        } else {
            Ok(None)
        }
    }

    async fn payload_bytes(&self, digest: &CausalDigest) -> Result<Option<Vec<u8>>> {
        let row = sqlx::query(
            "SELECT payload_data FROM event_payloads WHERE digest = ?"
        )
        .bind(&digest[..])
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("payload_data")))
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
    }
}