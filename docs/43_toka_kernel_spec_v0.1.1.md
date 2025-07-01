# Toka-Kernel – v0.1.1 Specification (Draft)

> Status: **Draft**  |  Version: **0.1.1**  |  Last-updated: 2025-07-01
>
> This document supersedes the initial [v0.1 spec](42_toka_kernel_spec_v0.1.md) and
> captures the refocus of **toka-kernel** as a _pure, capability-secured agent
> micro-kernel_.
>
> v0.1.1 removes finance & user primitives from the default build and frames the
> kernel through the lens of classical OS concerns—**process management,
> memory, scheduling, drivers & filesystems**—re-mapped to an *agentic* runtime.
>
---

## 1 Purpose

`toka-kernel` is the deterministic **nucleus** of the Toka Agentic Operating
System.  All state transitions must pass through it so that:

1. Every mutation is **authorised** by capability tokens.
2. The resulting event log is **replayable** and auditable.
3. Sub-systems (storage, networking, finance, identity …) can evolve
   independently while preserving a stable, minimal contract.

v0.1.1 narrows the contract to **agent primitives only** and introduces an
extensible *kernel surface* grouped after conventional OS disciplines:

| Discipline | Agentic Re-interpretation | Status |
|------------|--------------------------|---------|
| **Process Mgmt.** | Agents & their sub-tasks | ✅ v0.1
| **Memory Mgmt.** | In-memory `WorldState` tables, deterministic snapshotting | ✅ v0.1
| **Scheduling** | FIFO task queues per agent; pluggable schedulers | ⬜ v0.2
| **Drivers** | Plugin "device drivers" expose host resources (HTTP, WASM, …) via capability-guarded syscalls | ⬜ v0.2
| **File Systems** | Key-value stores & content-addressed blobs abstracted behind drivers | ⬜ v0.3
| **IPC / Signals** | Event bus (`toka-events`) for intra-agent communication | ✅ v0.1

Finance, identity & higher-level domains move to *toolkits* that sit _on top_ of
this surface.

---

## 2 Glossary (Additions)

| Term | Meaning |
|------|---------|
| **Agent Process (AP)** | Logical execution context identified by an `EntityId`. |
| **Task** | Deterministic function scheduled inside an AP (like a thread). |
| **Driver Capability** | Token that authorises access to a host driver (e.g. "kv.put"). |
| **Kernel Interface (KI)** | Stable API exposed by the kernel (`Message`, `Operation`). |

---

## 3 What's New in 0.1.1

1. **Feature-gated Families** – Financial & user opcodes now live behind the
   `finance` / `user` feature flags (disabled by default).
2. **WorldState Slimming** – Balances & user tables are compiled only when the
   associated feature is enabled.
3. **`UnsupportedOperation` Error** – Deterministic rejection when a caller
   submits an opcode that isn't compiled in.
4. **Kernel Drivers RFC** – Placeholder interface for host drivers (v0.2).
5. **Spec Structure** – Sections now map to typical OS kernel responsibilities.

---

## 4 Kernel Surface (v0.1.1)

### 4.1 Process Management

* **OpCodes**
  * `ScheduleAgentTask` – enqueue a deterministic task.
  * `SpawnSubAgent` – fork a child AP inheriting capabilities.
  * `EmitObservation` – publish state for offline analysis.
* **State**
  * `agent_tasks: HashMap<EntityId, Vec<TaskSpec>>` (always compiled).

### 4.2 Memory Management

* Single-threaded, in-memory `HashMap` tables.
* Snapshots via `serde_json` (CLI helper) until v0.2 storage adapters.
* Deterministic hashing planned for state proofs (v0.3).

### 4.3 Scheduling

* Current scheduler = simple FIFO per AP, executed synchronously when the task
  is committed (no pre-emption).
* v0.2 will introduce an async scheduler trait allowing round-robin or priority
  policies.

### 4.4 Drivers & I/O

> _Pluggable drivers expose host resources behind capability checks.  Draft
> design only._

* `DriverId` names a driver (e.g. `kv`, `http`, `wasm`).
* New opcode `DriverCall { driver: DriverId, payload: Vec<u8> }` _(v0.2)_
* The kernel merely **routes** the call to a registered driver implementation
  compiled in; drivers are free to be async & side-effectful.

### 4.5 File Systems

* Not addressed in 0.1.1 – will piggy-back on the driver interface (e.g.
  `driver = "fs", op = Put { path, bytes }`).

### 4.6 Inter-Process Communication (IPC)

* Implemented via `toka-events` broadcast bus.
* Future work: selective subscriptions, causal ordering guarantees.

---

## 5 Operation Enumeration (Agent-only build)

```rust
enum Operation {
    // Process / Tasking
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    EmitObservation   { agent: EntityId, data: Vec<u8> },

    // Optional families (feature-gated)
    #[cfg(feature = "finance")]
    TransferFunds { from: EntityId, to: EntityId, amount: u64 },
    // …etc
}
```

---

## 6 Kernel Error Model (excerpt)

```rust
pub enum KernelError {
    CapabilityDenied,
    InvalidOperation(String),
    UnsupportedOperation, // ⇽ new in 0.1.1
}
```

---

## 7 Execution Flow

1. **Dispatch** by enum tag (compile-time exhaustive).
2. **Authorise** via capability token.
3. **Execute** handler ⇒ mutate `WorldState`.
4. **Emit** `KernelEvent` onto bus.

Same as v0.1 but with compile-time pruning of unused families.

---

## 8 Roadmap

| Version | Theme | ETA |
|---------|-------|-----|
| **0.1.2** | Async task scheduler trait | 2025-07-15 |
| **0.2.0** | Storage adapters & driver API | 2025-07-30 |
| **0.3.0** | Deterministic snapshots & state proofs | 2025-08-20 |

---

## 9 Backwards Compatibility

* v0.1 `finance` & `user` opcodes remain **unchanged**—they just need the
  feature flag now.
* Event variants follow the same gating; downstream consumers should compile
  with matching features.

---

© 2025 Toka Project — Apache-2.0 / MIT