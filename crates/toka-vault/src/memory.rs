//! In-memory, non-persistent event bus implementation.

use crate::bus::EventBus;
use crate::events::{
    CausalDigest, EventHeader, EventId, EventPayload,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Default buffer size for the broadcast channel.
const DEFAULT_BUFFER: usize = 1024;

/// An in-memory, non-persistent event bus.
///
/// This implementation is useful for testing, local development, or any scenario
/// where event persistence is not required. It provides the same `EventBus`
/// interface but stores all data in memory, which is lost when the process exits.
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
impl EventBus for MemoryVault {
    /// Commits an event to the in-memory store.
    ///
    /// Note: The `embedding` argument is ignored in this implementation as there
    /// is no persistence or intent clustering.
    async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind: &str,
        _embedding: &[f32], // Ignored
    ) -> Result<EventHeader> {
        let payload_bytes = rmp_serde::to_vec_named(payload)?;
        let header =
            crate::events::create_event_header(parents, uuid::Uuid::nil(), kind.to_string(), payload)?;

        self.payloads
            .write()
            .await
            .insert(header.digest, payload_bytes);
        self.headers
            .write()
            .await
            .insert(header.id, header.clone());

        let _ = self.broadcast_tx.send(header.clone());

        Ok(header)
    }

    async fn get_header(&self, event_id: &EventId) -> Result<Option<EventHeader>> {
        Ok(self.headers.read().await.get(event_id).cloned())
    }

    async fn get_payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        if let Some(bytes) = self.payloads.read().await.get(digest) {
            let payload: P = rmp_serde::from_slice(bytes)?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
} 