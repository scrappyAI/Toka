# Toka Tools

> **Standard library of agent tools** for the [Toka](../../README.md) runtime.

This crate intentionally starts **empty** – except for a toy `echo` tool used
in tutorials and CI smoke-tests.  Every real tool must follow the
[Tool Development Guidelines](TOOL_DEVELOPMENT.md) and live behind its own
Cargo feature.

## Status

• **Incubating** – the API surface will change frequently.<br/>
• Only `echo` is shipped by default.<br/>
• New tools will land once the capability schema is finalised.

## Quick Start

```toml
[dependencies]
toka-tools = { version = "0.1", default-features = false, features = ["echo"] }
```

```rust
use toka_tools::{ToolRegistry, ToolParams};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let registry = ToolRegistry::new_empty();

// Register the demo echo tool
registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;

let mut args = std::collections::HashMap::new();
args.insert("message".into(), "hello".into());
let params = ToolParams { name: "echo".into(), args };

let res = registry.execute_tool("echo", &params).await?;
assert_eq!(res.output, "hello");
# Ok(())
# }
```

## Roadmap

1. Capability schema & manifest (Q1 2025)
2. Privileged tools – file I/O, HTTP fetch (Q2 2025)
3. Observability hooks & metrics (Q2 2025)

## License

Apache-2.0 OR MIT

© 2025 Toka Contributors 