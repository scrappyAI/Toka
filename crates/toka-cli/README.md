# Toka CLI

Command-line interface for the Toka platform.

## Overview

This crate provides a comprehensive command-line interface for interacting with the Toka runtime. It allows you to manage agents, execute tools, interact with the secure vault, and monitor the system.

## Features

- Agent lifecycle management (create, list, observe)
- Tool discovery and execution
- Secure vault operations
- System monitoring and diagnostics
- Event stream observation
- Configuration management

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-cli = "0.1.0"
```

Or install the binary:

```sh
cargo install toka-cli
```

## Usage

Once installed, you can use the `toka` command to interact with the various parts of the system.

### Agent Management

- **Create a new agent:**
  ```sh
  toka agent new "my-awesome-agent"
  ```

- **List existing agents:**
  ```sh
  toka agent list
  ```

- **Observe an agent's event stream:**
  ```sh
  toka agent observe <AGENT_ID>
  ```

- **Get agent details:**
  ```sh
  toka agent show <AGENT_ID>
  ```

### Tool Management

- **List available tools:**
  ```sh
  toka tool list
  ```

- **Run a tool:**
  ```sh
  toka tool run echo --payload '{"message": "Hello, world!"}'
  ```

- **Get tool documentation:**
  ```sh
  toka tool docs <TOOL_NAME>
  ```

### Vault Interaction

- **Get a value from the vault:**
  ```sh
  toka vault get "my-secrets/api-key"
  ```

- **Store a value in the vault:**
  ```sh
  toka vault put "my-secrets/api-key" --value "super-secret-value"
  ```

- **List vault contents:**
  ```sh
  toka vault list
  ```

### System Operations

- **Check system status:**
  ```sh
  toka system status
  ```

- **View system logs:**
  ```sh
  toka system logs
  ```

- **Monitor events:**
  ```sh
  toka system events
  ```

## Building from Source

You can build and run the CLI from the workspace root:

```sh
cargo run -p toka-cli -- agent list
```

## Configuration

The CLI can be configured via environment variables or configuration files:

- `TOKA_CONFIG_PATH`: Path to configuration file
- `TOKA_LOG_LEVEL`: Logging level (debug, info, warn, error)
- `TOKA_VAULT_PATH`: Path to vault storage

## Design Philosophy

- **User-Friendly**: Intuitive command structure and helpful error messages
- **Secure**: Secure handling of sensitive operations like vault access
- **Extensible**: Easy to add new commands and subcommands
- **Observable**: Rich output for monitoring and debugging

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 