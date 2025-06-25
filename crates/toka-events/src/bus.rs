//! The core, abstract `EventBus` trait.
//!
//! **Deprecated:** This trait originally lived in the `toka-vault` crate but has
//! since been moved to the dedicated `toka-bus` crate.  It is kept here only to
//! avoid breaking downstream code during the crate rename to `toka-events`.
#![allow(deprecated)]

use crate::events::{CausalDigest, EventHeader, EventId, EventPayload};
use anyhow::Result;
use async_trait::async_trait;

/// Defines the universal interface for an event bus.
///
/// Both persistent and in-memory event-store implementations adhere to this
/// trait, allowing for a consistent API and interchangeable back-ends.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Commit an event payload to the bus and return its newly created header.
    async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind: &str,
        embedding: &[f32],
    ) -> Result<EventHeader>;

    /// Retrieve an event header by its unique `EventId`.
    async fn get_header(&self, event_id: &EventId) -> Result<Option<EventHeader>>;

    /// Retrieve and deserialize an event payload by its [`CausalDigest`].
    async fn get_payload<P: EventPayload>(
        &self,
        digest: &CausalDigest,
    ) -> Result<Option<P>>;
} 