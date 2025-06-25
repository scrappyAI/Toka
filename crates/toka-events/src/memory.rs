//! In-memory, non-persistent event store implementation.

use crate::api::{EventSink, QueryApi};
use crate::events::{CausalDigest, EventHeader, EventId, EventPayload};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use anyhow::Result;

/// Default buffer size for the broadcast channel.
const DEFAULT_BUFFER: usize = 1024;

/// An in-memory, non-persistent event store.
///
/// This implementation is useful for testing, local development, or any scenario
/// where event persistence is not required.
#[derive(Debug, Clone)]
pub struct MemoryVault {
    headers: Arc<RwLock<HashMap<EventId, EventHeader>>>,
    payloads: Arc<RwLock<HashMap<CausalDigest, Vec<u8>>>>,
    broadcast_tx: broadcast::Sender<EventHeader>,
}

impl Default for MemoryVault {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryVault {
    /// Creates a new, empty `MemoryVault`.
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BUFFER);
        Self {
            headers: Arc::new(RwLock::new(HashMap::new())),
            payloads: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Subscribes to the live event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.broadcast_tx.subscribe()
    }
}

#[async_trait]
impl EventSink for MemoryVault {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        // Deduplicate payloads – multiple headers can reference same digest
        self.payloads
            .write()
            .await
            .entry(header.digest)
            .or_insert_with(|| payload.to_vec());

        // Persist header
        self.headers
            .write()
            .await
            .insert(header.id, header.clone());

        // Broadcast live update
        let _ = self.broadcast_tx.send(header.clone());

        Ok(())
    }
}

#[async_trait]
impl QueryApi for MemoryVault {
    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        Ok(self.headers.read().await.get(id).cloned())
    }

    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        if let Some(bytes) = self.payloads.read().await.get(digest) {
            let payload: P = rmp_serde::from_slice(bytes)?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
} 