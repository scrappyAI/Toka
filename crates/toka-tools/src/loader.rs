//! Loader utilities – *stub* migrated from `toka-toolkit-core`.
//!
//! The full async loader implementation (supporting JSON-RPC, WASM, etc.)
//! remains to be ported.  For the v0.1 consolidation we only provide the
//! minimal scaffolding required by the manifest validator.

#![allow(dead_code)]

/// Runtime loader placeholder – does nothing for now.
pub struct Loader;

impl Loader {
    /// Create a new no-op loader.
    pub fn new() -> Self { Self }
}

impl Default for Loader {
    fn default() -> Self { Self::new() }
}