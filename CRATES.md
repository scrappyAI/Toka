# Toka Workspace – Crate Inventory

> This file is the single-source-of-truth for crate purpose & future rename mapping.  Update it **immediately** when crates are added/removed/renamed so LLMs can build an accurate mental model without scanning the entire tree.

| Current Crate           | Proposed Name (Phase-2+) | Rule-of-Thumb Reason* | One-line Purpose |
|-------------------------|---------------------------|-----------------------|------------------|
| `toka-primitives`       | (keep)                    | ① no_std primitives  | Fundamental, dependency-free types (IDs, currency, etc.). |
| `toka-core`             | `toka-domain`             | ③ separate lifecycle | Higher-level domain logic built on primitives. |
| `toka-events`           | (keep)                    | ① no_std             | Ultra-light in-memory event bus. |
| `toka-security-auth`    | (renamed – done)          | ① + ③                | Capability-token primitives, crypto free. |
| `toka-security-vault`   | (renamed – done)          | ② heavy deps         | Encrypted key/value vault built on sled + aes-gcm. |
| `toka-agents`           | (renamed – done)          | ② optional deps      | Default agent implementations for the runtime. |
| `toka-toolkit-core`     | (keep)                    | ① light, reusable    | Tool trait & registry abstractions (no heavy deps). |
| `toka-toolkit`          | (keep)                    | ② heavy deps         | Batteries-included tool implementations (CSV, CBOR, etc.). |
| `toka-runtime`          | (keep)                    | ② heavy deps         | Async host for agents, event bus, toolkit. |
| `toka`                  | (new meta-crate)          | – aggregate crate    | Re-exports common preludes for easy onboarding. |

*Rule-of-Thumb Keys*
① Usable from `no_std` / lean targets  
② Contains heavy or optional dependencies  
③ Needs an independent release cadence

---

## Workspace Evolution

The initial crate-layout cleanup (Phases 1-6) is **finished** and reflected in this file.  Future structural work will be tracked in Git commit messages rather than an ever-growing list here.

Completed milestones:
1. Inventory written (Phase-1)
2. Thin *-core* crates consolidated & renamed (Phase-2)
3. CI workflow added (fmt, clippy, builds, tests) (Phase-3)
4. Deprecation shims unnecessary (Phase-4 skipped)
5. Preludes + SUMMARY.md to aid LLM navigation (Phase-5)
6. `toka` meta-crate introduced (Phase-6)

---

## Testing Roadmap

Testing work is tracked separately to keep context focussed.  The high-level phases are:

| Test Phase | Goal | Status |
|-----------|------|--------|
| **T-1** | Seed each crate with at least one integration test template | ✅ complete |
| **T-2** | Achieve 60 %+ line coverage via Tarpaulin | 🔄 in progress |
| **T-3** | Cross-crate integration tests (runtime end-to-end) | ⬜ pending |

Detailed guidelines live in `TESTS.md`.

---

*Each structural or testing milestone should end with `cargo check --workspace --all-features` running cleanly.*