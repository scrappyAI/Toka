//! Toka Runtime – orchestrates agents, tools, event bus, and vault
//!
//! This crate wires together the core subsystems (`toka-agents`, `toka-tools`,
//! `toka-bus`, `toka-events`, etc.) into a cohesive, async runtime.
//!
//! # Features
//!
//! | Feature | Purpose | Additional crates |
//! |---------|---------|-------------------|
//! | `toolkit` *(opt)* | Enables [`ToolRegistry`](crate::tools) & default tools | `toka-toolkit-core`, `toka-tools` |
//! | `auth` *(opt)*    | Capability-token validation & secret rotation | `toka-security-auth`, `jsonwebtoken` |
//! | `vault` *(opt)*   | Embed the canonical event store | `toka-events` + `sled` |
//!
//! ## Quick-Start
//! ```rust,no_run
//! # use toka_runtime::runtime::{Runtime, RuntimeConfig};
//! # async fn run() -> anyhow::Result<()> {
//! let rt = Runtime::new(RuntimeConfig::default()).await?;
//! rt.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Built with the ergonomics of LLM-driven agent development in mind.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use toka_bus as events;
#[cfg(feature = "auth")]
pub use toka_security_auth as auth;
#[cfg(all(feature = "toolkit", feature = "vault"))]
#[doc(hidden)]
pub mod runtime;
#[cfg(feature = "toolkit")]
#[doc(hidden)]
pub mod tools;
#[cfg(feature = "auth")]
pub mod security;

#[cfg(feature = "toolkit")]
pub use toka_agents as agents;

// Expose persistence API from toka-events
#[cfg(feature = "vault")]
pub use toka_events as event_store;

#[cfg(feature = "vault")]
pub use toka_events as vault;
