#![forbid(unsafe_code)]
//! Toka Revocation (v0.2-alpha)
//!
//! Pluggable revocation primitives for capability tokens.  The design follows
//! [RFC 7009](https://datatracker.ietf.org/doc/html/rfc7009) but remains
//! opinionated for internal *service-to-service* workloads.
//!
//! The crate is intentionally small:
//! * A single [`RevocationStore`] trait encapsulating the minimal operations.
//! * A reference **in-memory** implementation behind the `memory-store`
//!   feature flag (enabled by default).  Production deployments are expected
//!   to bring their own Postgres/Redis-backed implementation.
//!
//! ## Roadmap
//! * Redis store – constant-time look-ups with automatic key expiry.
//! * Postgres store – transactional revocation + auditing.
//! * gRPC & HTTP adapters under a sibling `toka-revocation-srv` crate.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Contract for storing and querying *revoked* capability tokens.
#[async_trait]
pub trait RevocationStore: Send + Sync + 'static {
    /// Persist a token identifier so subsequent validations can reject it.
    async fn revoke(&self, jti: Uuid, expires_at: DateTime<Utc>) -> Result<()>;

    /// Returns `true` if the token identifier has been revoked.
    async fn is_revoked(&self, jti: Uuid) -> Result<bool>;
}

// -------------------------------------------------------------------------------------------------
// In-memory store (dev/Test only)
// -------------------------------------------------------------------------------------------------

#[cfg(feature = "memory-store")]
mod memory {
    use super::*;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    /// Non-persistent in-memory revocation list – suitable for tests only.
    #[derive(Debug, Default)]
    pub struct MemoryStore {
        map: Mutex<HashMap<Uuid, DateTime<Utc>>>,
    }

    #[async_trait]
    impl RevocationStore for MemoryStore {
        async fn revoke(&self, jti: Uuid, expires_at: DateTime<Utc>) -> Result<()> {
            self.map.lock().insert(jti, expires_at);
            Ok(())
        }

        async fn is_revoked(&self, jti: Uuid) -> Result<bool> {
            let map = self.map.lock();
            Ok(map.get(&jti).map_or(false, |&exp| exp > Utc::now()))
        }
    }

    impl MemoryStore {
        /// Convenience helper for tests.
        pub fn new() -> Self {
            Self::default()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn memory_store_roundtrip() {
            let store = MemoryStore::new();
            let jti = Uuid::new_v4();
            let exp = Utc::now() + chrono::Duration::minutes(5);
            assert!(!store.is_revoked(jti).await.unwrap());
            store.revoke(jti, exp).await.unwrap();
            assert!(store.is_revoked(jti).await.unwrap());
        }
    }
}