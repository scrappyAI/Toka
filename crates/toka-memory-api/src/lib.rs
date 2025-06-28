//! Toka Memory – API crate
//!
//! This crate defines the minimal **contract** for the memory subsystem used
//! throughout the Toka workspace.  The goal is to decouple *what* memory means
//! for autonomous agents from *how* it is stored.  The trait purposefully lives
//! in its own `no_std`-friendly crate so lean targets (WASM, embedded) or
//! alternative back-ends (Redis, sled, Postgres, RocksDB, vector stores, …) can
//! depend solely on this interface without dragging in the heavyweight
//! [`tokio`] runtime or a concrete implementation.
//!
//! ## Beyond a "key-value cache"
//! Modern agents employ a **hierarchy of memories** very similar to the memory
//! hierarchy in conventional computer architecture.  At a minimum the runtime
//! distinguishes:
//!
//! | Layer | What it stores | Typical lifetime |
//! |-------|----------------|------------------|
//! | *Working / scratchpad* | The last *N* tokens (context window) the model is currently attending to. | milliseconds–seconds |
//! | *Ephemeral agent state* | Transient variables such as tool outputs and chain-of-thought artifacts. | one turn / task |
//! | *Short-term (episodic)* | Summaries of recent turns or events, often with recency weighting. | minutes–hours |
//! | *Long-term semantic* | Facts, preferences, project artefacts retrieved via embeddings or keys. | days–indefinite |
//! | *Procedural / skill* | Fine-tuned weights or LoRA adapters. | until re-training |
//!
//! The [`MemoryAdapter`] abstraction intentionally stays **agnostic** to those
//! layers – it merely offers a uniform *byte-oriented* interface for storing
//! and retrieving values.  The higher-level orchestration code decides *what*
//! to write and *when* to read, implement forgetting/summary policies, etc.
//!
//! ## Versioning policy
//! The API is pre-`1.0`.  Adding a required method is therefore released as a
//! *minor bump* (`0.3.x → 0.4.x`).  Consumers in the same workspace use a path
//! dependency so the change is coordinated atomically via the workspace lock
//! file.
//!
//! ## Feature flags
//! * `async` *(default)* — Enable the asynchronous variant of the trait using
//!   [`async-trait`].
//!
//! ## Usage
//! ```rust,ignore
//! use toka_memory_api::MemoryAdapter;
//! # #[cfg(feature = "async")]
//! # async fn demo(adapter: &dyn MemoryAdapter) -> anyhow::Result<()> {
//! adapter.put("session_123", b"{"state":42}".to_vec(), 300).await?;
//! let bytes = adapter.get("session_123").await?.unwrap();
//! adapter.delete("session_123").await?;
//! # Ok(()) }
//! ```
//!
//! [`tokio`]: https://crates.io/crates/tokio
//! [`async-trait`]: https://crates.io/crates/async-trait

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;
#[allow(unused_imports)]
use alloc::boxed::Box;

#[cfg(feature = "async")] use async_trait::async_trait;
#[cfg(feature = "async")] use anyhow::Result;

/// Asynchronous key–value cache interface.
#[cfg(feature = "async")]
#[async_trait]
pub trait MemoryAdapter: Send + Sync {
    /// Retrieve previously stored bytes for `key`.  Returns `None` if the key
    /// is absent **or** expired.
    async fn get(&self, key: &str) -> Result<Option<alloc::vec::Vec<u8>>>;

    /// Store `val` under `key` with a time-to-live of `ttl_secs` seconds.
    /// A `ttl_secs` of `0` indicates **no expiration**.
    async fn put(&self, key: &str, val: alloc::vec::Vec<u8>, ttl_secs: u64) -> Result<()>;

    /// Remove value associated with `key` *if present*.
    ///
    /// Implementations MUST treat deletion of a missing key as a **no-op** and
    /// return `Ok(())` so callers can issue idempotent clean-up calls without
    /// the need for an existence check.
    async fn delete(&self, key: &str) -> Result<()>;
}