# Toka-Kernel – v0.1 Specification (Draft)

> Status: **Draft** | Version: 0.1 | Last-updated: 2025-07-01

---

## 1  Purpose

`crates/toka-kernel` is the _deterministic state-machine core_ of the **Toka** agentic OS.  
It exposes a **capability-guarded** syscall surface—called **_Operations (Opcodes)_**—through which every
agent, user or subsystem mutates shared state.  
Version 0.1 focuses exclusively on:

* **Financial primitives** – secure asset & balance manipulation
* **Agent primitives** – task orchestration & observation
* **User primitives (minimal)** – human actors as first-class entities

Everything else (storage back-ends, networking, advanced tooling) is deliberately out of
scope for this first milestone.

---

## 2  Glossary

| Term | Meaning |
|------|---------|
| **Entity** | Addressable object identified by a `u128` (`EntityId`). May be an `Agent`, `User` or `System` module. |
| **Agent** | Autonomous program running inside/outside the cluster, authenticated via capabilities. |
| **User** | Human principal represented by an entity row. |
| **Operation / Opcode** | Canonical verb understood by the kernel (e.g. `TransferFunds`). |
| **Message** | Authenticated envelope `{ origin, capability, operation }` submitted to the kernel. |
| **Capability** | Unforgeable token describing _exactly_ which operations an entity may invoke. |
| **Event** | Immutable record emitted after successful execution, enabling event sourcing. |
| **WorldState** | Canonical view of all in-memory tables (balances, agent queues …). |

---

## 3  Architectural Principles

1. **Deterministic** – Same input → same output; required for replay & auditing.
2. **Capability Secure** – No operation executes unless the submitted capability authorises it.
3. **Extensible** – New opcodes are _additive_ (never breaking); old ones remain¹.
4. **Replayable** – Entire state is derivable from the ordered event log (CQS / ES).
5. **Observable** – Every state-change yields a typed event for downstream projections.

> ¹ _Breaking opcode removal requires a major version bump (semver)._  

---

## 4  High-Level Architecture

```mermaid
graph TD
    subgraph Application Layer
        U[Users / CLI]
        A[Agents]
    end

    subgraph Kernel Layer
        K[toka-kernel]
        K_dispatcher[Dispatcher]
        K_validator[Validator]
        K_executor[Executor]
        K_emitter[EventEmitter]
        K_state[(WorldState)]
    end

    subgraph Platform Crates
        Types[toka-types]
        Events[toka-events]
        Auth[toka-auth]
        Tools[toka-tools]
    end

    U -->|submit(Message)| K
    A -->|submit(Message)| K
    K --> Events
    K --> Auth
    Tools --> K

    K_dispatcher --> K_validator --> K_executor --> K_state --> K_emitter --> Events
```

---

## 5  Crate Layout (v0.1)

| Crate | Responsibility | Depends on |
|-------|----------------|------------|
| `toka-kernel` | Validates & executes operations, emits events | `toka-types`, `toka-auth`, `toka-events` |
| `toka-types` | Shared data structures (`EntityId`, `Operation`, `Event`) | – |
| `toka-events` | Event bus abstraction + event definitions | `toka-types` |
| `toka-auth` | Capability tokens & validation traits | `toka-types` |

---

## 6  Data Model

```rust
// crates/toka-types/src/lib.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EntityId(pub u128);

#[derive(Clone, Debug)]
pub enum Operation {
    /* — financial — */
    TransferFunds { from: EntityId, to: EntityId, amount: u64 },
    MintAsset     { asset: EntityId, to: EntityId, amount: u64 },
    BurnAsset     { asset: EntityId, from: EntityId, amount: u64 },
    /* — agent — */
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    EmitObservation   { agent: EntityId, data: Vec<u8> },
    /* — user — */
    CreateUser  { alias: String },
    AssignRole  { user: EntityId, role: Role },
}

#[derive(Clone, Debug)]
pub struct Message {
    pub origin: EntityId,
    pub capability: Capability,
    pub op: Operation,
}
```

`TaskSpec`, `AgentSpec` and `Role` live in `toka-types` but are trimmed here for brevity.

---

## 7  Opcode Semantics

| Opcode | Preconditions | Postconditions | Failure cases |
|--------|---------------|---------------|---------------|
| **TransferFunds** | `capability.allows("transfer", from)` & `balance(from) ≥ amount` | Debits `from`, credits `to`, emits `FundsTransferred` | `InsufficientBalance`, `CapDenied` |
| **MintAsset** | `capability.allows("mint", asset)` | Increases total supply, credits `to`, emits `AssetMinted` | `CapDenied` |
| **BurnAsset** | `capability.allows("burn", asset)` & `balance(from) ≥ amount` | Reduces supply, debits `from`, emits `AssetBurned` | `InsufficientBalance`, `CapDenied` |
| **ScheduleAgentTask** | `cap.allows("schedule", agent)` | Enqueues task in agent inbox; emits `TaskScheduled` | `CapDenied` |
| **SpawnSubAgent** | `cap.allows("spawn", parent)` | Persists new agent entity; emits `AgentSpawned` | `CapDenied` |
| **EmitObservation** | `cap.allows("observe", agent)` | Stores observation; emits `ObservationEmitted` | `CapDenied` |

> The exhaustive machine-readable semantics live in `docs/protocols/opcode_schema.yaml` (to be authored post-0.1).

---

## 8  Execution Flow

```rust
use toka_kernel::{Kernel, KernelError};
use toka_types::{Message, Operation};

fn main() -> Result<(), KernelError> {
    // Boot an in-memory kernel (durable storage arrives in v0.2)
    let mut kernel = Kernel::default();

    let op = Operation::TransferFunds {
        from: alice_id(),
        to: bob_id(),
        amount: 10_000,
    };

    let msg = Message::new(alice_id(), cap_transfer(&alice_id()), op);

    let event = kernel.submit(msg)?;
    println!("✅ event emitted: {event:?}");
    Ok(())
}
```

Steps inside `Kernel::submit(msg)`:

1. **Dispatch** by enum tag → handler function.
2. **Validate capability** (`toka-auth`) & business-level preconditions.
3. **State transition** applied to `WorldState` (in-memory HashMaps for v0.1).
4. **Emit** typed `Event` pushed to `toka-events` bus.
5. **Return** event (or `KernelError`).

All steps execute in a single synchronous thread for maximal determinism; async boundaries are at the _message ingress_ and _event egress_.

---

## 9  Error Model

```rust
pub enum KernelError {
    CapabilityDenied(CapabilityId),
    InsufficientBalance { entity: EntityId, needed: u64, available: u64 },
    UnknownEntity(EntityId),
    InvalidOperation(String),
    // …extendable
}
```

All errors are **non-panic, deterministic** and serialisable for remote callers.

---

## 10  Storage & Persistence (Roadmap)

* **v0.1** – In-memory `HashMap` tables; snapshot via `serde_json` for test fixtures.
* **v0.2** – Pluggable storage engine trait (`StorageAdapter`) with Postgres + RocksDB impls.
* **v0.3** – Incremental event sourcing with catch-up replays & snapshotting.

---

## 11  Integration Guidelines

1. Link `toka-kernel` with `default-features = false` if you want a minimal footprint.
2. Always submit a **capability token** with the message—kernels compiled **in release mode will refuse `capability.is_none()` even in tests**.
3. Handle `KernelError::CapabilityDenied` by request‐replying with HTTP `403` or gRPC status `PERMISSION_DENIED`.
4. Subscribe to `toka-events::Topic::Kernel` to project your own read-models.

---

## 12  Open Questions

* Do we need _gas accounting_ for every operation or only for external calls?  (→ v0.2 discussion)
* Should the kernel own task-scheduling or delegate to an `toka-agents-runtime` crate? (analysis ongoing)
* User roles vs. capabilities overlap—merge or keep separate?

---

## 13  Changelog

| Date | Change | Author |
|------|--------|--------|
| 2025-07-01 | Initial draft extracted & refined from design thread | DS |

---

## 14  References

* [40_capability_tokens_spec_v0.1.md](./40_capability_tokens_spec_v0.1.md)
* *Design Note* – `prompts/agentic_kernel_notes.md`

---

© 2025 Toka Project – Apache 2.0 / MIT dual-licensed.