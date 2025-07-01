# Toka OS – **Agentic Operating System** (v0.1)

> **Status:** *v0.1 – Kernel feature-freeze*  | *Documentation overhaul in progress*

Toka OS is an **agent-centric**, capability‐secured operating system written in Rust.  At its heart sits the deterministic [`toka-kernel`](crates/toka-kernel), a pure state-machine that applies **Operations** (opcodes) against a canonical **WorldState** and emits verifiable domain events.

The 0.1 kernel spec – captured in [`docs/42_toka_kernel_spec_v0.1.md`](docs/42_toka_kernel_spec_v0.1.md) – introduces three foundational primitive families:

1. **Financial primitives** – balance-safe asset minting, burning & transfers
2. **Agent primitives**    – task scheduling, spawning and observation hooks
3. **User primitives (β)**  – human actors with assignable roles

Everything above the kernel (storage back-ends, networking, advanced tooling) is intentionally out of scope for 0.1 and will ship incrementally.

# Vision

Toka's long-term goal is a **self-contained Agentic OS** where autonomous programs collaborate, transact and evolve within cryptographically enforced boundaries.  Economic primitives are first-class citizens: every resource is account-ed, every capability is explicit, and every state transition is transparently logged.

Key design pillars:

| Pillar | Manifestation |
|--------|--------------|
| **Determinism** | Single-threaded kernel → same input, same output |
| **Capability security** | `toka-auth` validates unforgeable permission tokens |
| **Event sourcing** | Append-only log (`toka-events`) enables replay & audits |
| **Extensibility** | New opcodes are additive – no breaking removals |
| **Minimal surface** | Everything non-essential (storage, WASM, networking) lives outside the kernel |

# Workspace Overview (July 2025)

| Crate | Status | Purpose |
|-------|--------|---------|
| **toka-types** | ✅ | Shared primitives (`EntityId`, `Operation`, etc.) |
| **toka-kernel** | ✅ | Deterministic state-machine core |
| **toka-events** | ✅ | Canonical event bus & store |
| **toka-auth** | ✅ | Capability token issuance & validation |
| **toka-toolkit-core** | ✅ | Tool trait + registry (no heavy deps) |
| **toka-tools** | 🟡 | Standard library of reference tools (minimal) |
| **toka-agents** | ⬜ *planned* | Default agent implementations layered atop the kernel |
| **toka-cli** | 🟡 | Developer CLI for interacting with the runtime |

> Legend: ✅ implemented 🟡 minimal / WIP ⬜ missing

# Quick Start

```bash
# Validate build – requires stable Rust 1.78+
cargo check --workspace --all-features

# Launch CLI help
cargo run -p toka-cli -- --help
```

```mermaid
graph TD
  subgraph Application
    Agents
    CLI
  end

  subgraph Kernel
    K[toka-kernel]
  end

  subgraph Infrastructure
    Types[toka-types]
    Auth[toka-auth]
    Events[toka-events]
  end

  Agents -->|Messages| K
  CLI -->|Messages| K
  K --> Events
  K -. verifies .-> Auth
  K -. uses .-> Types
```

## Roadmap (towards v0.1 stable)

| Phase | Goal | Target Date |
|-------|------|------------|
| **K-1** | Kernel v0.1 feature-freeze (done) | 2025-06-30 ✅ |
| **A-1** | Land `toka-agents` crate with default behaviours | ⬜ 2025-07-15 |
| **S-1** | Persist event store on disk (SQLite & RocksDB adapters) | ⬜ 2025-07-30 |
| **W-1** | WASM tool execution via `wasmtime` | ⬜ 2025-08-10 |
| **D-1** | Harden documentation & examples (`cargo doc` must pass `#![deny(missing_docs)]`) | 🟡 rolling |

> The full roadmap lives in [`docs/ROADMAP.md`](docs/ROADMAP.md).

© 2025 Toka Contributors · MIT OR Apache-2.0
