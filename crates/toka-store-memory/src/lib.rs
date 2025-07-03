#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-memory** – In-memory storage driver for Toka OS.
//!
//! This crate provides a fast, non-persistent storage backend suitable for
//! testing, development, and scenarios where event persistence is not required.
//! All data is stored in memory and will be lost when the process terminates.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};

use toka_store_core::{
    StorageBackend, EventHeader, EventId, CausalDigest
};

/// Default buffer size for the live event broadcast channel.
const DEFAULT_BUFFER: usize = 1024;

//─────────────────────────────
//  In-memory storage backend
//─────────────────────────────

/// An in-memory, non-persistent event store.
///
/// This implementation stores all events in memory using HashMap collections.
/// It provides excellent performance for read and write operations but offers
/// no persistence guarantees. All data is lost when the process terminates.
///
/// The storage backend also provides a live event stream via broadcast channels,
/// allowing subscribers to receive real-time notifications of committed events.
#[derive(Debug, Clone)]
pub struct MemoryBackend {
    headers: Arc<RwLock<HashMap<EventId, EventHeader>>>,
    payloads: Arc<RwLock<HashMap<CausalDigest, Vec<u8>>>>,
    broadcast_tx: broadcast::Sender<EventHeader>,
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

    /// Clear all stored events and payloads.
    ///
    /// This operation is useful for testing and development scenarios
    /// where you need to reset the storage state.
    pub async fn clear(&self) {
        self.headers.write().await.clear();
        self.payloads.write().await.clear();
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

        // Verify event is gone
        let result = backend.header(&header.id).await.unwrap();
        assert!(result.is_none());
    }
}