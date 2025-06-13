# Toka Toolkit

A comprehensive toolkit for the Toka platform, providing utilities, helpers, and integrations for building robust Toka applications.

## Overview

The Toka Toolkit crate offers a wide range of utilities and helpers to streamline development within the Toka ecosystem. It builds on the abstractions provided by `toka-toolkit-core` and integrates with other Toka components for a seamless developer experience.

## Features

- File system utilities
- Serialization/deserialization (JSON, CBOR, CSV)
- Random number generation
- Cryptographic utilities (hashing, encoding)
- Date and time handling
- UUID generation
- Async utilities and helpers
- Directory management
- Tracing and logging integration
- Type tagging support
- Integration with `toka-toolkit-core`

## Dependencies

### Core Dependencies
- anyhow: Error handling
- async-trait: Async trait support
- tokio: Async runtime (with multiple features)
- serde: Serialization/deserialization
- serde_json: JSON support
- serde_cbor: CBOR support
- csv: CSV parsing
- chrono: Date and time handling
- uuid: Unique identifier generation
- futures: Future utilities
- rand: Random number generation
- hex: Hexadecimal encoding/decoding
- sha2: Cryptographic hash functions
- base64: Base64 encoding/decoding
- directories: Directory management
- tracing: Logging and instrumentation
- typetag: Type tagging
- toka-toolkit-core: Core toolkit abstractions

### Dev Dependencies
- tokio: Async runtime for tests
- anyhow: Error handling (tests)
- rand: Random number generation (tests)
- serde_cbor: CBOR support (tests)
- tempfile: Temporary file handling (tests)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-toolkit = { version = "0.1.0" }
```

## Design Philosophy

- Comprehensive: Provides a wide range of utilities for Toka development
- Modular: Integrates with core abstractions and other Toka crates
- Async-first: Designed for modern, async Rust applications

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 