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
| `auth` | Capability token helpers | `toka-capability-jwt-hs256` |

The matrix composes – e.g. `