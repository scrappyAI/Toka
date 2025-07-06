# Toka Cursor Integration

This document explains how to set up and use Toka's background agents with Cursor's environment containers, providing seamless integration between Toka's native orchestration system and Cursor's development environment.

## 🏗️ Architecture Overview

The Cursor integration provides:

- **Dual Compatibility**: Works with both Toka's native environments and Cursor's background agent containers
- **Cursor-Optimized Agents**: Specialized agents for code assistance, git operations, and file management
- **Real-time Communication**: WebSocket integration for live updates
- **Enhanced Monitoring**: Cursor-specific health checks and performance metrics
- **Secure Sandboxing**: Isolated execution with strict capability validation

## 🔧 Components

### Core Files

```
.cursor/
├── environment.json          # Cursor environment configuration
├── Dockerfile               # Cursor-optimized Docker image
├── README-Cursor-Integration.md
└── scripts/
    ├── cursor-agent-init.sh     # Initialization script
    └── cursor-health-check.sh   # Health monitoring script

config/
└── cursor-agents.toml       # Cursor agent configuration
```

### Environment Configuration

The `environment.json` follows Cursor's schema and provides:

- **Container Build**: Optimized Dockerfile for cursor agents
- **Port Forwarding**: API (8080), Management (9000), WebSocket (9001)
- **Terminal Integration**: Pre-configured terminals for monitoring
- **Repository Dependencies**: GitHub access for Toka repository

### Cursor-Specific Agents

1. **cursor-code-assistant**: Real-time code analysis and suggestions
2. **cursor-git-assistant**: Version control operations and conflict resolution
3. **cursor-file-manager**: File system operations and search
4. **cursor-performance-monitor**: Resource monitoring and optimization

## 🚀 Getting Started

### Prerequisites

- Cursor IDE with environment support
- Docker Engine 20.10+
- Git access to Toka repository
- LLM API keys (Anthropic or OpenAI)

### 1. Environment Setup

The Cursor environment is automatically configured when you use Cursor's environment feature:

```bash
# Cursor will automatically:
# 1. Build the cursor-optimized Docker image
# 2. Start the cursor agent environment
# 3. Configure port forwarding
# 4. Launch monitoring terminals
```

### 2. Manual Setup (Development)

For development or testing outside Cursor:

```bash
# Build the cursor-optimized image
docker build -f .cursor/Dockerfile -t toka-cursor-agents .

# Run the cursor agent container
docker run -d \
  --name toka-cursor-agents \
  -p 8080:8080 \
  -p 9000:9000 \
  -p 9001:9001 \
  -p 3000:3000 \
  -e ANTHROPIC_API_KEY=your_key_here \
  -e CURSOR_AGENT_MODE=true \
  -v cursor_data:/app/data \
  -v cursor_logs:/app/logs \
  toka-cursor-agents
```

### 3. Verification

Check that the cursor agents are running:

```bash
# Health check
curl http://localhost:8080/health/cursor

# Agent status
curl http://localhost:9000/agents/status

# WebSocket connectivity
curl -I http://localhost:9001
```

## 🔧 Configuration

### Cursor Agent Configuration

The `config/cursor-agents.toml` file configures cursor-specific behavior:

```toml
[cursor_integration]
# Cursor-specific communication settings
websocket_port = 9001
api_timeout = 30
max_concurrent_requests = 50
heartbeat_interval = 15

# Cursor agent capabilities
cursor_file_operations = true
cursor_git_integration = true
cursor_lsp_integration = true
cursor_terminal_access = true
```

### Environment Variables

Key environment variables for cursor integration:

```bash
# Cursor mode
CURSOR_AGENT_MODE=true
CURSOR_SANDBOX_ENABLED=true
CURSOR_CAPABILITY_VALIDATION=strict

# Performance optimization
CURSOR_AGENT_POOL_SIZE=12
CURSOR_CONTEXT_CACHE_SIZE=2048
CURSOR_CONTEXT_CACHE_TTL=7200

# Communication
CURSOR_WEBSOCKET_ENABLED=true
CURSOR_WEBSOCKET_PORT=9001
CURSOR_MAX_CONCURRENT_REQUESTS=50
```

## 📊 Monitoring & Observability

### Cursor-Specific Monitoring

The integration provides enhanced monitoring:

1. **Health Checks**: Comprehensive health monitoring with cursor-specific checks
2. **Performance Metrics**: Resource usage, response times, and throughput
3. **Agent Status**: Real-time agent state and task progress
4. **Error Tracking**: Detailed error logging and analysis

### Health Check Script

Run the health check script to monitor cursor agent health:

```bash
# Run health check
/app/scripts/cursor-health-check.sh

# Sample output:
# ==================================
#    CURSOR AGENT HEALTH SUMMARY
# ==================================
# 
# Overall Status: HEALTHY
# 
# No health issues detected
# ==================================
```

### Monitoring Endpoints

- **Health**: `GET /health/cursor` - Cursor-specific health check
- **Metrics**: `GET /metrics` - Prometheus metrics with cursor labels
- **Status**: `GET /agents/status` - Agent status and performance
- **WebSocket**: `WS /ws` - Real-time updates and communication

## 🛡️ Security Features

### Cursor-Specific Security

1. **Sandboxed Execution**: Isolated agent execution with restricted capabilities
2. **File Access Control**: Limited file system access with validation
3. **Git Operation Limits**: Controlled version control operations
4. **Resource Limits**: CPU, memory, and disk usage constraints
5. **API Rate Limiting**: Controlled access to external APIs

### Security Configuration

```toml
[security.cursor_resource_limits]
max_memory = "768MB"
max_cpu = "0.75"
timeout = "1800"
max_file_operations = 100
max_git_operations = 50
```

## 🔄 Context Caching

### Cursor-Optimized Caching

The integration provides specialized caching:

1. **File Cache**: Frequently accessed files and their metadata
2. **Git Cache**: Repository status, diffs, and commit information
3. **Context Cache**: Conversation context and agent state
4. **Response Cache**: LLM responses for common queries

### Cache Configuration

```toml
[caching]
# Cursor-specific caching
cursor_file_cache_size = 1024
cursor_file_cache_ttl = 1800
cursor_git_cache_size = 512
cursor_git_cache_ttl = 3600
```

## 🔌 API Integration

### Cursor Agent API

The cursor integration provides REST and WebSocket APIs:

#### REST API

```bash
# Get agent status
GET /agents/status

# Create new task
POST /agents/tasks
{
  "agent_id": "cursor-code-assistant",
  "task_type": "code_analysis",
  "parameters": {
    "file_path": "src/main.rs",
    "analysis_type": "security"
  }
}

# Get task result
GET /agents/tasks/{task_id}
```

#### WebSocket API

```javascript
// Connect to cursor agent WebSocket
const ws = new WebSocket('ws://localhost:9001/ws');

// Send command
ws.send(JSON.stringify({
  type: 'agent_command',
  agent_id: 'cursor-git-assistant',
  command: 'git_status'
}));

// Receive updates
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Agent update:', data);
};
```

## 🛠️ Development & Debugging

### Development Mode

Enable development mode for detailed logging:

```bash
# Set environment variables
export CURSOR_DEBUG=true
export RUST_LOG=debug
export LLM_DEBUG=true

# Run with debug output
/app/scripts/cursor-agent-init.sh
```

### Debugging Tools

1. **Log Analysis**: Real-time log monitoring and analysis
2. **Performance Profiling**: Resource usage and bottleneck identification
3. **Agent Introspection**: Internal agent state and execution flow
4. **API Testing**: Built-in API testing and validation

### Debug Commands

```bash
# View real-time logs
tail -f /app/logs/cursor-agents.log

# Check agent performance
curl http://localhost:9000/agents/performance

# Debug specific agent
curl http://localhost:9000/agents/cursor-code-assistant/debug
```

## 🚀 Advanced Usage

### Custom Agent Development

Create custom cursor agents by extending the base configuration:

```toml
[[agents]]
name = "custom-cursor-agent"
version = "1.0.0"
domain = "custom-domain"
priority = "high"
workstream = "cursor-main"
cursor_agent = true

[agents.cursor_capabilities]
custom_operations = ["operation1", "operation2"]
file_operations = ["read", "write"]
```

### Integration with Toka Native

Use cursor agents alongside native Toka agents:

```bash
# Start both cursor and native agents
docker-compose -f docker-compose.yml -f docker-compose.cursor.yml up -d

# Monitor both environments
curl http://localhost:8080/health/cursor  # Cursor agents
curl http://localhost:8080/health         # Native agents
```

### Performance Optimization

Optimize cursor agent performance:

```bash
# Increase resource limits
export CURSOR_AGENT_POOL_SIZE=20
export CURSOR_CONTEXT_CACHE_SIZE=4096

# Enable performance profiling
export CURSOR_PERFORMANCE_PROFILING=true

# Use optimized caching
export CURSOR_CACHE_STRATEGY=aggressive
```

## 📈 Scaling

### Horizontal Scaling

Scale cursor agents for high-load scenarios:

```bash
# Scale cursor agent containers
docker-compose up -d --scale toka-cursor-agents=3

# Configure load balancing
# (requires additional load balancer configuration)
```

### Resource Management

Optimize resource allocation:

```yaml
# docker-compose.yml
services:
  toka-cursor-agents:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
```

## 🔍 Troubleshooting

### Common Issues

1. **Agent Not Starting**
   ```bash
   # Check logs
   docker logs toka-cursor-agents
   
   # Verify configuration
   /app/bin/toka-config-cli validate --config /app/config/cursor-agents.toml
   ```

2. **WebSocket Connection Failed**
   ```bash
   # Check port availability
   netstat -an | grep 9001
   
   # Test WebSocket connectivity
   curl -I http://localhost:9001
   ```

3. **High Resource Usage**
   ```bash
   # Monitor resource usage
   docker stats toka-cursor-agents
   
   # Run performance analysis
   /app/scripts/cursor-health-check.sh
   ```

### Debug Mode

Enable comprehensive debugging:

```bash
# Environment variables
export CURSOR_DEBUG=true
export RUST_LOG=debug
export TOKIO_CONSOLE=127.0.0.1:6669

# Run with debugging
docker run -e CURSOR_DEBUG=true -e RUST_LOG=debug toka-cursor-agents
```

## 🤝 Contributing

When contributing to the cursor integration:

1. **Test Both Environments**: Verify compatibility with both Toka native and cursor environments
2. **Update Documentation**: Keep cursor-specific documentation current
3. **Security Review**: Ensure cursor-specific security measures are maintained
4. **Performance Testing**: Validate performance with cursor-specific workloads

## 📄 License

This cursor integration is part of the Toka project and follows the same Apache 2.0 license. 