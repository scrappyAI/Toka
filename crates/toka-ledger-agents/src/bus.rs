//! Event bus with **intent clustering** built on top of `toka_ledger_core`.
//!
//! The implementation mirrors the original monolithic `toka-ledger::VaultBus`
//! so existing code can switch to [`AgentBus`] with minimal edits (ideally just
//! a crate rename).

use anyhow::Result;
use tokio::sync::broadcast;

use toka_ledger_core::core::{EventHeader, EventPayload};
use toka_ledger_core::hash::causal_hash;

use crate::intent::IntentStore;

use uuid::Uuid;
use chrono::Utc;

/// Agent-aware vault bus with online intent clustering.
#[derive(Debug)]
pub struct AgentBus {
    db_payloads: sled::Tree,
    db_headers:  sled::Tree,
    tx_notify:   broadcast::Sender<EventHeader>,
    intents:     IntentStore,
}

impl AgentBus {
    /// Open (or create) an agent vault at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let db: sled::Db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers  = db.open_tree("headers")?;
        let (tx_notify, _) = broadcast::channel(256);
        Ok(Self {
            db_payloads,
            db_headers,
            tx_notify,
            intents: IntentStore::new(),
        })
    }

    /// Commit an event payload and assign it to an intent cluster.
    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind:    &str,
        embedding: ndarray::Array1<f32>,
    ) -> Result<EventHeader> {
        // 1. Serialize payload
        let bytes = rmp_serde::to_vec_named(payload)?;

        // 2. Parent digests for causal hash
        let parent_digests: Vec<_> = parents.iter().map(|h| h.digest).collect();
        let digest = causal_hash(&bytes, &parent_digests);

        // 3. Dedup payload storage
        if self.db_payloads.get(&digest)?.is_none() {
            self.db_payloads.insert(&digest, bytes.clone())?;
        }

        // 4. Intent clustering
        let (intent, _is_new) = self.intents.assign(&embedding);

        // 5. Build header
        let hdr = EventHeader {
            id: uuid::Uuid::new_v4(),
            parents: parents.iter().map(|h| h.id).collect(),
            timestamp: chrono::Utc::now(),
            digest,
            intent,
            kind: kind.into(),
        };

        // 6. Persist header & broadcast
        self.db_headers.insert(hdr.id.as_bytes(), rmp_serde::to_vec_named(&hdr)?)?;
        let _ = self.tx_notify.send(hdr.clone());
        Ok(hdr)
    }

    /// Subscribe to live events.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.tx_notify.subscribe()
    }

    /// Number of discovered intent clusters.
    pub fn intent_cluster_count(&self) -> usize {
        self.intents.cluster_count()
    }
} 