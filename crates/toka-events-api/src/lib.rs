//! Event API crate – pure data types and trait contracts for the Toka event
//! subsystem.  This crate purposefully contains **no storage code** or heavy
//! dependencies so it can be used in any context, including `no_std` (once the
//! default features are disabled).
//!
//! Downstream crates are expected to implement the [`EventSink`] and
//! [`QueryApi`] traits and provide their own storage back-ends.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "serde-support")] 
extern crate alloc; // Required for no_std + serde (`Vec`)

#[cfg(feature = "serde-support")] 
use alloc::vec::Vec;

#[cfg(feature = "async")]
use alloc::boxed::Box;

// -------------------------------------------------------------------------------------------------
// Imports (gated by feature flags)
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "serde-support")] use serde::{Serialize, Deserialize};
#[cfg(feature = "serde-support")] use smallvec::SmallVec;
#[cfg(feature = "serde-support")] use chrono::{DateTime, Utc};
#[cfg(feature = "serde-support")] use uuid::Uuid;
#[cfg(feature = "digest")] use blake3;
#[cfg(feature = "async")] use async_trait::async_trait;
#[cfg(feature = "digest")] use rmp_serde;
#[cfg(feature = "serde-support")] use core::fmt::Debug;

// -------------------------------------------------------------------------------------------------
// Type aliases & core traits
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "serde-support")]
/// Unique identifier for a committed event (UUID v4).
pub type EventId = Uuid;

#[cfg(feature = "serde-support")]
/// Semantic identifier representing a high-level intent or task cluster.
pub type IntentId = Uuid;

#[cfg(feature = "serde-support")]
/// Blake3 digest representing the causal hash chain of an event.
pub type CausalDigest = [u8; 32];

/// Marker trait implemented by all serialisable event payloads.
#[cfg(feature = "serde-support")]
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

#[cfg(feature = "serde-support")]
impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Minimal header stored inline with every event.
#[cfg_attr(not(feature = "serde-support"), allow(dead_code))]
#[cfg(feature = "serde-support")]
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
    pub kind: alloc::string::String,
}

/// Compute Blake3 causal hash for an event payload.
#[cfg(feature = "digest")]
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
#[cfg(all(feature = "serde-support", feature = "digest"))]
pub fn create_event_header<P: EventPayload>(
    parents: &[EventHeader],
    intent: IntentId,
    kind: alloc::string::String,
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

// -------------------------------------------------------------------------------------------------
// Traits defining the storage contract
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "async")]
#[async_trait]
/// Abstraction over an append-only event sink.
pub trait EventSink: Send + Sync {
    /// Persist an [`EventHeader`] together with its serialized payload bytes.
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> anyhow::Result<()>;
}

#[cfg(feature = "async")]
#[async_trait]
/// Read-side query interface for event headers & payloads.
pub trait QueryApi: Send + Sync {
    /// Fetch an [`EventHeader`] by identifier.
    async fn header(&self, id: &EventId) -> anyhow::Result<Option<EventHeader>>;

    /// Materialise a payload value for a given digest.
    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> anyhow::Result<Option<P>>;
}

// -------------------------------------------------------------------------------------------------
// Prelude for convenience
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "serde-support")]
pub mod prelude {
    //! Convenient glob-import for the most common event types & traits.
    pub use super::{EventHeader, EventId, IntentId, CausalDigest, EventPayload};
    #[cfg(feature = "async")]
    pub use super::{EventSink, QueryApi};
}