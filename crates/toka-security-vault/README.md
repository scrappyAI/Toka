# Toka Vault Core

Secure encrypted key/value store for Toka runtime.

## Overview

The Toka Vault Core crate provides a secure, encrypted key-value storage solution for the Toka platform. It is designed to ensure data confidentiality and integrity for sensitive information managed by Toka applications.

## Features

- Encrypted key/value storage
- Secure random number generation
- Base64 encoding/decoding utilities
- Integration with the Sled embedded database
- AES-GCM encryption for data security
- Serialization/deserialization support

## Dependencies

### Core Dependencies
- anyhow: Error handling
- serde: Serialization/deserialization
- serde_json: JSON support
- sled: Embedded database
- aes-gcm: AES-GCM encryption
- rand: Random number generation
- base64: Base64 encoding/decoding

### Dev Dependencies
- tokio: Async runtime for integration tests
- anyhow: Error handling (tests)
- rand: Random number generation (tests)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-vault-core = { version = "0.1.0" }
```

## Security Considerations

- All data is encrypted at rest using AES-GCM
- Random number generation is used for secure key management
- Designed for use in both edge and cloud environments

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 