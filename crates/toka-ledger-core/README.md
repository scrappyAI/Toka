# Toka Ledger Core

Core ledger functionality and types for the Toka platform.

## Overview

This crate provides the foundational types and interfaces for the Toka ledger system. It defines the core data structures and operations needed for tracking financial transactions and agent activities.

## Features

- Core ledger data structures
- Transaction types and metadata
- Ledger entry interfaces
- Hash-based integrity verification
- Serialization/deserialization support
- Lightweight, dependency-free design

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-ledger-core = "0.1.0"
```

### Example

```rust
use toka_ledger_core::{LedgerEntry, Transaction, Hash};

// Create a ledger entry
let entry = LedgerEntry::new(
    "agent.payment",
    transaction_data,
    timestamp
);

// Verify entry integrity
let hash = entry.compute_hash();
assert!(entry.verify_hash(&hash));
```

## Design Philosophy

- **Integrity**: Hash-based verification for data integrity
- **Immutability**: Ledger entries are immutable once created
- **Extensibility**: Support for custom transaction types
- **Performance**: Efficient hashing and serialization

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 