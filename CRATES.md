# Toka Workspace – Crate Inventory (2025)

> This document is the single-source-of-truth for crate purpose and architecture. **Update it immediately** when the workspace structure changes so humans and LLMs can build an accurate mental model without scanning the entire tree.

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-primitives-api`      | ① `no_std` primitives | Fundamental, dependency-free types (IDs, currency, etc.). |
| `toka-events-api`          | ① contract-only       | Pure data types and traits for the canonical event subsystem. |
| `toka-bus-api`             | ① contract-only       | Minimal, `no_std`-friendly event-bus traits and headers. |
| `toka-memory-api`          | ① contract-only       | Trait contract for agent memory adapters (byte-oriented get/put/delete, no_std). |
| `toka-storage-api`         | ① contract-only       | Async key–value artefact storage contract. |
| `toka-capability`          | ② implementation     | Aggregator crate re-exporting capability primitives (deprecated shim). |
| `toka-capability-core`     | ① `no_std` primitives | Canonical Claims struct + capability traits (no crypto). |
| `toka-capability-jwt-hs256`| ② implementation      | HS256 JWT implementation of capability tokens. |
| `toka-agents`              | ② optional deps       | Default agent implementations for the runtime. |
| `toka-toolkit-core`        | ① light, reusable     | Tool trait and registry abstractions (zero heavy deps). |
| `toka-tools`               | ② optional deps       | Standard library of agent tools (currently minimal `echo`). |
| `toka-bus`                 | ② lightweight runtime | In-process, async event-bus implementation (Tokio broadcast). |
| `toka-memory`              | ② lightweight runtime | Reference in-process adapter (Tokio + RwLock, suited for tests & prototyping). |
| `toka-storage`             | ② heavy deps          | Local-filesystem storage adapter used by the runtime. |
| `toka-events`              | ② heavy deps          | **Canonical event store** replacing the historical `toka-vault`. |
| `toka-runtime`             | ② heavy deps          | Async host tying agents, tools, bus & event store together. |
| `toka`                     | – aggregate crate     | Meta-crate re-exporting common preludes for quick onboarding. |
| `toka-cli` (app)           | ② heavy deps          | Command-line interface for interacting with the runtime. |

*Rule-of-Thumb Keys*
① Usable from `no_std` / lean targets  
② Contains heavy or optional dependencies  
③ Needs an independent release cadence

---

## Workspace Evolution 2025

The 2025 *Great Consolidation* introduced the new [`toka-events`](crates/toka-events) crate as the single, canonical event store and simplified inter-crate boundaries. All retired crates (`toka-vault`, `toka-ledger-*`, etc.) have now been **fully removed** from the repository. Historical notes live in `/docs/history/` for posterity.

---

## Testing Roadmap

Testing work is tracked separately to keep context focused. The high-level phases are:

| Test Phase | Goal | Status |
|-----------|------|--------|
| **T-1** | Seed each crate with at least one integration-test template | ✅ complete |
| **T-2** | Achieve 60 %+ line coverage via Tarpaulin | 🔄 in progress |
| **T-3** | Cross-crate integration tests (runtime end-to-end) | ⬜ pending |

Detailed guidelines live in [`TESTS.md`](TESTS.md).

*Every structural or testing milestone must end with `cargo check --workspace --all-features` running cleanly.*