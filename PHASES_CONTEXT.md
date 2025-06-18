# Toka Architecture Phases – Working Context

This file tracks **incremental architectural work** that we are performing in the repository so that contributors & LLMs stay aligned with intent.  Keep it short-lived and prune once a phase is finished.

---

## Phase-1 — Unified Event System (Current)
* Goal: delete competing EventBus implementations and depend on a **single canonical crate** (`toka-events`).
* Work:
  1. Migrate rich event types from `toka-runtime/src/events` into `toka-events/src/rich.rs`.
  2. Re-export `EventBus`, `EventType`, `AgentEvent`, … at the crate root.
  3. Remove local stubs in `toka-agents` and make agents depend on the unified bus.
  4. Alias `pub use toka_events as events;` inside `toka-runtime` to keep paths stable.
* Status: ✅ code compiles, tests untouched (run `cargo check --workspace`).

## Phase-2 — Vault-backed Agent Memory
* Introduce `MemoryAdapter` trait in `toka-security-vault`.
* Agents implement `save_state / load_state`.
* Runtime persist and restore agents via the adapter.

## Phase-3 — Tool ↔ Agent bridge
* Add `invoke_tool` helper on agents.
* Emit `ToolEvent` via EventBus.

## Phase-4 — CLI Skeleton (`toka-cli`)
* Basic commands: `agent new/list`, `agent observe`, `tool list/run`, `vault get/put`.

## Guidelines
* Follow testing conventions in `TESTS.md` (integration tests per crate, coverage ≥ 60 %).
* Reflect crate-naming rules in `CRATES.md` when adding/renaming.
* Keep each phase small; end with `cargo check --workspace --all-features` green.

---

_Last updated: 2025-06-18_ 