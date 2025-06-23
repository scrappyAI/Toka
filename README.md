# Toka: A Modular Agent and Tool Runtime

Toka is an experimental, modular runtime for building and managing AI agents and their associated tools. It's designed to be a flexible foundation for a wide range of agent-based systems with a focus on security, extensibility, and clean architecture.

This project is currently in active development. The goal is to build a robust, open-source platform for agent development in a collaborative, community-driven way.

## Core Principles

- **Modularity:** Toka is built as a collection of small, independent crates. This allows you to pick and choose the components you need, keeping your application lean.
- **Security First:** Security is a top priority with capability-based authentication and encrypted vault storage.
- **Extensibility:** The toolkit-based architecture makes it easy to add new tools and capabilities to your agents.
- **Clean Architecture:** Clear separation of concerns with domain-driven design principles.

## Quick Start

To start using the full Toka platform, add the following to your `Cargo.toml`:

```toml
[dependencies]
toka = "0.1.0"
```

This gives you a batteries-included experience with agents, authentication, vault storage, and toolkit support.

If you only need specific functionality, you can disable the default features and select the ones you need:

```toml
[dependencies]
toka = { version = "0.1", default-features = false, features = ["auth"] }
```

## Crate Structure

The Toka workspace is organized into several focused crates:

### Core Foundation
- `toka-primitives`: Fundamental, dependency-free types (IDs, currency, etc.)
- `toka-core`: Higher-level domain logic and business rules
- `toka-events-core`: Event system primitives and types

### Runtime & Execution
- `toka-runtime`: Async host for agents, event bus, and toolkit integration
- `toka-agents`: Default agent implementations and interfaces
- `toka-cli`: Command-line interface for interacting with the Toka runtime

### Event System
- `toka-bus-memory`: In-memory event bus implementation
- `toka-bus-persist`: Persistent event bus implementation

### Security & Storage
- `toka-security-auth`: Capability-based authentication primitives
- `toka-secrets`: Encrypted key/value vault for secure storage
- `toka-storage`: Generic storage abstractions

### Toolkit & Tools
- `toka-toolkit-core`: Tool trait and registry abstractions
- `toka-toolkit`: Batteries-included tool implementations

### Ledger System
- `toka-ledger-core`: Core ledger functionality
- `toka-ledger-agents`: Agent-specific ledger operations
- `toka-ledger-finance`: Financial ledger operations

### Specialized
- `smart-embedder`: Smart embedding generation utilities

### Meta Crate
- `toka`: Convenience meta-crate that re-exports commonly used components

## Development Status

The project is currently in active development with the following milestones completed:

- ✅ Core crate structure and organization
- ✅ Security primitives and vault implementation
- ✅ Agent framework and runtime
- ✅ Event bus system (memory and persistent)
- ✅ Toolkit architecture
- ✅ Ledger system foundation
- 🔄 Testing and documentation improvements
- ⬜ Production readiness and performance optimization

## Contributing

This is an open-source project, and contributions are welcome! Please feel free to open an issue or submit a pull request.

For more details on how to contribute, please see the `CONTRIBUTING.md` file.

## License

This project is dual-licensed under the MIT and Apache 2.0 licenses. See the `LICENSE-MIT` and `LICENSE-APACHE` files for more details. 