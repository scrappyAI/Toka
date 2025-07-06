# Toka Agent Testing - Quick Start

**Get the Toka agentic system up and running in 5 minutes**

## Prerequisites

- Linux/macOS system
- Rust 1.75+ installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Internet connection for API testing

## Step 1: Setup (3 minutes)

```bash
# Make setup script executable
chmod +x setup_toka_testing.sh

# Run interactive setup
./setup_toka_testing.sh
```

**What you'll be asked:**
1. Choose LLM provider (Anthropic recommended)
2. Enter your API key
3. Wait for build to complete

## Step 2: Interactive CLI (2 minutes)

```bash
# Make interactive CLI executable
chmod +x toka_interactive.sh

# Start interactive session
./toka_interactive.sh
```

**What it provides:**
- Interactive menu-driven interface
- Stateful operation with SQLite storage
- Real-time agent spawning and task scheduling
- System state queries and monitoring
- Configuration management
- Development token generation

## Step 3: Explore Features

**In the interactive menu:**
1. **Start Daemon Mode** - Run Toka with persistent storage
2. **Spawn Agent** - Create new agents interactively
3. **Schedule Task** - Assign tasks to agents
4. **Query State** - View current system state
5. **Generate Token** - Create development tokens
6. **View Logs** - Monitor system activity

**Or use CLI directly:**
```bash
# Generate a token
./target/release/toka generate-token

# Start daemon mode
./target/release/toka daemon

# Query system state
./target/release/toka query-state
```

## That's it! 🎉

You've successfully:
- ✅ Built the Toka CLI and orchestration components
- ✅ Set up stateful agent management with SQLite storage
- ✅ Created an interactive menu-driven interface
- ✅ Enabled real-time agent spawning and task scheduling
- ✅ Configured development token generation
- ✅ Established system monitoring and logging

## Next Steps

1. **Explore daemon mode:** Start persistent operation with SQLite storage
2. **Spawn agents:** Create interactive agents for specific tasks
3. **Schedule tasks:** Assign work to agents and monitor progress
4. **Configuration management:** Use `toka-config` for file management
5. **Check logs:** Monitor system activity in real-time

## Manual CLI Commands

You can also use the CLI directly:

```bash
# Generate a development token
./target/release/toka generate-token --subject "my-user" --permissions "read,write"

# Start daemon mode with SQLite storage
./target/release/toka --storage sqlite --db-path ./data/toka.db daemon

# Spawn an agent
./target/release/toka --storage sqlite --db-path ./data/toka.db spawn-agent --name "my-agent"

# Query system state
./target/release/toka --storage sqlite --db-path ./data/toka.db query-state

# Schedule a task
./target/release/toka --storage sqlite --db-path ./data/toka.db schedule-task --agent 123 --description "Process files"
```

## Troubleshooting

**Build fails?**
```bash
# Check Rust version
rustc --version

# Clean and retry
cargo clean
./setup_toka_testing.sh
```

**CLI issues?**
```bash
# Check if binaries exist
ls -la target/release/toka*

# Check logs
cat logs/toka-interactive.log

# Test basic CLI
./target/release/toka --help
```

**API key issues?**
```bash
# Verify environment
cat .env | grep API_KEY

# Re-run setup
./setup_toka_testing.sh
```

---

**🚀 You're now ready to explore the Toka agentic operating system!** 