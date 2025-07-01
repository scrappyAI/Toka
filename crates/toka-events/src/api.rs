//! Internal API module – formerly its own `toka-events-api` crate.
//!
//! Contains the pure data types and trait contracts for the Toka event subsystem.
//!
//! The contents were migrated in July 2025 as part of the "crate consolidation" effort
//! (see `docs/code-clarity-report.md`). The external crate `toka_events_api` has been
//! deprecated – downstream code can now simply `use toka_events::api::*`.
//!
//! Note: we intentionally keep the API free of heavy dependencies so it can, in the
//! future, be compiled in `no_std` contexts again. For the time being the main crate
//! (`toka-events`) requires the Rust standard library, but the API sub-module refrains
//! from pulling additional crates unless absolutely necessary.

#![allow(clippy::module_inception)]

use std::vec::Vec;
use core::fmt::Debug;

use async_trait::async_trait;
use blake3;
use chrono::{DateTime, Utc};
use rmp_serde;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

/// Unique identifier for a committed event (UUID v4).
pub type EventId = Uuid;

/// Semantic identifier representing a high-level intent or task cluster.
pub type IntentId = Uuid;

/// Blake3 digest representing the causal hash chain of an event.
pub type CausalDigest = [u8; 32];

/// Marker trait implemented by all serialisable event payloads.
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Minimal header stored inline with every event.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventHeader {
    /// Event identifier (UUID v4).
    pub id: EventId,
    /// Parent event IDs this event causally depends on (can be empty).
    pub parents: SmallVec<[EventId; 4]>,
    /// Wall-clock timestamp when the event was committed.
    pub timestamp: DateTime<Utc>,
    /// Blake3 digest of the event payload and its causal parent digests.
    pub digest: CausalDigest,
    /// Semantic intent bucket this event belongs to.
    pub intent: IntentId,
    /// Application-defined kind, e.g. `ledger.mint` or `chat.msg`.
    pub kind: String,
}

/// Compute Blake3 causal hash for an event payload.
pub fn causal_hash(payload_bytes: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(payload_bytes);

    let mut sorted_parents = parent_digests.to_vec();
    sorted_parents.sort_unstable();

    for parent_digest in sorted_parents {
        hasher.update(&parent_digest);
    }

    hasher.finalize().into()
}

/// Utility to build an [`EventHeader`].
pub fn create_event_header<P: EventPayload>(
    parents: &[EventHeader],
    intent: IntentId,
    kind: String,
    payload: &P,
) -> Result<EventHeader, rmp_serde::encode::Error> {
    let parent_ids: SmallVec<[EventId; 4]> = parents.iter().map(|h| h.id).collect();
    let parent_digests: Vec<CausalDigest> = parents.iter().map(|h| h.digest).collect();

    let payload_bytes = rmp_serde::to_vec_named(payload)?;
    let digest = causal_hash(&payload_bytes, &parent_digests);

    Ok(EventHeader {
        id: Uuid::new_v4(),
        parents: parent_ids,
        timestamp: Utc::now(),
        digest,
        intent,
        kind,
    })
}

/// Abstraction over an append-only event sink.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Persist an [`EventHeader`] together with its serialized payload bytes.
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> anyhow::Result<()>;
}

/// Read-side query interface for event headers & payloads.
#[async_trait]
pub trait QueryApi: Send + Sync {
    /// Fetch an [`EventHeader`] by identifier.
    async fn header(&self, id: &EventId) -> anyhow::Result<Option<EventHeader>>;

    /// Materialise a payload value for a given digest.
    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> anyhow::Result<Option<P>>;
}

/// Convenient glob-import for the most common event types & traits.
pub mod prelude {
    pub use super::{CausalDigest, EventHeader, EventId, EventPayload, IntentId};
    pub use super::{EventSink, QueryApi};
}