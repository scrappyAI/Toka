//! # Toka Memory – reference in-process adapter
//!
//! This crate ships a **Tokio-based**, in-process implementation of the
//! [`toka_memory_api::MemoryAdapter`] trait.  It is optimised for *tests* and
//! lightweight evaluation setups, *not* for production workloads that require
//! persistence or horizontal scalability.
//!
//! ## Role in the hierarchy
//! The adapter sits at the *ephemeral agent state* layer (see the table in
//! `toka-memory-api`) – data is kept in the host process and therefore lost
//! once the process terminates.  Nevertheless it is a convenient default for:
//! * Unit/integration tests where determinism and zero external services are
//!   desirable.
//! * Local prototyping of agent behaviour before connecting to Redis, RocksDB
//!   or a vector store.
//!
//! ## Guarantees
//! * **Thread-safe** (`Send` + `Sync`) via `Arc<RwLock<…>` internally.
//! * **No background task** – expired entries are cleared lazily on access to
//!   keep the implementation tiny.
//! * **forbid(unsafe_code)** – 100 % safe Rust.
//!
//! ### Performance considerations
//! Every operation acquires a write lock because we mutate expiration metadata
//! on reads.  This is acceptable for tests but not for high-throughput agent
//! fleets.  Production deployments should provide a dedicated adapter
//! optimised for read concurrency and external persistence.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use anyhow::Result;
use async_trait::async_trait;
pub use toka_memory_api::MemoryAdapter;

// -------------------------------------------------------------------------------------------------
// Default in-memory implementation
// -------------------------------------------------------------------------------------------------

/// Internal entry wrapper so we can check expiration lazily.
#[derive(Debug, Clone)]
struct Entry {
    val: Vec<u8>,
    expires_at: Option<Instant>,
}

/// Simple process-local [`MemoryAdapter`] good enough for tests & small
/// workloads.  Expired values are removed upon access; there is **no**
/// background vacuum task to keep the implementation tiny.
#[derive(Debug, Default)]
pub struct InMemoryAdapter {
    inner: Arc<RwLock<HashMap<String, Entry>>>,
}

impl InMemoryAdapter {
    /// Construct an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    fn is_expired(entry: &Entry) -> bool {
        if let Some(exp) = entry.expires_at {
            Instant::now() >= exp
        } else {
            false
        }
    }
}

#[async_trait]
impl MemoryAdapter for InMemoryAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get(key) {
            if Self::is_expired(entry) {
                // eager cleanup
                guard.remove(key);
                return Ok(None);
            }
            return Ok(Some(entry.val.clone()));
        }
        Ok(None)
    }

    async fn put(&self, key: &str, val: Vec<u8>, ttl_secs: u64) -> Result<()> {
        let expires_at = if ttl_secs == 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(ttl_secs))
        };
        let entry = Entry { val, expires_at };
        self.inner.write().await.insert(key.to_owned(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.inner.write().await.remove(key);
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_get_roundtrip() -> Result<()> {
        let cache = InMemoryAdapter::new();
        cache.put("foo", b"bar".to_vec(), 0).await?;
        let v = cache.get("foo").await?.unwrap();
        assert_eq!(v, b"bar".to_vec());
        Ok(())
    }

    #[tokio::test]
    async fn test_ttl_expiry() -> Result<()> {
        let cache = InMemoryAdapter::new();
        cache.put("k", b"v".to_vec(), 1).await?; // 1 sec ttl
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(cache.get("k").await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_removes_key() -> Result<()> {
        let cache = InMemoryAdapter::new();
        cache.put("a", b"b".to_vec(), 0).await?;
        cache.delete("a").await?;
        assert!(cache.get("a").await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_nonexistent_noop() -> Result<()> {
        let cache = InMemoryAdapter::new();
        // should not error
        cache.delete("missing").await?;
        Ok(())
    }
} 