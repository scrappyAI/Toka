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

## Clean-Up Roadmap (High Level)

1. **Phase-1 (✅ completed)** – Write this inventory file, no code changes.
2. **Phase-2 (✅ completed)** – Consolidate thin "-core" crates into their parent crates with `core` feature-gates, perform renames.
3. **Phase-3 (✅ completed)** – Added GitHub Action `.github/workflows/ci.yml` to build workspace (all features) and lean runtime, plus fmt/clippy/tests.
4. **Phase-4 (skipped)** – No crates had been published to crates.io, so deprecation shims & yanks are unnecessary.
5. **Phase-5 (✅ completed)** – Added `prelude.rs` and `SUMMARY.md` to aid LLM comprehension.
6. **Phase-6 (YOU ARE HERE)** – Introduced `toka` meta-crate + docs deployment planning.

Each phase ends with a `cargo check --workspace --all-features` gate. 