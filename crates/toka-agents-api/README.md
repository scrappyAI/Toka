# toka-agents-api

Toka Agents – API crate

This crate provides the **pure data types**, flags and minimal trait
contracts required to interact with agent implementations in the Toka
platform.  No heavy dependencies – only optional `serde`, `async` and
`uuid/semver` helpers behind feature flags – so the crate is suitable for
lightweight, embedded or `no_std` environments.

## Feature Flags
* `serde` *(default)* – derive `Serialize`/`Deserialize` for all public
  structs and enable JSON helpers.
* `async` *(default)* – include the async `Agent` trait extension that
  depends on `async-trait`, `anyhow` and the `toka-memory-api` crate.
* `std`  *(default)* – link against the Rust standard library.  Disable to
  compile for `no_std` targets.

## Layering
This crate lives in the **ApiLayer** as per the workspace rules.  It must
therefore remain free of heavyweight dependencies and implementation code.

License: MIT OR Apache-2.0
