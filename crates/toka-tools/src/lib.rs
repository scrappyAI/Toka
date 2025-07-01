#![forbid(unsafe_code)]
#![warn(missing_docs)]
//!
//! **toka-tools** – Standard library of _agent tools_ for **Toka OS**.
//!
//! The crate complements the deterministic [`toka-kernel`](../../toka-kernel) by providing
//! reusable building blocks that agents can invoke _at arm's length_.  Tools **never** bypass the
//! kernel's capability checks – they are regular Rust (or WASM) functions that prepare
//! [`Operation`](toka_types::Operation)s and submit authenticated [`Message`](toka_types::Message)s.
//!
//! 📜 For the canonical opcode semantics see [`docs/42_toka_kernel_spec_v0.1.md`](../../../docs/42_toka_kernel_spec_v0.1.md).
//!
//! _Design goals_
//! * **Modularity** – every tool lives behind its own Cargo feature flag.
//! * **Determinism** – tools must be side-effect free unless explicitly documented.
//! * **Minimal deps** – keep the dependency graph shallow so agents can vendor-select.
//!
//! ## Feature Flags
//! | Feature        | Tool / Capability | Extra Deps |
//! |----------------|-------------------|------------|
//! | `echo` *(default via `minimal`)* | Simple *echo* tool used in tutorials | – |
//! | `minimal`      | Alias that enables only the lightweight demo tools | – |
//! | _future_       | More heavy-weight tools will land behind dedicated flags | varies |
//!
//! Enable just the *echo* tool:
//! ```toml
//! [dependencies]
//! toka-tools = { version = "0.1", default-features = false, features = ["echo"] }
//! ```
//!
//! ## Quick Example
//! ```rust
//! use std::sync::Arc;
//! use toka_tools::{ToolRegistry, ToolParams};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Empty registry – you pick which tools to install.
//!     let registry = ToolRegistry::new_empty();
//!
//!     // Register the demo echo tool (behind `echo` feature flag).
//!     registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
//!
//!     let mut args = std::collections::HashMap::new();
//!     args.insert("message".into(), "hello".into());
//!     let res = registry.execute_tool("echo", &ToolParams { name: "echo".into(), args }).await?;
//!
//!     assert_eq!(res.output, "hello");
//!     Ok(())
//! }
//! ```
//!
//! ---
//! This crate forbids `unsafe` and keeps its public API intentionally small
//! so that downstream workspaces can vendor or fork individual tools without
//! pulling the entire Toka dependency graph.
//!
//! ⚠️ **Experimental crate** – public APIs are subject to change without notice.  Expect **breaking
//! changes** until the project reaches `v1.0` maturity.
//!
//! _Note_: The former **`toka-toolkit-core`** crate has been fully merged into *toka-tools* as of
//! July 2025.  All core abstractions now live under [`crate::core`]. Downstream crates should
//! simply depend on `toka-tools` instead of the removed legacy crate.

pub mod core;

pub mod tools;

// Re-export the important types so downstream code can simply `use toka_tools::{Tool, ToolRegistry}`
pub use crate::core::{Tool, ToolMetadata, ToolParams, ToolRegistry, ToolResult};
