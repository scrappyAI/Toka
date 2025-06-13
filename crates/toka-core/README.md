# Toka Core

Core business logic and domain rules for the Toka platform, including currency, resource, and identifier definitions.

## Overview

This crate provides the fundamental building blocks and domain models for the Toka platform. It is designed with a modular feature system that allows downstream consumers to opt-in to specific functionality they need.

## Features

The crate ships with no functionality enabled by default. Consumers must explicitly opt-in to the domain areas they need:

### Fundamental Building Blocks
- `ids`: Type-safe identifiers
- `currency`: Micro-USD and helper functions

### Higher-level Domain Areas
- `models`: LLM model metadata and pricing
- `resources`: Generic resource descriptors
- `vaults`: Vault metadata structs
- `economics`: Fee schedules and platform economics
- `products`: User-facing SKUs such as credit packs
- `pricing`: Pricing policies and service facade

### Tooling
- `schema-gen`: JSON Schema generation support

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-core = { version = "0.1.0", features = ["ids", "currency"] }
```

## Dependencies

- serde: Serialization/deserialization
- rust_decimal: Decimal number handling
- uuid: Unique identifier generation
- chrono: Date and time handling
- thiserror: Error handling
- schemars: JSON Schema generation (optional)
- anyhow: Error handling utilities

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 