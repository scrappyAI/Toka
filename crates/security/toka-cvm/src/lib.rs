//! **toka-cvm** – Capability Validation Module
//!
//! Host-side helper for verifying **capability tokens** inside untrusted
//! WebAssembly guests executed via Wasmtime.  The crate is currently a
//! skeleton reserving the namespace and API surface described in
//! `docs/41_capability_tokens_architecture.md` §5.  No public functions are
//! exported yet – the first MVP will expose a minimal
//! `validate(token: &str) -> anyhow::Result<()>` FFI boundary.
//!
//! This early version exists primarily so that downstream crates can gate
//! experimental features behind a Cargo feature flag (`cvm`) without
//! circular dependencies.
//!
//! **Status**: _Placeholder – API subject to change._
#![forbid(unsafe_code)]

/// Placeholder function to silence `unused` warnings until the crate gains
/// real functionality.
#[allow(dead_code)]
fn _placeholder() {}