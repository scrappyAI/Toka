// This module is only built when the `toolkit` feature flag is enabled.
// It re-exports everything from the standalone `toka-toolkit` crate so that
// existing `crate::tools::*` paths continue to compile when the toolkit is
// pulled in as an optional dependency.

#![cfg(feature = "toolkit")]

pub use toka_tools::*;
pub use toka_tools_api::{Tool, ToolMetadata, ToolParams, ToolRegistry, ToolResult};
