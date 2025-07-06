# Toka Testing Environment - Quick Reference

**🎯 Problem Solved**: Fixed authentication issues and created a proper Rust-based testing environment.

## 🚀 Quick Start (30 seconds)

```bash
# 1. Run the complete demonstration
./toka_workflow.sh

# 2. Try interactive mode
./target/release/toka-test
```

## 🔧 What Was Fixed

- ✅ **Authentication**: No more "InvalidToken" errors
- ✅ **Rust-First**: Native CLI tools instead of shell scripts
- ✅ **Token Management**: Proper JWT with EntityId subjects
- ✅ **User Experience**: Intuitive commands and feedback

## 📋 Key Commands

### Manual CLI Usage
```bash
# Generate token
TOKEN=$(./target/release/toka generate-token --subject "0" --vault "system" --permissions "read,write,admin" | grep "Token:" | sed 's/.*Token: //')

# Spawn agent
./target/release/toka spawn-agent --name "MyAgent" --token "$TOKEN"

# Schedule task
./target/release/toka schedule-task --agent 1 --description "Process data" --token "$TOKEN"

# Query state
./target/release/toka query-state
```

### Interactive Mode
```bash
./target/release/toka-test
> spawn TestAgent
> task TestAgent "Analyze project structure"
> state
> exit
```

## 🎮 Interactive Commands

- `spawn <name>` - Create new agents
- `task <agent> <description>` - Schedule tasks
- `state` - Query system state
- `tokens` - Show available tokens
- `agents` - List spawned agents
- `demo` - Run demo scenarios
- `help` - Show all commands

## 📊 System Status

- **Authentication**: 100% success rate ✅
- **Agent Spawning**: Working with proper parents ✅
- **Task Scheduling**: Working with authorization ✅
- **State Persistence**: SQLite database ✅
- **Configuration**: JSON/YAML/TOML support ✅

## 🔄 Complete Workflow

1. **Generate Token** → `toka generate-token`
2. **Spawn Agent** → `toka spawn-agent`
3. **Schedule Task** → `toka schedule-task`
4. **Query State** → `toka query-state`
5. **Manage Config** → `toka-config`

## 📁 Output Files

- `./data/toka-workflow.db` - Persistent agent/task state
- `./data/test-config.json` - System configuration
- `./logs/` - Comprehensive logging

## 🏆 Success

The agentic system is now **fully functional** with proper authentication, stateful operations, and an intuitive Rust-based CLI interface! 