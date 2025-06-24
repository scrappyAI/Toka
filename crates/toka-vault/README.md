# Toka Vault

> Canonical event store for the Toka ecosystem

`toka-vault` is the single source of truth for **what happened** in a Toka-powered system. It stores immutable events, supports live subscriptions, and allows reliable replay for audit or projection building.

---

## Feature Matrix

| Feature flag | Purpose | Extra Deps |
|--------------|---------|------------|
| _default_ | In-memory vault – perfect for testing and ephemeral workflows | `tokio`, `serde` |
| `persist-sled` | Durable storage on top of [sled](https://github.com/spacejam/sled) | `sled`, `bincode` |
| *(coming soon)* `intent-cluster` | Semantic grouping of events | _TBD – design in progress_ |

The crate ships **without heavy dependencies** by default. Enable only what you need.

---

## Quick Start

Add a dependency in your `Cargo.toml`:

```toml
# In-memory only
[dependencies]
toka-vault = "0.1"
```

With durable sled back-end:

```toml
[dependencies]
toka-vault = { version = "0.1", features = ["persist-sled"] }
```

Example usage:

```rust
use toka_vault::{Vault, InMemoryVault};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Spin up an in-memory vault
    let vault = Vault::Memory(InMemoryVault::new(1024)?);

    // Subscribe before we emit to avoid missing events
    let mut sub = vault.subscribe();

    // Commit an event (payload can be any `serde::Serialize` type)
    let header = vault.commit_json("user.login", serde_json::json!({"user": "alice"}), &[]).await?;
    println!("committed event {}", header.id);

    // Receive it back on the subscription
    if let Some(event) = sub.next().await {
        println!("got {} -> {:?}", event.header.kind, event.payload);
    }

    Ok(())
}
```

---

## Design Goals

1. **Causality** – Each event carries its parents' IDs; vault enforces a partial order.
2. **Content Addressing** – Payloads are stored by Blake3 hash to avoid duplication.
3. **Streaming First** – The API exposes `tokio::broadcast` receivers for low-latency streaming.
4. **Pluggable Storage** – Back-ends implement the same `EventStore` trait.

---

## On Intent Clustering

Intent clustering (automatic semantic grouping of events) is **on hold** while we flesh out the concept. The `intent-cluster` feature flag exists but is _not yet implemented_. Feel free to contribute ideas or proofs-of-concept in a draft PR!

---

## Testing

```bash
# run all vault tests (in-memory only)
cargo test -p toka-vault

# include durable back-end tests
cargo test -p toka-vault --features persist-sled
```

---

## License

Dual-licensed under MIT or Apache-2.0, at your option. 