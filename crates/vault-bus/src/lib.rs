//! Vault Bus – append-only event log with causal hashing, intent clustering and
//! broadcast capabilities.
//!
//! The current implementation stores payloads and headers in two RocksDB column
//! families on local disk.  A simple `tokio::sync::broadcast` channel provides
//! live subscription support.  This is **experimental** and will evolve.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use rocksdb::{Options, DB};
use tokio::sync::broadcast;

use vault_core::{EventHeader, EventPayload};
use vault_hash::causal_hash;
use vault_intent::IntentStore;

/// Re-export common vault types for convenience.
pub use vault_core::{CausalDigest, EventId, IntentId};

/// Vault event bus with local RocksDB-backed storage.
#[derive(Debug)]
pub struct VaultBus {
    db_payloads: DB,              // digest → payload bytes
    db_headers:  DB,              // id     → header bytes
    tx_notify:   broadcast::Sender<EventHeader>,
    intents:     IntentStore,
}

impl VaultBus {
    /// Open (or create) a vault database at `path`.
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db_payloads = DB::open(&opts, format!("{path}/payloads"))?;
        let db_headers  = DB::open(&opts, format!("{path}/headers"))?;
        let (tx_notify, _) = broadcast::channel(256);
        Ok(Self {
            db_payloads,
            db_headers,
            tx_notify,
            intents: IntentStore::new(),
        })
    }

    /// Commit an event payload to the vault.
    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind:    &str,
        embedding: ndarray::Array1<f32>,
    ) -> Result<EventHeader> {
        // 1. Serialize payload (CBOR via rmp-serde for compactness)
        let bytes = rmp_serde::to_vec_named(payload)?;

        // 2. Parent digests for causal hash
        let parent_digests: Vec<_> = parents.iter().map(|h| h.digest).collect();

        // 3. Compute digest
        let digest = causal_hash(&bytes, &parent_digests);

        // 4. Dedup: store payload only once per digest
        if self.db_payloads.get(digest)?.is_none() {
            self.db_payloads.put(digest, &bytes)?;
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
        self.db_headers.put(hdr.id, rmp_serde::to_vec_named(&hdr)?)?;
        let _ = self.tx_notify.send(hdr.clone());
        Ok(hdr)
    }

    /// Subscribe to live event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.tx_notify.subscribe()
    }
} 