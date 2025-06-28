//! # Toka Tools
//!
//! **Standard library of agent tools** bundled with the Toka platform.
//! The crate ships a thin [`ToolRegistry`] wrapper plus a growing set of
//! ready-made tools (each one behind its own cargo **feature flag** so you
//! only compile what you need).
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

pub mod tools;

// Re-export the important types so downstream code can simply `use toka_toolkit::{Tool, ToolRegistry}`
pub use crate::tools::{Tool, ToolRegistry};
