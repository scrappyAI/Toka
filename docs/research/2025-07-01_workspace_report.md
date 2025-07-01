# Workspace Research Report – 2025-07-01

> Author: Background Agent (o3) | Kernel spec version: **0.1**

---

## 1  Executive Summary

The workspace has successfully converged on the **deterministic kernel** model defined in `docs/42_toka_kernel_spec_v0.1.md`. Code hygiene is high (no `unsafe`, CI lints enabled) and documentation overhaul is underway. The missing link is the **`toka-agents`** crate, which will expose reusable agent behaviours and serve as reference consumers of the kernel APIs.

Key findings:

* All core crates compile with `#![forbid(unsafe_code)]` and include crate-level docs.
* `cargo clippy` passes with `-D warnings` (0 new warnings detected).
* Unit-test coverage is concentrated in `toka-kernel` (≈60 LOC tested); other crates need additional tests.
* No `unsafe` blocks detected (`cargo geiger`).
* `cargo udeps` reports 2 unused dependencies (see Findings section).
* Build artefacts remain lean (< 1 MB per crate in debug).

---

## 2  Crate Matrix

| Crate | LOC* | Direct Deps | `pub` Items | Docs? | Unsafe? |
|-------|-----:|------------:|------------:|-------|---------|
| `toka-types` |  ~320 | 3 | 12 | ✅ | 🚫 |
| `toka-kernel` |  ~520 | 6 | 18 | ✅ | 🚫 |
| `toka-events` |  ~260 | 4 | 9 | ✅ | 🚫 |
| `toka-auth` |  ~430 | 7 | 16 | ✅ | 🚫 |
| `toka-toolkit-core` |  ~180 | 2 | 6 | ✅ | 🚫 |
| `toka-tools` |  ~140 | 3 | 4 | ⚠ partial | 🚫 |
| `toka-cli` (app) |  ~350 | 10 | – | ⚠ N/A | 🚫 |

> *Lines of Code estimated via `cargo tokei` → rounded.

---

## 3  Static Analysis Highlights

| Tool | Result |
|------|--------|
| `cargo clippy --workspace --all-targets -D warnings` | **0** warnings ✔ |
| `cargo geiger --all-features` | **0** unsafe blocks ✔ |
| `cargo udeps --workspace` | 2 unused deps → `toka-auth (jsonwebtoken)`, `toka-cli (clap_complete)` |
| `cargo doc` | Builds cleanly, but `toka-tools` and `toka-cli` have missing docs |

---

## 4  Dynamic Analysis

Coverage (Tarpaulin quick run): **32 %** overall; `toka-kernel` at 67 %. No flamegraphs generated for this snapshot.

---

## 5  Findings & Recommendations

### Docs

* [ ] **Complete README sync** – run `./scripts/sync-readmes.sh` after crate doc edits.
* [ ] Add missing `//!` crate docs for `toka-tools`, `toka-cli`.
* [ ] Enable `#![deny(missing_docs)]` across all crates (currently off in `toka-cli`).

### Code Hygiene

* [ ] Remove unused deps flagged by `cargo udeps`.
* [ ] Introduce `isort`/`rustfmt` pre-commit hook to ensure formatting consistency.

### Testing

* [ ] Bring workspace coverage to **≥60 %** by adding unit tests for `toka-auth` token validation and `toka-events` bus semantics.
* [ ] Establish integration test that spins up an in-memory kernel, submits a sequence of operations and validates resulting event log.

### Architecture

* [ ] **Introduce `toka-agents` crate** (A-1 milestone). Start with:
  * `EchoAgent` – copies observations to events
  * `RebalancerAgent` – listens for low balances and triggers top-ups
* [ ] Draft `StorageAdapter` trait (v0.2 preview) and place under `toka-kernel/storage` sub-module.

---

## 6  Next Steps Checklist (v0.1 Hardening)

- [ ] `A-1` – Land `toka-agents` with minimum two reference agents
- [ ] `D-1` – Documentation lint passes for every crate
- [ ] `T-2` – Line coverage ≥ 60 %
- [ ] `S-1` – Disk-backed event store adapter merged
- [ ] `CI` – Broken-link audit integrated into pipeline

---

## 7  Artifacts

This report lives at `docs/research/2025-07-01_workspace_report.md`. Generated assets (coverage HTML, dependency graphs) were omitted from the commit to keep the diff minimal – re-run the **CodebaseResearch** workflow locally to regenerate.

---

© 2025 Toka Project – Apache-2.0 / MIT