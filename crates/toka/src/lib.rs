#![warn(missing_docs)]
//! # `toka` – Meta Crate
//!
//! Batteries-included entry-point that re-exports the **most common types** from
//! the Toka ecosystem (agents, event store, auth, toolkit …).  Think of it as
//! the *standard library* for quickly prototyping on the platform.
//!
//! If you want fine-grained control over dependencies simply depend on the
//! individual crates (`toka-agents`, `toka-events`, …) and disable default
//! features.
//!
//! ## Feature Flags
//! | Flag       | Re-exported Sub-system | Extra crates pulled in |
//! |------------|------------------------|------------------------|
//! | **default** | `agents`, `auth`, `events`, `toolkit` | see below |
//! | `agents`   | `toka-agents`        | `tokio`, `anyhow`, … |
//! | `auth`     | `toka-capability`    | `jsonwebtoken` |
//! | `events`   | `toka-events`        | `sled`, `blake3`, … |
//! | `toolkit`  | `toka-toolkit-core` + `toka-tools` | `wasmtime` (optional) |
//!
//! Example with _only_ the auth helpers enabled:
//! ```toml
//! [dependencies]
//! toka = { version = "0.1", default-features = false, features = ["auth"] }
//! ```
//!
//! ## Quick Example
//! ```rust
//! use toka::prelude::*;
//!
//! // Create a signed capability token (auth feature)
//! let token = CapabilityToken::new(
//!     "alice",          // subject
//!     "vault1",         // vault id
//!     vec!["read".into()],
//!     "my-32-byte-secret",
//!     3600,              // 1 h TTL
//! ).unwrap();
//! assert!(token.is_valid("my-32-byte-secret"));
//! ```
//!
//! ---
//! This crate is `#![forbid(unsafe_code)]` and merely re-exports – it contains
//! *no* runtime logic.

// -----------------------------------------------------------------------------
// Prelude – glob-import to grab common symbols in one line
// -----------------------------------------------------------------------------
#[allow(unused_imports)]
pub mod prelude {
    #[cfg(feature = "agents")]
    pub use toka_agents::prelude::*;
    #[cfg(feature = "auth")]
    pub use toka_capability::prelude::*;
    #[cfg(feature = "events")]
    pub use toka_events::prelude::*;
    // Future: add toolkit prelude when available.
}

// -----------------------------------------------------------------------------
// Re-export entire crates behind feature flags so users can access full APIs.
// -----------------------------------------------------------------------------
#[cfg(feature = "agents")]
pub use toka_agents as agents;
#[cfg(feature = "auth")]
pub use toka_capability as auth;
#[cfg(feature = "events")]
pub use toka_events as events;
#[cfg(feature = "toolkit")]
pub use toka_toolkit_core::{Tool, ToolRegistry, ToolParams, ToolResult};
pub use toka_tools as toolkit;
#[cfg(feature = "toolkit")]
pub use toka_toolkit_core as toolkit_core;
