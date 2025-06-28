//! Toka Memory – API crate
//!
//! This lightweight crate exposes the [`MemoryAdapter`] trait used across the
//! workspace for ephemeral key–value caching.  It purposefully avoids heavy
//! dependencies so alternate back-ends (e.g. Redis, sled) can implement the
//! contract without carrying unnecessary baggage.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;
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
}