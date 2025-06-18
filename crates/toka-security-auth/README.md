# Toka Auth Core

Lightweight capability-token primitives for Toka platform authentication.

## Overview

The Toka Auth Core crate provides the fundamental authentication primitives for the Toka platform, focusing on capability-based token systems. It offers a lightweight and secure approach to handling authentication and authorization within the Toka ecosystem.

## Features

- Capability token primitives
- Secure token generation and validation
- Cryptographic hash functions
- Hex encoding/decoding utilities

## Dependencies

### Core Dependencies
- serde: Serialization/deserialization
- sha2: Cryptographic hash functions
- hex: Hexadecimal encoding/decoding

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-auth-core = { version = "0.1.0" }
```

## Security Considerations

This crate is designed with security as a primary concern:
- Uses cryptographic hash functions for token generation
- Implements capability-based access control
- Provides secure token validation mechanisms

## Design Philosophy

The crate follows these principles:
- Minimal dependencies for security and reliability
- Focus on capability-based security model
- Lightweight implementation for edge deployments
- Clear separation of concerns

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 