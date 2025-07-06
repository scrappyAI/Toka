# Toka Agentic System Testing Environment

**A focused testing environment for the Toka OS agentic system with minimal configuration and clear examples.**

## Overview

This testing environment provides a stripped-down baseline for testing Toka's agentic capabilities. Based on the production readiness report, all core systems are operational and ready for testing.

### What's Included

- **3 Test Agents**: File operations, system monitoring, and API research
- **Essential Tools**: ReadFile, WriteFile, RunCommand, HttpRequest  
- **Complete Setup**: First-time user onboarding with API key configuration
- **Validation Scripts**: Automated testing and health checks
- **Minimal Configuration**: Focused on core functionality

## Production Readiness Status

✅ **PRODUCTION READY** - All critical components operational:
- **Agent Runtime**: 31 passing tests, complete execution pipeline
- **Security Framework**: 24 passing tests, capability delegation working
- **Essential Tools**: All 4 essential tools functional
- **Build System**: Clean compilation in release mode
- **Integration**: 183 total tests, 98.4% success rate

## Quick Start

### 1. Initial Setup

```bash
# Make setup script executable
chmod +x setup_toka_testing.sh

# Run the setup (interactive)
./setup_toka_testing.sh
```

The setup script will:
- Check Rust installation
- Configure LLM provider (Anthropic Claude or OpenAI)
- Create environment configuration
- Build the orchestration service
- Create test agent configurations
- Generate test input files

### 2. Start Testing

```bash
# Make testing script executable
chmod +x test_toka_agents.sh

# Start service and run all tests
./test_toka_agents.sh full
```

Or run individual steps:
```bash
# Start the orchestration service
./test_toka_agents.sh start

# Run agent tests
./test_toka_agents.sh test

# Check logs
./test_toka_agents.sh logs

# Stop service
./test_toka_agents.sh stop
```

## Test Agents

### 1. File Operations Agent
- **Purpose**: Demonstrates file reading, writing, and text processing
- **Capabilities**: `filesystem-read`, `filesystem-write`, `text-processing`
- **Test**: Processes `data/test_input.txt` and creates analysis summary
- **Output**: `data/file_ops_output.txt`

### 2. System Monitoring Agent  
- **Purpose**: Shows system monitoring and resource reporting
- **Capabilities**: `system-monitoring`, `command-execution`, `report-generation`
- **Test**: Gathers system metrics and generates status report
- **Output**: `data/system_monitor_report.txt`

### 3. API Research Agent
- **Purpose**: Demonstrates HTTP requests and data processing
- **Capabilities**: `http-requests`, `data-processing`, `json-parsing`
- **Test**: Fetches data from JSONPlaceholder API and analyzes results
- **Output**: `data/api_research_report.txt`

## Architecture Overview

### Core Components

```
Toka Testing Environment
├── Orchestration Service (HTTP API)
│   ├── Health checks (/health)
│   ├── Status monitoring (/status)
│   └── Agent management (/agents)
├── Agent Runtime
│   ├── Agent execution engine
│   ├── Task coordination
│   └── Progress reporting
├── Essential Tools
│   ├── ReadFileTool
│   ├── WriteFileTool
│   ├── RunCommandTool
│   └── HttpRequestTool
└── Storage Layer
    ├── SQLite database
    ├── File system
    └── In-memory cache
```

### Security Model

- **Capability-based security**: Agents must declare required capabilities
- **Resource limits**: CPU, memory, and timeout enforcement
- **Sandboxing**: Process isolation (configurable)
- **JWT authentication**: Token-based security for API access

## Configuration Files

### Environment Variables (`.env`)
```bash
# LLM Configuration
ANTHROPIC_API_KEY=your_api_key_here
LLM_PROVIDER=anthropic
LLM_MODEL=claude-3-5-sonnet-20241022

# Storage
DATABASE_URL=sqlite:///app/data/agents.db
STORAGE_TYPE=sqlite

# Agent Settings
AGENT_POOL_SIZE=3
MAX_CONCURRENT_AGENTS=2
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict
```

### Agent Configuration (`config/testing/agents.toml`)
```toml
[orchestration]
max_concurrent_agents = 2
agent_spawn_timeout = 30
workstream_timeout = 1800

[[agents]]
name = "file-ops-agent"
domain = "file-operations"
priority = "high"

[agents.capabilities]
primary = ["filesystem-read", "filesystem-write", "text-processing"]
```

## API Endpoints

### Health Check
```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.2.1",
  "orchestration_status": "running",
  "agent_count": 3,
  "uptime_seconds": 150
}
```

### Orchestration Status
```bash
curl http://localhost:8080/status
```

Response:
```json
{
  "session_id": "session_12345",
  "current_phase": "Executing",
  "progress": 0.75,
  "completed": false,
  "error": null,
  "spawned_agents": 2
}
```

## Testing Workflow

### Manual Testing Steps

1. **Environment Setup**
   ```bash
   ./setup_toka_testing.sh
   ```

2. **Service Start**
   ```bash
   ./test_toka_agents.sh start
   ```

3. **Health Verification**
   ```bash
   curl http://localhost:8080/health
   ```

4. **Agent Testing**
   ```bash
   ./test_toka_agents.sh test
   ```

5. **Results Validation**
   ```bash
   # Check generated files
   ls -la data/
   cat data/file_ops_output.txt
   cat data/system_monitor_report.txt
   cat data/api_research_report.txt
   ```

### Automated Testing

The testing script provides several automation options:

```bash
# Full automated test cycle
./test_toka_agents.sh full

# Individual components
./test_toka_agents.sh health    # Health check only
./test_toka_agents.sh status    # Status check only
./test_toka_agents.sh test      # Agent tests only
./test_toka_agents.sh logs      # View logs only
```

## Expected Outputs

### File Operations Test
- **Input**: `data/test_input.txt` (sample text file)
- **Output**: `data/file_ops_output.txt` (analysis report)
- **Validation**: Report contains content summary and structure analysis

### System Monitoring Test
- **Input**: System state and metrics
- **Output**: `data/system_monitor_report.txt` (system report)
- **Validation**: Report contains CPU, memory, disk, and process information

### API Research Test
- **Input**: JSONPlaceholder API endpoint
- **Output**: `data/api_research_report.txt` (API analysis)
- **Validation**: Report contains API response data and analysis

## Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Check Rust version
   rustc --version
   
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

2. **Service Won't Start**
   ```bash
   # Check port availability
   lsof -i :8080
   
   # Check logs
   cat logs/orchestration.log
   ```

3. **Agent Tests Fail**
   ```bash
   # Verify service is running
   curl http://localhost:8080/health
   
   # Check configuration
   cat config/testing/agents.toml
   ```

4. **LLM Integration Issues**
   ```bash
   # Verify API key
   echo $ANTHROPIC_API_KEY
   
   # Check environment
   cat .env | grep LLM
   ```

### Log Locations

- **Orchestration Service**: `logs/orchestration.log`
- **Agent Runtime**: `logs/agent-runtime.log`
- **Test Output**: `data/*.txt` files

## Directory Structure

```
toka/
├── setup_toka_testing.sh        # Initial setup script
├── test_toka_agents.sh          # Testing script
├── README_TOKA_TESTING.md       # This file
├── .env                         # Environment variables
├── config/
│   └── testing/
│       └── agents.toml          # Agent configurations
├── data/                        # Test data and outputs
│   ├── test_input.txt          # File operations test input
│   ├── file_ops_output.txt     # File operations results
│   ├── system_monitor_report.txt # System monitoring results
│   └── api_research_report.txt  # API research results
├── logs/                        # Service logs
│   └── orchestration.log       # Main service log
└── target/
    └── release/
        └── toka-orchestration-service # Built binary
```

## Next Steps

After successful testing, consider:

1. **Custom Agents**: Create your own agent configurations
2. **Tool Integration**: Add custom tools for specific use cases
3. **Production Deployment**: Scale up with full configuration
4. **Monitoring**: Add Prometheus/Grafana for metrics
5. **Security**: Implement production-grade security policies

## Support

For issues or questions:
1. Check the logs in `logs/orchestration.log`
2. Review the production readiness report
3. Verify all prerequisites are met
4. Ensure API keys are properly configured

## Key Features Demonstrated

- ✅ **Agent Orchestration**: Complete lifecycle management
- ✅ **Tool Integration**: Essential tools working with agents
- ✅ **Security**: Capability-based access control
- ✅ **Storage**: Persistent data management
- ✅ **Monitoring**: Health checks and status reporting
- ✅ **LLM Integration**: AI-powered agent intelligence

This testing environment provides a solid foundation for exploring Toka's agentic capabilities and serves as a baseline for further development and customization. 