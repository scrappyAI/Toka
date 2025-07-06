# Toka Testing Environment Guide

**A comprehensive guide to testing the Toka agentic operating system using proper Rust CLI tools.**

## Overview

This guide demonstrates the **correct** way to test the Toka agentic system using the native Rust CLI tools. The previous shell-script approach had authentication issues - this new approach fixes those problems and provides a proper testing environment.

## What's New

### 🔧 Fixed Authentication
- **Fixed CLI**: The `toka` CLI now properly accepts JWT tokens via `--token` parameter
- **Proper Token Generation**: Uses the built-in `generate-token` command for authentication
- **No More "demo-token"**: Removed hardcoded tokens that caused authentication failures

### 🦀 Rust-First Approach
- **Interactive Testing Tool**: New `toka-test` binary for comprehensive testing
- **Configuration Management**: Robust `toka-config` CLI for managing configs
- **Minimal Shell Scripts**: Only use shell scripts for workflow orchestration

### 🎮 Better User Experience
- **Interactive Mode**: Use `toka-test` for exploratory testing
- **Demo Mode**: Run `toka-test --demo` for automated scenarios
- **Helpful Error Messages**: Clear guidance when authentication fails

## Quick Start

### 1. Build the Tools

```bash
# Build all required binaries
cargo build --release --bin toka --bin toka-config --bin toka-test
```

### 2. Run the Workflow

```bash
# Run the complete workflow demonstration
chmod +x toka_workflow.sh
./toka_workflow.sh
```

### 3. Try Interactive Mode

```bash
# Run the interactive testing environment
./target/release/toka-test
```

## Core Components

### 🔑 Authentication (Fixed!)

The main issue was that the CLI was using hardcoded `"demo-token"` instead of proper JWT tokens. Now:

```bash
# Generate a proper JWT token
TOKEN=$(./target/release/toka generate-token --subject "admin" --permissions "read,write,admin" | grep "Token:" | cut -d' ' -f2)

# Use the token for operations
./target/release/toka spawn-agent --name "TestAgent" --token "$TOKEN"
```

### 🤖 Agent Operations

```bash
# Spawn an agent with authentication
./target/release/toka spawn-agent --name "FileAgent" --token "$TOKEN"

# Schedule a task for the agent
./target/release/toka schedule-task --agent 1 --description "Process files" --token "$TOKEN"

# Query the current system state
./target/release/toka query-state
```

### ⚙️ Configuration Management

```bash
# Create configuration files
./target/release/toka-config create --file config.json --format json --content '{...}'

# Read configurations
./target/release/toka-config read --file config.json

# Update specific keys
./target/release/toka-config update --file config.json --key "app.debug" --value "true"
```

### 🎮 Interactive Testing

```bash
# Run interactive mode
./target/release/toka-test

# Commands available in interactive mode:
toka-test> help          # Show all commands
toka-test> spawn FileAgent admin    # Spawn agent with admin token
toka-test> task FileAgent Process files  # Schedule task
toka-test> state         # Query system state
toka-test> demo          # Run demo scenarios
```

## Testing Scenarios

### Scenario 1: File Operations Agent

```bash
# Generate tokens
ADMIN_TOKEN=$(./target/release/toka generate-token --subject "admin" --permissions "read,write,admin" | grep "Token:" | cut -d' ' -f2)

# Spawn file agent
./target/release/toka spawn-agent --name "FileAgent" --token "$ADMIN_TOKEN"

# Schedule file processing tasks
./target/release/toka schedule-task --agent 1 --description "Read project configuration" --token "$ADMIN_TOKEN"
./target/release/toka schedule-task --agent 1 --description "Generate summary report" --token "$ADMIN_TOKEN"
```

### Scenario 2: System Monitoring Agent

```bash
# Generate user token
USER_TOKEN=$(./target/release/toka generate-token --subject "user" --permissions "read,write" | grep "Token:" | cut -d' ' -f2)

# Spawn monitoring agent
./target/release/toka spawn-agent --name "MonitorAgent" --token "$USER_TOKEN"

# Schedule monitoring tasks
./target/release/toka schedule-task --agent 2 --description "Check system resources" --token "$USER_TOKEN"
```

### Scenario 3: Configuration Management

```bash
# Create agent configuration
./target/release/toka-config create --file ./data/agents.json --format json --content '{
  "agents": {
    "FileAgent": {"enabled": true, "tools": ["ReadFile", "WriteFile"]},
    "MonitorAgent": {"enabled": true, "tools": ["RunCommand"]}
  }
}'

# Update configuration
./target/release/toka-config update --file ./data/agents.json --key "agents.FileAgent.enabled" --value "false"

# Validate configuration
./target/release/toka-config validate --file ./data/agents.json
```

## Interactive Testing Tool

The new `toka-test` binary provides the best testing experience:

### Features

- **Automatic Token Management**: Generates admin and user tokens on startup
- **Agent Tracking**: Keeps track of spawned agents by name
- **Interactive Commands**: Type commands directly instead of long CLI invocations
- **Demo Mode**: Run `--demo` flag for automated testing scenarios
- **Colored Output**: Beautiful terminal output with syntax highlighting

### Usage

```bash
# Interactive mode
./target/release/toka-test

# Demo mode (automated scenarios)
./target/release/toka-test --demo

# Custom storage
./target/release/toka-test --storage sqlite --db-path ./test.db
```

### Interactive Commands

```
help          - Show available commands
tokens        - List generated tokens
agents        - List spawned agents
spawn <name>  - Spawn a new agent
task <agent> <description> - Schedule a task
state         - Query system state
demo          - Run demo scenarios
token <name> <subject> <perms> - Generate custom token
quit/exit     - Exit the testing environment
```

## Files and Storage

### Generated Files

- `./data/toka-workflow.db` - SQLite database with persistent storage
- `./data/test-config.json` - Configuration management example
- `./logs/` - Application logs

### Storage Options

- **Memory**: `--storage memory` (no persistence)
- **SQLite**: `--storage sqlite --db-path ./data/toka.db` (recommended)
- **Sled**: `--storage sled --db-path ./data/toka-sled` (embedded key-value)

## Troubleshooting

### Common Issues

1. **Authentication Failures**: Make sure you're using proper JWT tokens generated by the `generate-token` command
2. **Build Errors**: Ensure all dependencies are installed with `cargo build --release`
3. **Database Locked**: Stop any running daemon processes before testing

### Debug Mode

```bash
# Run with debug logging
./target/release/toka --log-level debug <command>
./target/release/toka-test --log-level debug
```

## Architecture

### Authentication Flow

1. **Generate Token**: `toka generate-token` creates JWT with proper claims
2. **Validate Token**: Runtime validates JWT signature and permissions
3. **Authorize Operation**: Token permissions checked against operation requirements

### Agent Lifecycle

1. **Spawn Agent**: `spawn-agent` creates new agent with unique EntityId
2. **Schedule Tasks**: `schedule-task` assigns work to specific agents
3. **Query State**: `query-state` shows current agents and their tasks

### Configuration Management

1. **Create/Read/Update/Delete**: Full CRUD operations on config files
2. **Format Support**: JSON, YAML, and TOML formats
3. **Validation**: Syntax and structure validation
4. **Dot Notation**: Nested key access (e.g., `app.database.host`)

## Next Steps

1. **Explore Interactive Mode**: Run `./target/release/toka-test` and try different commands
2. **Try Daemon Mode**: Start `./target/release/toka daemon` for persistent operation
3. **Custom Scenarios**: Create your own testing scenarios using the CLI tools
4. **Integration**: Integrate the CLI tools into your own applications

## Summary

The improved Toka testing environment provides:

- ✅ **Fixed Authentication**: Proper JWT token support
- ✅ **Rust-First Approach**: Native CLI tools instead of shell scripts
- ✅ **Interactive Testing**: User-friendly `toka-test` environment
- ✅ **Configuration Management**: Robust `toka-config` tool
- ✅ **Better UX**: Clear error messages and helpful guidance

This is the **correct** way to test the Toka agentic system - using the proper Rust CLI tools with authenticated operations. 