# Toka Workspace – Crate Inventory (2025)

> This document is the single-source-of-truth for crate purpose and architecture. **Update it immediately** when the workspace structure changes so humans and LLMs can build an accurate mental model without scanning the entire tree.

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-types`               | ① `no_std` primitives | Fundamental, dependency-light types (EntityId, Operation, Message). |
| `toka-kernel`              | ② core runtime        | Deterministic state-machine core with capability validation and agent primitives. |
| `toka-auth`                | ② crypto deps         | JWT-based capability token validation and claims processing. |
| `toka-events`              | ② heavy deps          | Canonical event store with persistent storage (sled) and bus integration. |
| `toka-runtime`             | ② heavy deps          | Async host orchestrating kernel, agents, tools, and event bus. |
| `toka-tools`               | ② optional deps       | Standard library of agent tools and utility helpers. |
| `security`                 | ③ independent         | Security-related utilities and cryptographic primitives. |

*Rule-of-Thumb Keys*
① Usable from `no_std` / lean targets  
② Contains heavy or optional dependencies  
③ Needs an independent release cadence

---

## Workspace Evolution 2025

The 2025 *Kernel Refactor* (v0.2) removed finance and user opcode families from the core kernel, establishing a **minimal, agent-centric nucleus**. Domain-specific functionality is expected to live in extension crates that plug into the kernel via the new `OpcodeHandler` registry.

### Key Changes
- **`toka-kernel`** now contains only agent primitives (`ScheduleAgentTask`, `SpawnSubAgent`, `EmitObservation`)
- **Finance & user families** removed from core; will be re-established as extension crates
- **`toka-types`** simplified to core operation enum without domain-specific variants
- **Extension mechanism** introduced via `OpcodeHandler` trait for pluggable opcode families

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