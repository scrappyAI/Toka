# Toka Memory

Lightweight, async **in-process** key–value cache powering short-lived memory needs across the Toka workspace.

## Motivation
Agents, runtime components and projections frequently need to stash small blobs of state for a brief time (think *conversation context* or *idempotency keys*).  Pulling in a full database adds latency, complexity and a deployment surface area that defeats the purpose.

`toka-memory` provides a **minimal** abstraction – a single `MemoryAdapter` trait with `get`/`put` and TTL – plus a tiny `InMemoryAdapter` default implementation suitable for tests and single-process workloads.

```rust
use toka_memory::{InMemoryAdapter, MemoryAdapter};

# async fn demo() -> anyhow::Result<()> {
let cache = InMemoryAdapter::new();
cache.put("foo", b"bar".to_vec(), 300).await?; // 5 min TTL
assert_eq!(cache.get("foo").await?.unwrap(), b"bar".to_vec());
# Ok(())
# }
```

## Guarantees
1. **No unsafe** – the crate is `#![forbid(unsafe_code)]`.
2. **Zero background threads** – expired keys purge lazily on access to keep the footprint tiny.
3. **Battle-tested** – covered by async unit tests and fuzz-friendly deterministic timing.

For distributed deployments, swap the adapter out for a Redis-backed implementation without changing call-sites. 