# Toka Testing Guidelines

This document defines conventions for writing and running tests across the workspace.  Adhering to a consistent style keeps the codebase approachable for both humans and LLMs.

---

## 1. Folder layout

```
crates/
  toka-primitives/
    src/
    tests/           # 👈 integration tests live here
  toka-runtime/
    src/
    tests/           # integration tests for binary-level behaviour
workspace_root/
  tests/             # cross-crate integration tests (end-to-end)
```

* Unit tests that exercise private helpers can stay inline (`#[cfg(test)]` in source files).
* Prefer **integration tests** (`crates/<name>/tests/*.rs`) when testing the public API.

---

## 2. Naming conventions

* `*_tests.rs` for simple groupings (`currency_tests.rs`, `ids_tests.rs`).
* `mod integration` if you need multiple support functions inside one file.
* Use descriptive function names: `fn micro_usd_roundtrip()` (no `test_` prefix required — `cargo` adds it).

---

## 3. Feature flags

Many crates have optional features (`core`, `toolkit`, etc.). Your tests should:

1. Compile under **default features** (`cargo test -p <crate>`), **and**
2. Compile when optional heavy features are disabled, if meaningful:

```bash
cargo test -p toka-runtime --no-default-features
```

Gate test functions with `#[cfg(feature = "vault")]` etc. when they depend on a feature.

---

## 4. External resources

* Stash temp files in `std::env::temp_dir()` and clean them up with `tempfile`.  CI runs in parallel so avoid hard-coded paths.
* Never depend on real network access; use stubs or mock servers.

---

## 5. Running the full suite

* All features: `cargo nextest run --workspace --all-features` (CI does this).
* Lean runtime build: `cargo check -p toka-runtime --no-default-features`.

---

## 6. Coverage

CI uses `cargo tarpaulin` on Linux to track coverage.  Aim for at least **60 %** line coverage in each crate.

---

## 7. Adding new crates

1. Add an entry to `CRATES.md`.
2. Copy `tests/template.rs` (TBD) into `tests/` and flesh out.
3. Ensure the new crate is part of the workspace members list.

---

Happy testing! 🎉 