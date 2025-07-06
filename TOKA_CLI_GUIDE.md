# Toka Interactive CLI Guide

**Stateful, interactive interface to the Toka agentic operating system**

## Overview

The Toka Interactive CLI provides a menu-driven interface for managing agents, tasks, and system state using the actual Toka CLI binaries. This creates a stateful, persistent environment where you can spawn agents, schedule tasks, and monitor the system in real-time.

## Key Features

### ✅ **Stateful Operation**
- **SQLite storage** for persistent state across sessions
- **Daemon mode** for continuous background operation
- **Real-time state queries** to monitor system activity

### ✅ **Interactive Agent Management**
- **Spawn agents** with custom names and configurations
- **Schedule tasks** with specific descriptions and requirements
- **Monitor progress** through system state queries

### ✅ **Development Tools**
- **JWT token generation** for authentication
- **Configuration management** with YAML/JSON/TOML support
- **Log monitoring** with real-time viewing capabilities

### ✅ **Production-Ready Components**
- Uses actual Toka CLI binaries (not simulations)
- Leverages the production agent runtime (31 passing tests)
- Integrates with security framework (24 passing tests)

## Quick Start

### 1. Setup
```bash
# Run initial setup (builds CLI binaries)
./setup_toka_testing.sh

# Start interactive session
./toka_interactive.sh
```

### 2. Basic Workflow
1. **Start Daemon Mode** (option 1) - Enables persistent operation
2. **Generate Token** (option 6) - Create authentication token
3. **Spawn Agent** (option 3) - Create your first agent
4. **Schedule Task** (option 4) - Assign work to the agent
5. **Query State** (option 5) - Monitor system activity

## Menu Commands

### Core Operations
- **1. Start Daemon Mode** - Launch background Toka daemon with SQLite storage
- **2. Stop Daemon Mode** - Gracefully shutdown daemon process
- **3. Spawn Agent** - Create new agent with interactive name input
- **4. Schedule Task** - Assign tasks to agents with custom descriptions
- **5. Query State** - View current system state and agent activity

### System Management
- **6. Generate Token** - Create development JWT tokens for authentication
- **7. Check System Status** - View daemon, database, and log status
- **8. View Logs** - Monitor real-time system activity
- **9. Configuration Management** - Manage YAML/JSON/TOML configuration files
- **10. Run Tests** - Execute system validation tests

## Usage Examples

### Spawning an Agent
```
Choose option [0-10]: 3
Enter agent name: file-processor
🤖 Spawning agent: file-processor
✅ Agent 'file-processor' spawned successfully
```

### Scheduling a Task
```
Choose option [0-10]: 4
Enter agent ID (numeric): 123
Enter task description: Process documents in /data directory
📋 Scheduling task for agent 123: Process documents in /data directory
✅ Task scheduled successfully
```

### Generating a Token
```
Choose option [0-10]: 6
Enter subject [dev-user]: my-user
Enter vault [dev-vault]: my-vault
Enter permissions [read,write]: read,write,execute
🔑 Generating JWT token...
🔑 Generated JWT token:
👤 Subject: my-user
🏪 Vault: my-vault
🔐 Permissions: read,write,execute
🎫 Token: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
```

## Direct CLI Usage

You can also use the Toka CLI directly without the interactive menu:

### Token Generation
```bash
./target/release/toka generate-token \
    --subject "production-user" \
    --vault "production-vault" \
    --permissions "read,write"
```

### Daemon Mode
```bash
./target/release/toka \
    --storage sqlite \
    --db-path ./data/toka.db \
    --log-level info \
    daemon
```

### Agent Operations
```bash
# Spawn an agent
./target/release/toka \
    --storage sqlite \
    --db-path ./data/toka.db \
    spawn-agent --name "data-processor"

# Schedule a task
./target/release/toka \
    --storage sqlite \
    --db-path ./data/toka.db \
    schedule-task \
    --agent 456 \
    --description "Analyze logs and generate report"

# Query system state
./target/release/toka \
    --storage sqlite \
    --db-path ./data/toka.db \
    query-state
```

## Configuration Management

The interactive CLI includes full configuration management via `toka-config`:

### Available Operations
1. **List configuration files** - Find all config files in a directory
2. **Create configuration file** - New YAML/JSON/TOML files
3. **Read configuration file** - Display file contents with formatting
4. **Update configuration value** - Modify specific keys with dot notation
5. **Validate configuration file** - Check syntax and structure

### Example Configuration Operations
```bash
# Create a new agent configuration
toka-config create \
    --file config/my-agent.yaml \
    --format yaml \
    --content '{"name": "my-agent", "capabilities": ["file-processing"]}'

# Update a configuration value
toka-config update \
    --file config/my-agent.yaml \
    --key capabilities \
    --value '["file-processing", "data-analysis"]'

# Validate configuration
toka-config validate --file config/my-agent.yaml
```

## Storage and Persistence

### SQLite Database
- **Location**: `./data/toka.db`
- **Purpose**: Persistent storage for agents, tasks, and system state
- **Benefits**: Survives restarts, enables stateful operations

### Log Files
- **Interactive logs**: `./logs/toka-interactive.log`
- **System logs**: Various log files based on operations
- **Real-time monitoring**: View logs through the interactive menu

### Configuration Files
- **Agent configs**: `config/testing/agents.toml`
- **Environment**: `.env` file with API keys and settings
- **Custom configs**: Created via `toka-config` CLI

## System Status Monitoring

The interactive CLI provides comprehensive system monitoring:

### Status Checks
- **Daemon status**: Running/stopped with PID information
- **Database status**: File existence and size
- **Log status**: File location and line count
- **Configuration status**: Config file validation

### Example Status Output
```
✅ Daemon: Running (PID: 12345)
✅ Database: ./data/toka.db (256K)
✅ Logs: ./logs/toka-interactive.log (142 lines)
✅ Configuration: config/testing/agents.toml
```

## Advanced Features

### Hot Configuration Reloading
- Modify configuration files while daemon is running
- Changes are detected and applied automatically
- No need to restart daemon for configuration updates

### Multi-Agent Coordination
- Spawn multiple agents with different capabilities
- Schedule interdependent tasks
- Monitor agent interactions through state queries

### Development Workflow
1. Start daemon for persistent operation
2. Generate tokens for authentication
3. Spawn specialized agents for different tasks
4. Schedule work and monitor progress
5. Query state to understand system behavior
6. Use configuration management for adjustments

## Troubleshooting

### Common Issues

**Daemon won't start**
```bash
# Check if port/database is locked
ps aux | grep toka
rm -f .toka_daemon.pid
./toka_interactive.sh
```

**Agent spawn fails**
```bash
# Check logs for details
tail -f logs/toka-interactive.log

# Verify database state
./target/release/toka --storage sqlite --db-path ./data/toka.db query-state
```

**Configuration errors**
```bash
# Validate configuration files
./target/release/toka-config validate --file config/testing/agents.toml

# Check configuration syntax
./target/release/toka-config read --file config/testing/agents.toml
```

### Log Analysis
- Use menu option 8 to view recent log entries
- Check `logs/toka-interactive.log` for detailed operation logs
- Monitor daemon output for system-level information

## Production Considerations

### Security
- Generate unique JWT secrets for production
- Use appropriate permission sets for tokens
- Store sensitive configuration in environment variables

### Performance
- SQLite storage suitable for development and small deployments
- Consider PostgreSQL or distributed storage for large-scale production
- Monitor database size and performance metrics

### Monitoring
- Set up log aggregation for production environments
- Implement health checks for daemon processes
- Use system monitoring tools for resource usage

## Summary

The Toka Interactive CLI provides a complete, stateful interface to the Toka agentic operating system. It enables:

- **Real agent management** using production-ready components
- **Persistent operations** with SQLite storage
- **Interactive workflows** for development and testing
- **Configuration management** for system customization
- **Monitoring capabilities** for system observability

This approach gives you hands-on experience with the actual Toka CLI while providing a user-friendly interface for exploring the system's capabilities.

Ready to build intelligent agent systems? Start with `./toka_interactive.sh` and explore the future of agentic computing! 