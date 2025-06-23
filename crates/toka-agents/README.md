# Toka Agents Core

Default agent implementations for the Toka platform.

## Overview

The Toka Agents Core crate provides the foundational agent implementations and interfaces for the Toka platform. It serves as the base layer for building and extending agent functionality within the Toka ecosystem.

## Features

- Core agent interfaces and traits
- Default agent implementations
- Async execution support
- Serialization/deserialization capabilities
- Tracing and logging integration
- UUID-based agent identification

## Dependencies

### Core Dependencies
- anyhow: Error handling
- async-trait: Async trait support
- serde: Serialization/deserialization
- tokio: Async runtime
- tracing: Logging and instrumentation
- uuid: Unique identifier generation

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-agents = { version = "0.1.0" }
```

## Design Philosophy

The crate is designed with the following principles in mind:
- Minimal dependencies for core functionality
- Async-first approach for efficient execution
- Strong typing and error handling
- Extensible architecture for custom agent implementations

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 