//! Storage API crate – defines the minimal asynchronous key–value artefact
//! storage contract for Toka services.
//!
//! Enable the `async` feature (default) to pull in `async_trait` and `anyhow`.
#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;
use alloc::boxed::Box;

#[cfg(feature = "async")] use async_trait::async_trait;
#[cfg(feature = "async")] use anyhow::Result;

/// Pluggable artefact store abstraction.
#[cfg(feature = "async")]
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    /// Store raw bytes under `key`.
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<()>;
    /// Retrieve bytes previously stored under `key`.
    async fn get(&self, key: &str) -> Result<Option<alloc::vec::Vec<u8>>>;
    /// Remove the item – succeed even if absent.
    async fn delete(&self, key: &str) -> Result<()>;
}

/// Crate prelude for convenient downstream import.
#[cfg(feature = "async")]
pub mod prelude {
    pub use super::StorageAdapter;
}