//! The core, abstract `EventBus` trait.

use crate::events::{CausalDigest, EventHeader, EventId, EventPayload};
use anyhow::Result;
use async_trait::async_trait;

/// Defines the universal interface for an event bus.
///
/// Both persistent and in-memory vault implementations adhere to this trait,
/// allowing for a consistent API and interchangeable backends.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Commits an event payload to the bus.
    ///
    /// This is the primary method for writing to the event store.
    ///
    /// # Arguments
    /// * `payload` - The event data to be stored. Must implement `EventPayload`.
    /// * `parents` - A slice of `EventHeader`s that this event causally depends on.
    /// * `kind` - An application-defined string describing the event type (e.g., "chat.message").
    /// * `embedding` - An optional float slice representing the semantic embedding of the event,
    ///   used for intent clustering. If empty or the feature is disabled, it's ignored.
    ///
    /// # Returns
    /// A `Result` containing the `EventHeader` of the newly committed event.
    async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind: &str,
        embedding: &[f32],
    ) -> Result<EventHeader>;

    /// Retrieves an event header by its unique `EventId`.
    async fn get_header(&self, event_id: &EventId) -> Result<Option<EventHeader>>;

    /// Retrieves a deserialized event payload by its `CausalDigest`.
    ///
    /// This is the content-addressed part of the store. Because payloads are
    /// deduplicated, multiple headers can point to the same payload digest.
    async fn get_payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>>;
} 