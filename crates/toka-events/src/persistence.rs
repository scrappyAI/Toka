//! The persistent event-store implementation, backed by sled.

use crate::api::{EventSink, QueryApi};
use crate::events::{CausalDigest, EventHeader, EventId, EventPayload};
use crate::strategy::{IntentStrategy, NilIntentStrategy};
use anyhow::Result;
use async_trait::async_trait;
use sled::Tree;
use tokio::sync::broadcast;

/// Default broadcast channel size for live event streaming.
const DEFAULT_BROADCAST_SIZE: usize = 256;

/// A persistent event store with `sled` storage and optional intent clustering.
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
impl<S: IntentStrategy> EventSink for PersistentVault<S> {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        // 1. Deduplicate payload storage by digest
        if self.db_payloads.get(&header.digest)?.is_none() {
            self.db_payloads.insert(&header.digest, payload)?;
        }

        // 2. Persist header (overwrite-safe)
        let header_bytes = rmp_serde::to_vec_named(header)?;
        self.db_headers.insert(header.id.as_bytes(), header_bytes)?;

        // 3. Broadcast live update
        let _ = self.broadcast_tx.send(header.clone());

        Ok(())
    }
}

#[async_trait]
impl<S: IntentStrategy> QueryApi for PersistentVault<S> {
    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        if let Some(bytes) = self.db_headers.get(id.as_bytes())? {
            let header: EventHeader = rmp_serde::from_slice(&bytes)?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }

    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        if let Some(bytes) = self.db_payloads.get(digest)? {
            let payload: P = rmp_serde::from_slice(&bytes)?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
} 