# Toka Kernel v0.1.1 – Codebase Alignment Report (2025-07-01)

> **Author:** Automated Research Agent (o3)  
> **Spec Reference:** [`docs/43_toka_kernel_spec_v0.1.1.md`](../43_toka_kernel_spec_v0.1.1.md)  
> **Workflow:** _CodebaseResearch_ – condensed manual run

---

## 1  Executive Summary

The kernel implementation (`crates/toka-kernel`) **faithfully realises the v0.1.1 draft**: capability-guarded message dispatch, feature-gated opcode families, and deterministic in-memory state.  The crate is compact (<600 LOC), `#![forbid(unsafe_code)]`, and enjoys unit-tests for financial flows.  No immediate security red-flags have been identified.

Nonetheless, several architectural growth areas remain:

* **Scheduler & Drivers** surfaces are stubbed for v0.2 – early API sketches would de-risk the next milestone.
* **World-state concurrency** currently uses `RwLock + async` despite single-threaded semantics; this may hide latent contention once async schedulers land.
* **Feature creep** – optional `finance` & `user` families still live _inside_ the kernel crate; long-term we may want to externalise them as adapters to keep the nucleus minimal.

The report elaborates on strengths, improvement vectors, pitfalls, and candidates for removal.

---

## 2  Spec → Code Trace

| Spec Section | Implementation Evidence | Notes |
|--------------|-------------------------|-------|
| 4.1 Process Mgmt – opcodes | `Operation::{ScheduleAgentTask, SpawnSubAgent, EmitObservation}` in `toka-types` | Fully implemented & **always compiled**. |
| 4.2 Memory Mgmt | `WorldState` stores `HashMap`s; snapshotting delegated to CLI tooling | Deterministic hashing **not yet**; aligns with roadmap. |
| 4.3 Scheduling | FIFO, synchronous – hard-coded in `Kernel::submit` flow | Matches v0.1.1; async trait pending. |
| 4.4 Drivers & I/O | *Not implemented* (placeholder only) | High risk for v0.2 – no API boundary yet. |
| Feature-gated families | `#[cfg(feature = "finance"|"user")]` blocks around enums & handlers | Works; unit tests gated behind `finance`. |
| `UnsupportedOperation` error | Present in `KernelError` & used as default arm | ✅ |

---

## 3  What's Good

1. **Clear Layering** – `toka-types` defines opaque IDs & opcodes; `toka-kernel` consumes them without creating back-edges.
2. **Capability First** – All entry points validate a token via `toka-auth::TokenValidator` _before_ dispatch.
3. **Compile-time Pruning** – Cargo features strip entire opcode families, reducing attack surface of lightweight builds.
4. **No `unsafe`** – Kernel & deps compile with `#![forbid(unsafe_code)]`; `cargo geiger` reports 0 unsafe blocks.
5. **Deterministic Tests** – Finance unit tests simulate mint/transfer and assert state transitions, providing behavioural documentation.

---

## 4  Areas for Improvement

### 4.1 Scheduler Abstraction
* The synchronous FIFO loop is hard-coded; introducing a `Scheduler` trait _now_ (even if only one impl exists) isolates future async & priority strategies.
* Consider separating **task queue** from **executor** concepts to allow pluggable fairness algorithms.

### 4.2 Drivers API Definition
* The spec introduces `DriverCall { driver, payload }` but the codebase lacks any trait or enum for it.
* Propose a minimal `Driver` trait in a new `toka-driver-api` crate → `fn call(&self, bytes) -> Result<bytes>`; kernel holds a registry.

### 4.3 World-State Concurrency
* Kernel is _logically single-threaded_ yet wrapped in `Arc<RwLock<…>>`; once async tasks run in parallel this can become a hotspot.
* Evaluate **immutable snapshots + copy-on-write** or an **event-sourced log** to avoid coarse locks.

### 4.4 Feature Scope Creep
* Finance & user tables live inside the kernel → violates "minimal nucleus".  Suggest moving to `toka-kernel-finance` & `toka-kernel-user` add-on crates with their own opcode enums that wrap core `DriverCall`.

### 4.5 Error Granularity
* `KernelError::InvalidOperation(String)` hides structured causes; replace with variant per sub-domain or attach typed context via `thiserror` sources.

### 4.6 Testing Coverage
* Only `finance` tests exist; add tests for **task scheduling** and **unsupported opcode** rejection.
* Integrate `tarpaulin` in CI (target ≥ 70 % for kernel crate).

---

## 5  Pitfalls / Watch Out

| Area | Risk | Mitigation |
|------|------|-----------|
| **Driver sandboxing** | Untrusted driver code may break determinism or safety | Enforce WASI or capability-filtered host fns; validate payload sizes. |
| **`RwLock` poisoning** | Panic inside handler poisons lock, halting kernel | Use `parking_lot::RwLock` with poisoning disabled or wrap errors. |
| **64-bit balance types** | Future asset divisibility (decimals) not addressed | Migrate to fixed-point (e.g. `u128` with 1e6 scale) before mainnet. |
| **Enum Exhaustiveness** | Adding new opcodes is a _breaking_ change across network | Use `#[non_exhaustive]` or version field in `Message`. |
| **Capability Inflation** | `*` wildcard permission in tests must **never** appear in prod configs | Add deny-list lint in `toka-auth`.

---

## 6  What Could Be Removed

1. **Internal User Registry** – Identity likely belongs to higher-level toolkit; consider _removing_ the `user` family entirely from the kernel surface.
2. **Supply Tracking** – `WorldState::supply` duplicates ledger info; if assets are fungible & burn/mint events are canonical, supply can be derived off-chain.
3. **Alias strings in `AgentSpec`** – Non-essential for deterministic execution; could move to metadata driver.

---

## 7  Recommendations & Next Steps

| Priority | Action | Owner | Target |
|----------|--------|-------|--------|
| P0 | Draft `Scheduler` trait + default FIFO impl | Kernel WG | v0.1.2 |
| P0 | Define `Driver` trait in `toka-driver-api`; wire registry | Kernel WG | v0.2.0 |
| P1 | Extract `finance` & `user` into extension crates | Finance WG / Identity WG | v0.2.0 |
| P1 | Add coverage & failure-mode tests | QA | ongoing |
| P2 | Evaluate state snapshot strategy (COW / event-sourcing) | Arch WG | v0.3.0 |
| P2 | Harden error types & add `#[non_exhaustive]` on enums | Arch WG | v0.3.0 |

---

## 8  Appendix – Quick Metrics

| Metric | Value |
|--------|------:|
| Kernel LOC (`toka-kernel`) | ≈ 520 |
| Public Items | 18 |
| Unsafe Blocks | 0 |
| Unit-Tests | 2 (finance) |
| Feature Flags | `finance`, `user` |

> _Generated via manual inspection and `cargo +nightly geiger`, `tokei`, `cargo test --features=finance`._

---

© 2025 Toka Project — Apache-2.0 / MIT 