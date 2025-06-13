# Toka Toolkit Core

Minimal tool trait and registry abstractions for Toka.

## Overview

The Toka Toolkit Core crate provides the foundational abstractions for tool traits and registries within the Toka platform. It is designed to be lightweight and extensible, enabling the creation and management of tools in a modular fashion.

## Features

- Minimal tool trait definitions
- Tool registry abstractions
- Async trait support for tool operations
- Serialization/deserialization support
- Tracing and logging integration

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
toka-toolkit-core = { version = "0.1.0" }
```

## Design Philosophy

- Minimalism: Only the essential abstractions are provided
- Extensibility: Designed to be extended for custom tool implementations
- Async-first: All tool operations are async-ready

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 