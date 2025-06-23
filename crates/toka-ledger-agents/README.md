# Toka Ledger Agents

Agent-specific ledger operations for the Toka platform.

## Overview

This crate provides ledger operations specifically designed for agent activities and interactions. It extends the core ledger functionality with agent-specific transaction types and operations.

## Features

- Agent-specific transaction types
- Agent activity tracking
- Agent interaction logging
- Integration with agent runtime
- Event-driven ledger updates
- Agent performance metrics

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-ledger-agents = "0.1.0"
```

### Example

```rust
use toka_ledger_agents::{AgentLedger, AgentTransaction};
use toka_primitives::AgentID;

let ledger = AgentLedger::new();

// Log agent creation
ledger.log_agent_created(agent_id, metadata).await?;

// Log agent interaction
ledger.log_interaction(
    agent_id,
    "tool_executed",
    interaction_data
).await?;

// Get agent activity summary
let summary = ledger.get_agent_summary(agent_id).await?;
```

## Integration

This crate integrates with:
- `toka-ledger-core` for core ledger functionality
- `toka-agents` for agent runtime integration
- `toka-events-core` for event-driven updates

## Design Philosophy

- **Agent-Centric**: Designed specifically for agent workflows
- **Event-Driven**: Integrates with the event system for real-time updates
- **Observable**: Comprehensive logging of agent activities
- **Extensible**: Support for custom agent transaction types

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 