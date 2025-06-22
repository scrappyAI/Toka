//! Event bus with persistent storage and live streaming.

use anyhow::Result;
use sled::Db;
use tokio::sync::broadcast;

use crate::core::{EventHeader, EventPayload};
use crate::hash::causal_hash;
use crate::intent::IntentStore;

/// Vault event bus with local RocksDB-backed storage.
///
/// The bus provides:
/// - Persistent, content-addressed event storage
/// - Causal hashing for deduplication and integrity
/// - Intent clustering for semantic organization
/// - Live event streaming via broadcast channels
#[derive(Debug)]
pub struct VaultBus {
    db_payloads: sled::Tree,      // digest → payload bytes
    db_headers:  sled::Tree,      // id     → header bytes
    tx_notify:   broadcast::Sender<EventHeader>,
    intents:     IntentStore,
}

impl VaultBus {
    /// Open (or create) a vault database at the specified path.
    ///
    /// This creates two RocksDB instances:
    /// - `{path}/payloads`: stores event payloads by their causal digest
    /// - `{path}/headers`: stores event headers by their ID
    pub fn open(path: &str) -> Result<Self> {
        let db: Db = sled::open(path)?;
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

    /// Commit an event payload to the vault.
    ///
    /// This operation:
    /// 1. Serializes the payload using MessagePack
    /// 2. Computes a causal hash from payload + parent digests
    /// 3. Deduplicates payload storage by digest
    /// 4. Assigns the event to an intent cluster based on the embedding
    /// 5. Persists the event header and broadcasts it to subscribers
    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind:    &str,
        embedding: ndarray::Array1<f32>,
    ) -> Result<EventHeader> {
        // 1. Serialize payload (MessagePack for compactness)
        let bytes = rmp_serde::to_vec_named(payload)?;

        // 2. Parent digests for causal hash
        let parent_digests: Vec<_> = parents.iter().map(|h| h.digest).collect();

        // 3. Compute digest
        let digest = causal_hash(&bytes, &parent_digests);

        // 4. Dedup: store payload only once per digest
        if self.db_payloads.get(&digest)?.is_none() {
            self.db_payloads.insert(&digest, bytes.clone())?;
        }

        // 5. Intent clustering
        let (intent, _is_new) = self.intents.assign(&embedding);

        // 6. Assemble header
        let hdr = EventHeader {
            id: uuid::Uuid::new_v4(),
            parents: parents.iter().map(|h| h.id).collect(),
            timestamp: chrono::Utc::now(),
            digest,
            intent,
            kind: kind.into(),
        };

        // 7. Persist header and broadcast
        self.db_headers.insert(hdr.id.as_bytes(), rmp_serde::to_vec_named(&hdr)?)?;
        let _ = self.tx_notify.send(hdr.clone());
        Ok(hdr)
    }

    /// Subscribe to live event stream.
    ///
    /// Returns a broadcast receiver that will receive all newly committed events.
    /// Note that if the receiver falls behind, it may miss events.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.tx_notify.subscribe()
    }

    /// Get the number of discovered intent clusters.
    pub fn intent_cluster_count(&self) -> usize {
        self.intents.cluster_count()
    }
} 