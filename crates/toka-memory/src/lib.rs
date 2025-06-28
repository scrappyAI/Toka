//! # Toka Memory – lightweight, in-process key–value cache
//!
//! Slice 3 of the SIMPLECT refactor introduces a small, **async** cache
//! abstraction living in its own crate so other subsystems (agents, runtime,
//! projections) can share a pluggable memory model without pulling heavy
//! databases.
//!
//! The API is intentionally minimal: single get/put with a TTL.  Callers that
//! need more complex semantics (atomic counters, CAS etc.) should wrap their
//! own newtype around [`MemoryAdapter`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use anyhow::Result;
use async_trait::async_trait;
use toka_memory_api::MemoryAdapter;

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
} 