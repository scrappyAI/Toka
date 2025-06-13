# Toka Runtime

A flexible runtime environment for the Toka platform, providing core execution capabilities and optional integrations with other Toka components.

## Overview

The Toka Runtime crate provides the foundational runtime environment for executing Toka applications. It offers a modular design that allows for optional integration with various Toka components through feature flags.

## Features

The crate ships with minimal functionality by default. Additional capabilities can be enabled through feature flags:

### Core Features
- Async runtime support via Tokio
- Tracing and logging infrastructure
- Serialization/deserialization utilities
- UUID generation and handling
- Future utilities

### Optional Features
- `toolkit`: Enables full toolkit support including:
  - CLI interface via clap
  - Type tagging support
  - Integration with toka-agents-core
  - Integration with toka-toolkit-core
- `vault`: Enables secure local vault storage capabilities
- `auth`: Enables authentication and capability token utilities

## Dependencies

### Core Dependencies
- anyhow: Error handling
- async-trait: Async trait support
- tokio: Async runtime
- serde: Serialization/deserialization
- tracing: Logging and instrumentation
- uuid: Unique identifier generation
- futures: Future utilities

### Optional Dependencies
- clap: CLI argument parsing
- typetag: Type tagging
- Various Toka crates (toka-toolkit, toka-agents-core, etc.)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-runtime = { version = "0.1.0", features = ["toolkit", "vault"] }
```

## Development

The crate includes test utilities and temporary file handling for development purposes.

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 