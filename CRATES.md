# Toka Workspace – Crate Inventory

> This file is the single-source-of-truth for crate purpose and architecture. Update it **immediately** when the crate structure changes so LLMs can build an accurate mental model without scanning the entire tree.

| Crate Name              | Rule-of-Thumb Reason* | One-line Purpose |
|-------------------------|-----------------------|------------------|
| `toka-primitives`       | ① `no_std` primitives | Fundamental, dependency-free types (IDs, currency, etc.). |
| `toka-core`             | ③ separate lifecycle  | Higher-level domain logic and business rules built on primitives. |
| `toka-security-auth`    | ① + ③                 | Capability-token primitives, crypto-free and `no_std` compatible. |
| `toka-vault`            | ② heavy deps          | **Canonical event store**; the single source of truth for "what happened". |
| `toka-agents`           | ② optional deps       | Default agent implementations for the runtime. |
| `toka-toolkit-core`     | ① light, reusable     | Tool trait and registry abstractions (no heavy deps). |
| `toka-toolkit`          | ② heavy deps          | Batteries-included tool implementations (e.g., file system, shell). |
| `toka-runtime`          | ② heavy deps          | Async host for agents, event bus, and toolkit. |
| `toka-cli`              | ② heavy deps          | Command-line interface for the Toka ecosystem. |
| `smart-embedder`        | ② optional ML deps    | Pluggable utility for generating semantic embeddings from events. |
| `toka`                  | – aggregate crate     | Meta-crate that re-exports common preludes for easy onboarding. |

*Rule-of-Thumb Keys*
① Usable from `no_std` / lean targets
② Contains heavy or optional dependencies
③ Needs an independent release cadence

---

## Workspace Evolution: The Great Consolidation

The workspace has undergone a significant refactoring to simplify its architecture and eliminate redundancy. The core change is the introduction of the `toka-vault` crate as the single, canonical event store.

**This consolidation effort retired the following crates:**
- `toka-events-core`
- `toka-bus-memory`
- `toka-bus-persist`
- `toka-ledger-core`
- `toka-ledger-agents`
- `toka-ledger-finance`
- `toka-storage`
- `toka-secrets`

Their functionality has been merged into `toka-vault` or removed to keep the core lean. The new architecture is simpler, more maintainable, and provides a clear separation of concerns between the event store (`toka-vault`) and the domain logic that uses it.

---

## Testing Roadmap

Testing work is tracked separately to keep context focussed. The high-level phases are:

| Test Phase | Goal | Status |
|-----------|------|--------|
| **T-1** | Seed each crate with at least one integration test template | ✅ complete |
| **T-2** | Achieve 60 %+ line coverage via Tarpaulin | 🔄 in progress |
| **T-3** | Cross-crate integration tests (runtime end-to-end) | ⬜ pending |

Detailed guidelines live in `TESTS.md`.

*Each structural or testing milestone should end with `cargo check --workspace --all-features` running cleanly.*