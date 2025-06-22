# Toka CLI

This crate provides a command-line interface (CLI) for interacting with the Toka runtime. It allows you to manage agents, execute tools, and interact with the secure vault.

_Note: This CLI is currently a skeleton implementation. The commands are defined, but they are not yet fully wired up to the runtime._

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

### Tool Management

- **List available tools:**
  ```sh
  toka tool list
  ```

- **Run a tool:**
  ```sh
  toka tool run echo --payload '{"message": "Hello, world!"}'
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

## Building from Source

You can build and run the CLI from the workspace root:

```sh
cargo run -p toka-cli -- agent list
``` 