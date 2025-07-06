# Toka Orchestration System

A comprehensive agent orchestration platform that coordinates agent spawning, lifecycle management, and provides intelligent coordination for the Toka agentic operating system.

## Quick Start

### 1. Environment Setup

Copy the example environment file and configure your API keys:

```bash
cp env.example .env
# Edit .env with your API keys
```

**Required Environment Variables:**
- `ANTHROPIC_API_KEY` - Your Anthropic Claude API key (recommended)
- `OPENAI_API_KEY` - Your OpenAI API key (alternative)

### 2. Start Orchestration

```bash
# Check configuration and environment
make orchestration-check

# Build and start the orchestration system
make orchestration-start
```

## Features

### 🚀 **Agent Orchestration**
- Parallel agent spawning with dependency resolution
- Intelligent workstream coordination
- Resource management and conflict resolution
- Automatic agent lifecycle management

### 🔧 **Configuration Management**
- TOML-based agent configuration
- Environment variable validation
- Hot-reloading of configuration changes
- Multiple configuration profiles (standard, Cursor mode)

### 📊 **Monitoring & Health Checks**
- Real-time orchestration status
- HTTP health check endpoints
- Progress tracking and reporting
- Comprehensive logging and tracing

### 🤖 **LLM Integration**
- Intelligent agent coordination using Claude or GPT
- Context-aware decision making
- Automatic fallback to rule-based coordination
- Secure API key management

### 🐳 **Docker Support**
- Multi-service Docker Compose setup
- Integrated monitoring with Prometheus and Grafana
- Development and production configurations
- Automatic service dependency management

## Usage

### Command Line

```bash
# Start with default configuration
./scripts/start-orchestration.sh

# Start in development mode
./scripts/start-orchestration.sh --dev --log-level debug

# Start in Cursor mode for background agents
./scripts/start-orchestration.sh --cursor-mode

# Use custom configuration
./scripts/start-orchestration.sh --config config/custom.toml

# Check configuration only
./scripts/start-orchestration.sh --check-only
```

### Make Targets

```bash
# Build orchestration service
make orchestration-build

# Start orchestration system
make orchestration-start

# Start in development mode
make orchestration-dev

# Start in Cursor mode
make orchestration-cursor

# Check configuration and environment
make orchestration-check

# Get orchestration status
make orchestration-status

# Start with Docker
make orchestration-docker

# Stop Docker services
make orchestration-stop
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f toka-agents

# Stop all services
docker-compose down
```

## Configuration

### Agent Configuration

Edit `config/agents.toml` to define your agents:

```toml
[orchestration]
max_concurrent_agents = 8
agent_spawn_timeout = 30
workstream_timeout = 3600

[[agents]]
name = "my-agent"
version = "1.0.0"
domain = "automation"
priority = "high"
workstream = "main"

[agents.capabilities]
primary = ["task-execution", "data-processing"]
secondary = ["monitoring", "reporting"]

[[agents.objectives]]
description = "Automate specific tasks"
deliverable = "Task completion report"
validation = "Success metrics achieved"
```

### Environment Variables

Key environment variables in `.env`:

```env
# LLM Configuration
ANTHROPIC_API_KEY=your_api_key_here
LLM_PROVIDER=anthropic
LLM_MODEL=claude-3-5-sonnet-20241022

# Database
DATABASE_URL=sqlite:///app/data/agents.db

# Agent Settings
AGENT_POOL_SIZE=10
MAX_CONCURRENT_AGENTS=8

# Security
JWT_SECRET=your-secret-key
CAPABILITY_VALIDATION=strict
```

## API Endpoints

The orchestration service provides HTTP endpoints for monitoring and control:

### Health Check
```bash
GET http://localhost:8080/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.2.1",
  "orchestration_status": "running",
  "agent_count": 4,
  "uptime_seconds": 1234
}
```

### Orchestration Status
```bash
GET http://localhost:8080/status
```

Response:
```json
{
  "session_id": "uuid-here",
  "current_phase": "ParallelDevelopment",
  "progress": 0.65,
  "completed": false,
  "error": null,
  "spawned_agents": 3
}
```

### List Agents
```bash
GET http://localhost:8080/agents
```

Response:
```json
["code-analyst", "test-orchestrator", "security-auditor"]
```

## Development

### Building

```bash
# Build orchestration service
cargo build --release --bin toka-orchestration

# Build all components
cargo build --workspace
```

### Testing

```bash
# Test orchestration components
cargo test -p toka-orchestration
cargo test -p toka-orchestration-service

# Test entire workspace
cargo test --workspace
```

### Debugging

```bash
# Start with debug logging
./scripts/start-orchestration.sh --dev --log-level debug

# Check configuration
./scripts/start-orchestration.sh --check-only

# Validate environment
make orchestration-check
```

## Orchestration Phases

The system operates through several phases:

1. **Initialization** - Load configuration and validate environment
2. **Critical Infrastructure** - Spawn essential system agents
3. **Foundation Services** - Start core service agents
4. **Parallel Development** - Launch development agents in parallel
5. **Monitoring** - Active monitoring and coordination
6. **Completion** - Graceful shutdown and cleanup

## Security

### Authentication
- JWT-based authentication for all agent operations
- Secure API key storage using the `secrecy` crate
- Environment variable validation and sanitization

### Agent Security
- Sandbox mode for isolated agent execution
- Capability-based security model
- Resource limits and timeout enforcement
- Audit logging for all orchestration actions

### Network Security
- TLS-encrypted communications
- Rate limiting for API endpoints
- Request validation and sanitization

## Monitoring

### Metrics
- Prometheus metrics collection
- Grafana dashboards for visualization
- Custom metrics for agent performance
- Resource usage tracking

### Logging
- Structured logging with tracing
- Multiple log levels and filtering
- JSON log format for machine parsing
- Log rotation and retention

### Health Checks
- HTTP health check endpoints
- Service dependency validation
- Automatic failure detection
- Graceful degradation handling

## Troubleshooting

### Common Issues

**Environment Variables Not Set**
```bash
# Check environment configuration
make orchestration-check

# Verify .env file exists and has required keys
cat .env | grep -E "(ANTHROPIC|OPENAI)_API_KEY"
```

**Configuration Errors**
```bash
# Validate configuration file
./scripts/start-orchestration.sh --check-only

# Check configuration syntax
cargo run --bin toka-config-cli -- validate config/agents.toml
```

**Service Not Starting**
```bash
# Check logs for errors
docker-compose logs toka-agents

# Verify port availability
netstat -tulpn | grep :8080
```

**Agent Spawning Issues**
```bash
# Check orchestration status
curl http://localhost:8080/status

# Review orchestration logs
docker-compose logs -f toka-agents
```

### Debug Mode

Enable debug mode for detailed troubleshooting:

```bash
# Start with debug logging
./scripts/start-orchestration.sh --dev --log-level debug

# Or with environment variable
export RUST_LOG=debug
make orchestration-start
```

## Contributing

### Code Style
- Follow Rust best practices and workspace conventions
- Use `cargo fmt` and `cargo clippy` for code formatting
- Write comprehensive tests for new features
- Document all public APIs

### Testing
- Unit tests for individual components
- Integration tests for orchestration workflows
- Performance tests for scalability
- Security tests for vulnerability assessment

### Documentation
- Update README files for new features
- Document API changes and breaking changes
- Provide examples for new configuration options
- Maintain changelog for version releases

## License

This project is licensed under the Apache License 2.0. See [LICENSE-APACHE](LICENSE-APACHE) for details. 