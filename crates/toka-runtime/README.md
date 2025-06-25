# Toka Runtime

Async host that wires **agents**, the **canonical event store** (`toka-events`) and an optional **toolkit** into a single executable.

---

## Highlights

1. **Event-sourced by design** – Every event goes through the embedded `EventBus` and can be streamed to or replayed from the vault.
2. **Opt-in surface area** – Keep binaries tiny by enabling only the feature flags you need:

| Feature | What you get | Major extra deps |
|---------|--------------|------------------|
| *(none)* | Pure runtime scaffold (async loops, `EventBus`) | `tokio`, `tracing` |
| `vault` | Secure local event store backend (sled or in-mem) | `toka-events` |
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

* **Single Source of Truth** – The runtime no longer embeds multiple bus / ledger back-ends.  All persistence is delegated to [`toka-events`](../../toka-events/README.md) as described in [`EVENT_SYSTEM_REFACTOR.md`](../../EVENT_SYSTEM_REFACTOR.md).
* **Pluggable Storage** – `storage("local")`, `storage("vault")` etc. allow agents & tools to store artefacts without depending on a concrete backend.
* **Graceful Degradation** – If you compile without `toolkit` the public API still works; you just get a no-op tool registry.

---

## Status

Early alpha (≥ v0.2.0-alpha).  Expect sharp edges and breaking changes while we stabilise the lifecycle APIs.

---

## License

Apache-2.0 OR MIT

© 2024 Toka Contributors 

---

### Enabling Secret Rotation (`auth` feature)

With the `auth` feature enabled the runtime manages a **secret pool** used to
sign capability tokens.  Two knobs are available via `RuntimeConfig`:

| Field | Description | Default |
|-------|-------------|---------|
| `initial_secret` | Bootstrap key (if `None` a random 256-bit key is generated) | `None` |
| `retired_ttl_secs` | Grace period during which _retired_ secrets remain valid | `300` |

Example:

```rust,ignore
let mut cfg = RuntimeConfig::default();
cfg.initial_secret = Some("my-bootstrap-secret".into());
cfg.retired_ttl_secs = 120; // 2-minute overlap
```

You can rotate the active secret at runtime:

```rust,ignore
runtime.rotate_secrets();
```

Or, via the CLI (build with `--features auth,vault,toolkit`):

```bash
$ toka auth rotate-secret
✅ Secret rotated successfully
``` 