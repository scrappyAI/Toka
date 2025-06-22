//! Event bus with persistent storage and live streaming (core version).
//!
//! This is a trimmed-down copy of the original `toka-ledger` implementation that
//! **omits intent clustering**.  Down-stream crates (e.g. `toka-ledger-agents`)
//! can wrap or extend `VaultBus` to add semantic features.

use anyhow::Result;
use sled::Db;
use tokio::sync::broadcast;

use crate::core::{EventHeader, EventPayload};
use crate::hash::causal_hash;

/// Vault event bus with local sled-backed storage.
///
/// Features provided by the *core* implementation:
/// • Persistent, content-addressed storage (payload + header trees)
/// • Causal hashing for deduplication & integrity
/// • Live event streaming via Tokio broadcast
#[derive(Debug)]
pub struct VaultBus {
    db_payloads: sled::Tree,      // digest → payload bytes
    db_headers:  sled::Tree,      // id     → header bytes
    tx_notify:   broadcast::Sender<EventHeader>,
}

impl VaultBus {
    /// Open (or create) a vault database at the specified path.
    /// Two sled trees are created:
    ///  • `payloads` – content-addressed payload bytes
    ///  • `headers`  – event headers keyed by UUID
    pub fn open(path: &str) -> Result<Self> {
        let db: Db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers  = db.open_tree("headers")?;
        let (tx_notify, _) = broadcast::channel(256);
        Ok(Self { db_payloads, db_headers, tx_notify })
    }

    /// Commit an event payload to the vault.
    ///
    /// Workflow:
    /// 1. Serialize payload via MessagePack
    /// 2. Compute causal digest
    /// 3. Deduplicate payload storage
    /// 4. Build & persist header, broadcast to subscribers
    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind:    &str,
        _embedding: ndarray::Array1<f32>, // accepted for API parity; ignored here
    ) -> Result<EventHeader> {
        // 1. Serialize payload
        let bytes = rmp_serde::to_vec_named(payload)?;

        // 2. Parent digests for causal hash
        let parent_digests: Vec<_> = parents.iter().map(|h| h.digest).collect();

        // 3. Compute digest
        let digest = causal_hash(&bytes, &parent_digests);

        // 4. Dedup payload storage
        if self.db_payloads.get(&digest)?.is_none() {
            self.db_payloads.insert(&digest, bytes.clone())?;
        }

        // 5. Assemble header – intent left as `Uuid::nil()` (unknown)
        let hdr = EventHeader {
            id: uuid::Uuid::new_v4(),
            parents: parents.iter().map(|h| h.id).collect(),
            timestamp: chrono::Utc::now(),
            digest,
            intent: uuid::Uuid::nil(),
            kind: kind.into(),
        };

        // 6. Persist header & broadcast
        self.db_headers.insert(hdr.id.as_bytes(), rmp_serde::to_vec_named(&hdr)?)?;
        let _ = self.tx_notify.send(hdr.clone());
        Ok(hdr)
    }

    /// Subscribe to the live event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.tx_notify.subscribe()
    }
} 