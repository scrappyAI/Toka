# Toka Credit Ledger System

> **⚠️ Work In Progress (v0.1.0)** - This is an initial implementation. Expect refinements and changes until fully working.

A secure, double-entry credit ledger system with hybrid online/offline capabilities, built in Rust.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                 CLEAN ARCHITECTURE                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─────────┐    ┌─────────────┐    ┌──────────┐     │
│  │   UI    │───▶│   RUNTIME   │───▶│  DOMAIN  │     │
│  │ (React) │    │(Client Biz) │    │(Entities)│     │
│  └─────────┘    └─────────────┘    └──────────┘     │
│      │                                   ▲          │
│      │              ┌──────────┐         │          │
│      └─────────────▶│  LEDGER  │─────────┘          │
│                     │(Server)  │                    │
│                     └──────────┘                    │
└─────────────────────────────────────────────────────┘
```

## 📦 Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| **`domain`** | Shared entities (Account, pricing models) | ✅ Implemented |
| **`ledger`** | Server-side authoritative double-entry accounting | ✅ Implemented |
| **`runtime`** | Client-side logic, sync protocols, offline capabilities | ✅ Implemented |
| **`ui`** | User interface components (React/web) | 🚧 Placeholder |

## 🚀 Features

- ✅ **Double-Entry Accounting**: All transactions maintain accounting integrity
- ✅ **Atomic Transactions**: All-or-nothing transaction commits with WAL support
- ✅ **Immutable Events**: Append-only event sourcing for audit trail
- ✅ **Pluggable Storage**: Memory, file, or custom storage backends
- ✅ **Offline-Capable**: Client-side ledger with server sync
- ✅ **Clean Architecture**: Proper separation of concerns
- 🚧 **Real-time Sync**: Server synchronization (coming soon)
- 🚧 **Web UI**: User interface components (coming soon)

## 📋 Quick Start

### Prerequisites
- Rust 1.70+ (edition 2021)
- Cargo workspace support

### Running Examples

```bash
# Basic ledger usage
cargo run --example basic_usage

# Atomic transactions demonstration  
cargo run --example atomic_transactions

# Domain pricing example
cargo run --example content_unlock_flow
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate tests
cargo test -p ledger
cargo test -p domain
cargo test -p runtime
```

## 🎯 Current Status (v0.1.0)

### ✅ Completed
- Core ledger infrastructure with WAL
- Double-entry accounting with validation
- Account entity abstraction
- Client-side user ledger with sync protocols
- Comprehensive examples and tests
- Clean modular architecture

### 🚧 In Progress
- API refinements and optimizations
- Error handling improvements
- More comprehensive testing
- Documentation enhancements

### 📅 Planned
- Web API endpoints
- Real-time synchronization
- React UI components
- Production deployment configurations

## 📚 Documentation

- [Ledger Architecture](crates/ledger/HYBRID_LEDGER.md)
- [Action Mapping](docs/ledger_action_mapping.md)
- [Domain Models](crates/domain/README.md)
- [Storage Systems](crates/ledger/README.md)

## 🤝 Development

This is a work-in-progress implementation. The system is functional but expect:
- API changes and refinements
- Performance optimizations
- Additional feature implementations
- Bug fixes and stability improvements

## 📄 License

MIT OR Apache-2.0 