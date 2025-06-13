# Toka Events

Ultra-lightweight in-memory event bus for the Toka platform. External backends (e.g., Redis, gRPC) are provided in separate crates.

## Overview

The Toka Events crate provides a simple, efficient, in-memory event bus for the Toka platform. It is designed for low-latency, high-throughput event delivery within a single process, with the option to extend to external backends as needed.

## Features

- In-memory event bus for fast, local event delivery
- Async event publishing and subscription
- Strongly-typed event payloads
- Serialization/deserialization support
- Error handling with `anyhow` and `thiserror`
- UUID-based event identification
- Extensible design for external backends (e.g., Redis, gRPC)

## Dependencies

### Core Dependencies
- anyhow: Error handling
- async-trait: Async trait support
- serde: Serialization/deserialization
- serde_json: JSON support
- thiserror: Error handling
- uuid: Unique identifier generation
- tokio: Async runtime (sync primitives)
- chrono: Date and time handling

### Dev Dependencies
- tokio: Async runtime for tests
- anyhow: Error handling (tests)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-events = { version = "0.1.0" }
```

## Design Philosophy

- Lightweight: Minimal overhead for in-memory event delivery
- Extensible: External backends can be implemented in separate crates
- Async-first: Designed for modern, async Rust applications

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 