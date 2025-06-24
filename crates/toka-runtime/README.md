# Toka Runtime

Async host that wires **agents**, the **canonical event store** (`toka-vault`) and an optional **toolkit** into a single executable.

---

## Highlights

1. **Event-sourced by design** – Every event goes through the embedded `EventBus` and can be streamed to or replayed from the vault.
2. **Opt-in surface area** – Keep binaries tiny by enabling only the feature flags you need:

| Feature | What you get | Major extra deps |
|---------|--------------|------------------|
| *(none)* | Pure runtime scaffold (async loops, `EventBus`) | `tokio`, `tracing` |
| `vault` | Secure local vault backend (sled or in-mem) | `toka-vault` |
| `toolkit` | `ToolRegistry`, CLI, type-erased `serde` plugin system | `toka-toolkit`, `clap`, `typetag` |
| `auth` | Capability token helpers | `toka-security-auth` |

The matrix composes – e.g. `features = ["vault", "toolkit"]` gives you a full local dev stack.

---

## Quick Start

```toml
[dependencies]
toka-runtime = { version = "0.1", features = ["vault", "toolkit"] }
```

```rust,ignore
use toka_runtime::runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = RuntimeConfig::default();
    let runtime = Runtime::new(cfg).await?;
    runtime.start().await?;

    // … register agents, emit events …

    runtime.stop().await?;
    Ok(())
}
```

---

## Design Decisions

* **Single Source of Truth** – The runtime no longer embeds multiple bus / ledger back-ends.  All persistence is delegated to [`toka-vault`](../../toka-vault/README.md) as described in [`EVENT_SYSTEM_REFACTOR.md`](../../EVENT_SYSTEM_REFACTOR.md).
* **Pluggable Storage** – `storage("local")`, `storage("vault")` etc. allow agents & tools to store artefacts without depending on a concrete backend.
* **Graceful Degradation** – If you compile without `toolkit` the public API still works; you just get a no-op tool registry.

---

## Status

Early alpha (≥ v0.2.0-alpha).  Expect sharp edges and breaking changes while we stabilise the lifecycle APIs.

---

## License

Apache-2.0 OR MIT

© 2024 Toka Contributors 