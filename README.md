# Toka: A Modular Agent and Tool Runtime

Toka is an experimental, modular runtime for building and managing AI agents and their associated tools. It's designed to be a flexible foundation for a wide range of agent-based systems.

This project is currently in its early stages. The goal is to build a robust, open-source platform for agent development in a collaborative, community-driven way.

## Core Principles

- **Modularity:** Toka is built as a collection of small, independent crates. This allows you to pick and choose the components you need, keeping your application lean.
- **Security:** Security is a top priority. The `toka-security-auth` and `toka-security-vault` crates provide a solid foundation for securing your agents and their data.
- **Extensibility:** The toolkit-based architecture makes it easy to add new tools and capabilities to your agents.

## Getting Started

To start using the full Toka platform, add the following to your `Cargo.toml`:

```toml
[dependencies]
toka = "0.1.0"
```

If you only need specific functionality, you can disable the default features and select the ones you need:

```toml
[dependencies]
toka = { version = "0.1", default-features = false, features = ["auth"] }
```

## Crate Structure

The Toka workspace is organized into several crates:

- `toka`: The main meta-crate that provides a batteries-included experience.
- `toka-core`: Defines the core data models, business logic, and domain rules.
- `toka-agents`: Provides the core agent implementation.
- `toka-runtime`: The agent and tool runtime.
- `toka-toolkit`: A collection of standard tools for agents.
- `toka-security-auth`: Primitives for capability-based authentication.
- `toka-security-vault`: A secure vault for managing agent secrets.
- `toka-events`: A lightweight event system for inter-component communication.
- `toka-primitives`: Basic data types used throughout the platform.
- `toka-cli`: A command-line interface for interacting with the Toka runtime.

## Contributing

This is an open-source project, and contributions are welcome! Please feel free to open an issue or submit a pull request.

For more details on how to contribute, please see the (soon to be created) `CONTRIBUTING.md` file.

## License

This project is dual-licensed under the MIT and Apache 2.0 licenses. See the `LICENSE-MIT` and `LICENSE-APACHE` files for more details. 