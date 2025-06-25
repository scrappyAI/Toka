//! # Toka Events – Canonical Event Store (renamed from `toka-vault`)
//!
//! This crate provides the canonical, secure, and persistent event store for the
//! Toka platform. It is the single source of truth for "what happened" across
//! the entire system.
//!
//! ## Core Features
//!
//! *   **Unified API**: A single `Vault` entry point for different backends.
//! *   **Persistent Storage**: A `sled`-backed, content-addressed store for events.
//! *   **In-Memory Mode**: A non-persistent, in-memory bus for testing and lightweight scenarios.
//! *   **Causal Hashing**: Events are linked via a Blake3 causal hash chain, ensuring integrity.
//! *   **Intent Clustering**: (Optional) Automatically groups events by semantic intent using embeddings.
//! *   **Live Streaming**: Broadcasts committed event headers to live subscribers.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use toka_events::Vault;
//! use toka_events::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyEvent {
//!     id: u32,
//!     message: String,
//! }
//! impl EventPayload for MyEvent {}
//!
//! # async fn run() -> anyhow::Result<()> {
//! // Open a persistent store
//! let vault = Vault::open_persistent("./my-events-data")?;
//!
//! // Or use an in-memory vault
//! let memory_vault = Vault::new_memory();
//!
//! // Commit an event
//! let my_payload = MyEvent { id: 1, message: "Hello, Events!".to_string() };
//! let header = vault.commit(&my_payload, &[], "my.event.kind", &[]).await?;
//!
//! println!("Committed event with ID: {}", header.id);
//!
//! // Subscribe to live events
//! let mut subscriber = vault.subscribe();
//! while let Ok(header) = subscriber.recv().await {
//!     println!("Received live event: {:?}", header);
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// The new, consolidated modules
// Legacy bus trait is deprecated – retained for transitional compilation only.
#[deprecated(note = "EventBus has moved to the `toka-bus` crate – use that instead.")]
pub mod bus;
/// Persistence-only public traits (Slice 2).
pub mod api;
pub mod events;
pub mod memory;
pub mod persistence;
pub mod strategy;

/// A convenient prelude for importing the most common types.
pub mod prelude {
    // Persistence layer traits
    pub use crate::api::{EventSink, QueryApi};
    // Legacy export kept for downstreams that haven't migrated yet.
    #[allow(deprecated)]
    pub use crate::bus::EventBus;
    pub use crate::Vault;
    pub use crate::events::{
        causal_hash, create_event_header, CausalDigest, DomainEvent, EventHeader, EventId,
        EventPayload, IntentId,
    };
    #[cfg(feature = "memory-vault")]
    pub use crate::memory::MemoryVault;

    #[cfg(feature = "persist-sled")]
    pub use crate::persistence::PersistentVault;

    #[cfg(feature = "intent-cluster")]
    pub use crate::strategy::{IntentStrategy, NilIntentStrategy, OnlineClusterStrategy};
}

use anyhow::Result;
use prelude::*;
use tokio::sync::broadcast;
use async_trait::async_trait;
use crate::api::{EventSink, QueryApi};

/// The primary, unified entry point for interacting with the event store.
///
/// This enum abstracts over the different backend implementations (e.g., persistent vs. in-memory),
/// providing a consistent API surface for committing and subscribing to events.
#[derive(Debug)]
pub enum Vault {
    /// A persistent, `sled`-backed vault.
    #[cfg(feature = "persist-sled")]
    Persistent(persistence::PersistentVault),
    /// An in-memory, non-persistent vault.
    #[cfg(feature = "memory-vault")]
    Memory(memory::MemoryVault),
}

impl Vault {
    /// Opens a new or existing persistent store at the specified file path.
    ///
    /// This will create the necessary database files if they don't exist.
    ///
    /// # Features
    /// This method is only available when the `persist-sled` feature is enabled.
    #[cfg(feature = "persist-sled")]
    pub fn open_persistent(path: &str) -> Result<Self> {
        Ok(Self::Persistent(persistence::PersistentVault::open(path)?))
    }

    /// Creates a new, non-persistent, in-memory vault.
    ///
    /// Ideal for testing or scenarios where event persistence is not required.
    ///
    /// # Features
    /// This method is only available when the `memory-vault` feature is enabled.
    #[cfg(feature = "memory-vault")]
    pub fn new_memory() -> Self {
        Self::Memory(memory::MemoryVault::new())
    }

    /// Subscribes to the live stream of `EventHeader`s.
    ///
    /// Every event committed to the vault will be broadcast to all subscribers.
    /// The receiver will receive a clone of the `EventHeader` immediately after it has been persisted.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        match self {
            #[cfg(feature = "persist-sled")]
            Self::Persistent(v) => v.subscribe(),
            #[cfg(feature = "memory-vault")]
            Self::Memory(v) => v.subscribe(),
        }
    }
}

// Implement the new persistence-only traits ----------------------------------------------------

#[async_trait]
impl EventSink for Vault {
    async fn commit(&self, header: &EventHeader, payload: &[u8]) -> Result<()> {
        match self {
            #[cfg(feature = "persist-sled")]
            Self::Persistent(v) => v.commit(header, payload).await,
            #[cfg(feature = "memory-vault")]
            Self::Memory(v) => v.commit(header, payload).await,
        }
    }
}

#[async_trait]
impl QueryApi for Vault {
    async fn header(&self, id: &EventId) -> Result<Option<EventHeader>> {
        match self {
            #[cfg(feature = "persist-sled")]
            Self::Persistent(v) => v.header(id).await,
            #[cfg(feature = "memory-vault")]
            Self::Memory(v) => v.header(id).await,
        }
    }

    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        match self {
            #[cfg(feature = "persist-sled")]
            Self::Persistent(v) => v.payload(digest).await,
            #[cfg(feature = "memory-vault")]
            Self::Memory(v) => v.payload(digest).await,
        }
    }
} 