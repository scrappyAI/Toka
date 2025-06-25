//!
//! Vault public traits – persistence-only API (crate-local rename maintained).
//!
#![allow(async_fn_in_trait)]

use crate::events::{CausalDigest, EventHeader, EventId, EventPayload};
use anyhow::Result;
use async_trait::async_trait;

/// Write-only persistence interface used by upstream components.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Persist an [`EventHeader`] together with its serialized payload bytes.
    ///
    /// Implementations **must** be idempotent – committing the same header &
    /// payload more than once must not corrupt the store.
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()>;
}

/// Read-only query interface for historical look-ups.
#[async_trait]
pub trait QueryApi: Send + Sync {
    /// Fetch a previously persisted header by identifier.
    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>>;

    /// Materialise an owned payload value of type `P` for a given digest.
    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>>;
} 