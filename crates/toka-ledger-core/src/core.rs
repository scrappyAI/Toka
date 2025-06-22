//! Core event types and primitives for the Toka Ledger Core.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

/// Unique identifier for an event in the vault.
pub type EventId = Uuid;
/// Identifier for an intent cluster.
pub type IntentId = Uuid;
/// 32-byte Blake3 digest used for causal hashing.
pub type CausalDigest = [u8; 32];

/// Trait implemented by all event payload structures that can be committed
/// to the vault.
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Minimal header stored inline with every event.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// For the *core* crate we don't try to cluster; callers can set it to
    /// whatever value they need (e.g. `Uuid::nil()` when unknown).
    pub intent: IntentId,
    /// Application-defined kind, e.g. `ledger.mint` or `chat.msg`.
    pub kind: String,
} 