//! The persistent vault implementation, backed by sled.

use crate::bus::EventBus;
use crate::events::{
    causal_hash, CausalDigest, EventHeader, EventId, EventPayload, IntentId,
};
use crate::strategy::{IntentStrategy, NilIntentStrategy};
use anyhow::Result;
use async_trait::async_trait;
use sled::Tree;
use tokio::sync::broadcast;

/// Default broadcast channel size for live event streaming.
const DEFAULT_BROADCAST_SIZE: usize = 256;

/// A persistent event vault with `sled` storage and optional intent clustering.
#[derive(Debug)]
pub struct PersistentVault<S: IntentStrategy = NilIntentStrategy> {
    db_payloads: Tree,
    db_headers: Tree,
    broadcast_tx: broadcast::Sender<EventHeader>,
    intent_strategy: S,
}

impl PersistentVault<NilIntentStrategy> {
    /// Creates a new persistent vault at the given path without intent clustering.
    pub fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers = db.open_tree("headers")?;
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BROADCAST_SIZE);

        Ok(Self {
            db_payloads,
            db_headers,
            broadcast_tx,
            intent_strategy: NilIntentStrategy,
        })
    }
}

impl<S: IntentStrategy> PersistentVault<S> {
    /// Creates a new persistent vault with a custom intent strategy.
    pub fn with_intent_strategy(path: &str, strategy: S) -> Result<Self> {
        let db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers = db.open_tree("headers")?;
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BROADCAST_SIZE);

        Ok(Self {
            db_payloads,
            db_headers,
            broadcast_tx,
            intent_strategy: strategy,
        })
    }

    /// Subscribes to the live event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.broadcast_tx.subscribe()
    }

    /// Gets the number of discovered intent clusters.
    pub async fn intent_cluster_count(&self) -> usize {
        self.intent_strategy.cluster_count().await
    }

    /// Gets the total number of events stored.
    pub fn event_count(&self) -> usize {
        self.db_headers.len()
    }

    /// Gets the total number of unique payloads stored.
    pub fn payload_count(&self) -> usize {
        self.db_payloads.len()
    }
}

#[async_trait]
impl<S: IntentStrategy> EventBus for PersistentVault<S> {
    async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind: &str,
        embedding: &[f32],
    ) -> Result<EventHeader> {
        // 1. Serialize payload
        let payload_bytes = rmp_serde::to_vec_named(payload)?;

        // 2. Compute causal hash
        let parent_digests: Vec<CausalDigest> = parents.iter().map(|h| h.digest).collect();
        let digest = causal_hash(&payload_bytes, &parent_digests);

        // 3. Deduplicate payload storage
        if self.db_payloads.get(&digest)?.is_none() {
            self.db_payloads.insert(&digest, payload_bytes)?;
        }

        // 4. Assign intent
        let (intent, _is_new_cluster) = self.intent_strategy.assign_intent(embedding).await?;

        // 5. Create event header
        let header = crate::events::create_event_header(parents, intent, kind.to_string(), payload)?;

        // 6. Persist header
        let header_bytes = rmp_serde::to_vec_named(&header)?;
        self.db_headers.insert(header.id.as_bytes(), header_bytes)?;

        // 7. Broadcast to live subscribers
        let _ = self.broadcast_tx.send(header.clone());

        Ok(header)
    }

    async fn get_header(&self, event_id: &EventId) -> Result<Option<EventHeader>> {
        if let Some(bytes) = self.db_headers.get(event_id.as_bytes())? {
            let header: EventHeader = rmp_serde::from_slice(&bytes)?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }

    async fn get_payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        if let Some(bytes) = self.db_payloads.get(digest)? {
            let payload: P = rmp_serde::from_slice(&bytes)?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
} 