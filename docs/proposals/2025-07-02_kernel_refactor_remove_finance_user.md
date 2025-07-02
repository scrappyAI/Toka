# Kernel Refactor Proposal – Agentic-Centric Core (Removal of Finance & User Primitives)

> Version: **0.2 – DRAFT** | Date: 2025-07-02 | Status: **Proposed**
> Author: Core Kernel Working Group (CKWG)
> Supersedes: [2025-07-01] Codebase Alignment Report v0.1.1
> Related Specs: `docs/42_toka_kernel_spec_v0.1.md`

---

## 1 Executive Summary

The current `toka-kernel` embeds *financial* and *user* opcode families that, while
useful for the v0.1 milestone, violate the long-term vision of a **minimal, agent-centric
kernel**. The CKWG therefore proposes a **hard removal**—*not deprecation*—of all finance
and user primitives from the core. These capabilities will be re-established as
**external adapters** living at the application layer or as pluggable runtime crates.

Key outcomes:

* Kernel surface is trimmed to **agent orchestration only** (`ScheduleAgentTask`,
  `SpawnSubAgent`, `EmitObservation`, plus platform housekeeping opcodes).
* `finance` and `user` feature flags disappear; corresponding code is deleted.
* New extension crates (`toka-kernel-finance`, `toka-kernel-user`) may re-introduce
  the removed behaviour without inflating the trusted computing base (TCB).
* Version bump to **v0.2.0** of the kernel crate to mark the semantic break.

This proposal details motivation, scope, required code changes, migration steps,
and an implementation roadmap.

---

## 2 Background & Motivation

The [v0.1 specification](../42_toka_kernel_spec_v0.1.md) and subsequent
[alignment report](../research/2025-07-01_kernel_spec_v0.1.1_code_report.md)
identified feature creep as an architectural risk (see §4.4 of the report).
The kernel's mandate is to serve as the **deterministic nucleus** for agentic
coordination, *not* as a general ledger or identity provider. Embedding
financial/User logic causes:

1. **Bloated TCB** – every additional opcode increases the critical attack
   surface audited on every release.
2. **Slowed Evolution** – domain-specific families (finance, roles, etc.) follow
   different lifecycles than the core; coupling forces lock-step versioning.
3. **Barrier to Specialisation** – lightweight deployments (e.g. embedded edge
   nodes) must currently compile unwanted functionality.
4. **Semantic Leakage** – user and asset semantics leak into the kernel API,
   complicating determinism and backwards compatibility.

A clean-slate, agent-only kernel solves these issues while enabling richer
domain features to evolve independently via adapters.

---

## 3 Scope of Change

### 3.1 In-Scope

* **Delete** finance opcodes: `TransferFunds`, `MintAsset`, `BurnAsset`.
* **Delete** user opcodes: `CreateUser`, `AssignRole`.
* **Remove** associated tables from `WorldState` (`balances`, `supply`, `users`).
* **Excise** finance/user capability checks and error variants.
* **Purge** feature flags `finance` and `user` from `Cargo.toml` and CI matrix.
* **Refactor** tests & docs accordingly.
* **Introduce** extension points (see §4 Architecture) enabling external crates
  to register additional opcode families.

### 3.2 Out-of-Scope

* Persistency refactor (event sourcing, storage engines) – handled separately.
* Scheduler abstraction – tracked by [roadmap item P0] in the alignment report.
* WASM driver integration – see `2025-06-28_loader_wasm_design.md`.

---

## 4 Proposed Architecture

### 4.1 Kernel Surface

After refactor, `toka_types::Operation` becomes:

```rust
pub enum Operation {
    // — agent primitives —
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    EmitObservation   { agent: EntityId, data: Vec<u8> },
    // future non-domain-specific opcodes may be added here
}
```

The kernel exposes a **registration API** whereby external crates contribute
additional opcode families via compile-time feature flags:

```rust
pub trait OpcodeHandler {
    fn dispatch(&self, op: &Operation, ctx: &mut WorldState) -> Result<Event, KernelError>;
}

pub fn register_handler(tag: &'static str, h: Box<dyn OpcodeHandler>);
```

This ensures the core remains stable while allowing adapters like
`finance::TransferFundsHandler` to plug in.

### 4.2 Extension Crates

1. `toka-kernel-finance`
   * Depends on `toka-kernel` (v0.2) and re-introduces finance opcodes & tables.
   * Published under its own release cadence.
2. `toka-kernel-user`
   * Provides identity & role management, potentially backed by external auth.

Both crates implement `OpcodeHandler` and register themselves on `crate::init()`.

---

## 5 API & Type Changes

| Item | Action |
|------|--------|
| `toka_types::Operation` | Remove finance & user variants |
| `KernelError` | Delete `InsufficientBalance` and `CapabilityDenied` variants tied to removed ops; keep generic `CapabilityDenied` for agent ops |
| `WorldState` | Remove `balances`, `supply`, `users` fields |
| Feature flags | Delete `finance`, `user`; default features become empty |
| Tests | Migrate remaining agent tests; move finance tests to new crate |

---

## 6 Migration Plan

1. **Code Pruning (Week 1)**
   * Remove flagged code paths, tables, error variants.
   * `cargo check --all-features` must pass with *no* finance/user flags.
2. **API Stabilisation (Week 2)**
   * Implement `OpcodeHandler` registry.
   * Refactor kernel internals to route via registry (even if only built-in
     agent handler exists initially).
3. **Extension Crate Bootstrap (Week 2-3)**
   * Scaffold `toka-kernel-finance` & port existing finance logic/tests.
   * Scaffold `toka-kernel-user` (optional placeholder).
4. **Documentation & Spec Update (Week 3)**
   * Author v0.2 spec draft reflecting new surface.
   * Update READMEs & architecture diagrams.
5. **Release & CI (Week 4)**
   * Bump crate version to `0.2.0`.
   * Ensure CI pipeline builds kernel ± finance extension combos.

---

## 7 Compatibility Considerations

* **Breaking Change** – network/state snapshots created with v0.1 are **not**
  compatible. A state migration tool may be developed separately if needed.
* No backwards-compatibility shim will be provided inside the kernel. Down-stream
  applications must adopt the finance/user extension crates or refactor.

---

## 8 Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| External crates diverge from core determinism guarantees | `OpcodeHandler` API enforces deterministic event signatures; CI runs integration tests across official extensions |
| Fragmented capability model across core and extensions | Define canonical `CapabilityScope` enum in `toka-auth` and share across crates |
| Increase in compile-time complexity due to dynamic registry | Registry uses compile-time `inventory` pattern; negligible runtime overhead |

---

## 9 Outstanding Questions

1. Should we publish extension crates under the same semver namespace (`toka-kernel-*`) or a separate group (`toka-extensions-*`)?
2. Do we enforce WASM sandboxing for finance logic immediately or defer?
3. How will event types emitted by extensions be namespaced to prevent clashes?

Feedback on these points is requested during proposal review.

---

## 10 Appendix A – Deleted Code Map

| File/Module | LOC | Action |
|-------------|----:|--------|
| `toka_types::Operation::TransferFunds` | 12 | Delete |
| `toka_types::Operation::MintAsset` | 8 | Delete |
| `toka_types::Operation::BurnAsset` | 8 | Delete |
| `toka_types::Operation::CreateUser` | 6 | Delete |
| `toka_types::Operation::AssignRole` | 6 | Delete |
| `world_state::balances` | 40 | Delete |
| `world_state::supply` | 15 | Delete |
| `world_state::users` | 32 | Delete |
| `kernel/handlers/finance.rs` | 120 | Move to `toka-kernel-finance` |
| `kernel/handlers/user.rs` | 80 | Move to `toka-kernel-user` |

(The actual diff will be tracked in PR implementation.)

---

© 2025 Toka Project – Apache-2.0 / MIT