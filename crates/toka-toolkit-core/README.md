# Toka Toolkit Core

Tool trait and registry abstractions for the Toka platform.

## Overview

This crate provides the foundational abstractions for the Toka toolkit system. It defines the core traits and interfaces that tools must implement, along with the registry system for managing and discovering tools.

## Features

- Core tool trait definitions
- Tool registry interfaces
- Tool metadata and discovery
- Lightweight, dependency-free design
- Extensible tool architecture
- Async tool execution support

## Dependencies

### Core Dependencies
- anyhow: Error handling
- async-trait: Async trait support
- serde: Serialization/deserialization
- tokio: Async runtime (sync primitives)
- tracing: Logging and instrumentation

### Dev Dependencies
- tokio: Async runtime for tests
- anyhow: Error handling (tests)
- async-trait: Async trait support (tests)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-toolkit-core = "0.1"
```

### Example

```rust
use toka_toolkit_core::{Tool, ToolRegistry, ToolMetadata};

// Define a custom tool
struct MyTool;

#[async_trait::async_trait]
impl Tool for MyTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::new("my_tool", "A custom tool")
    }
    
    async fn execute(&self, input: &str) -> anyhow::Result<String> {
        // Tool implementation
        Ok("Tool executed successfully".to_string())
    }
}

// Register the tool
let mut registry = ToolRegistry::new();
registry.register(Box::new(MyTool));

// Discover and use tools
let tool = registry.get("my_tool")?;
let result = tool.execute("input").await?;
```

## Design Philosophy

- **Abstraction**: Platform-agnostic tool interfaces
- **Flexibility**: Support for any type of tool implementation
- **Discovery**: Built-in tool discovery and metadata system
- **Performance**: Async-first design for efficient execution

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 