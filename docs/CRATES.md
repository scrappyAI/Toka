# Toka Workspace – Crate Inventory (v0.2.1)

> This document is the single-source-of-truth for crate purpose and architecture. **Update it immediately** when the workspace structure changes so humans and LLMs can build an accurate mental model without scanning the entire tree.

## Core Deterministic Layer (Build Order 0-1)

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-types`               | ① `no_std` primitives | Pure POD structs & enums (EntityId, Operation, Message, KernelEvent). |
| `toka-auth`                | ① minimal crypto      | JWT-based capability token validation and claims processing. |
| `toka-bus-core`            | ① deterministic       | Lightweight, in-memory event broadcasting with no I/O. |
| `toka-kernel`              | ② core runtime        | Deterministic state-machine core with agent primitives only. |

## Storage Layer (Build Order 2-3)

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-store-core`          | ① storage traits      | Pure storage abstractions with no concrete implementations. |
| `toka-store-memory`        | ② memory impl         | Fast, non-persistent storage driver for testing/development. |
| `toka-store-sled`          | ② persistent impl     | Sled-based persistent storage driver with ACID guarantees. |

## Runtime Layer (Build Order 4)

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-runtime`             | ② fuzzy/async         | Bridges deterministic kernel with storage and provides configuration. |

## Tools and Security

| Crate Name                 | Rule-of-Thumb Reason* | One-line Purpose |
|----------------------------|-----------------------|------------------|
| `toka-tools`               | ② optional deps       | Standard library of agent tools and utility helpers. |
| `security/*`               | ③ independent         | Security-related utilities and cryptographic primitives. |

*Rule-of-Thumb Keys*
① Usable from `no_std` / lean targets  
② Contains heavy or optional dependencies  
③ Needs an independent release cadence

---

## Architecture Highlights (v0.2.1)

The **v0.2.1 Kernel Refactor** established a clean separation between deterministic core operations and fuzzy user-space concerns:

### Key Changes
- **Pure OS Kernel**: `toka-kernel` treats itself as a pure OS kernel for agent operating systems
- **Layer Separation**: Deterministic core (0-1) → Storage abstractions (2-3) → Runtime coordination (4) 
- **No Heavy Dependencies**: Kernel avoids `tokio::time`, `rand`, `std::env`, and I/O operations
- **Pluggable Storage**: Multiple storage drivers (memory, sled) implement common `StorageBackend` trait
- **Event Bus Extraction**: Lightweight bus moved to `toka-bus-core` with no external dependencies

### Removed Legacy Components
- **`toka-events`** ➜ Split into `toka-bus-core` + `toka-store-*` family
- **Finance/User Opcodes** ➜ Moved to user-space (will be re-established as extension crates)
- **CLI Application** ➜ Removed pending new CLI that uses runtime layer properly

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