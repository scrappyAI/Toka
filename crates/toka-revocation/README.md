# Toka Revocation (`toka-revocation`)

> **Status**: 0.2.0-alpha – experimental

Primitives and interfaces for **revoking** capability tokens.  Implements the
[RFC 7009](https://datatracker.ietf.org/doc/html/rfc7009) semantics while
remaining optimised for *service-to-service* traffic inside the Toka runtime.

This crate exposes:

* `RevocationStore` – trait encapsulating the minimal operations (`revoke`,
  `is_revoked`).
* `MemoryStore` – default in-memory implementation behind the `memory-store`
  feature for local development and unit tests.

Production setups are expected to supply their own Postgres, Redis or
HashiCorp Vault backed implementation.

---

## Example

```rust,no_run
use toka_revocation::RevocationStore;
use toka_revocation::memory::MemoryStore; // default impl
use uuid::Uuid;
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() {
    let store = MemoryStore::new();
    let jti = Uuid::new_v4();
    store.revoke(jti, Utc::now() + Duration::minutes(15)).await.unwrap();
    assert!(store.is_revoked(jti).await.unwrap());
}
```

---

## License

Apache-2.0 OR MIT