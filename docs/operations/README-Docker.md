# Toka Background Agent Docker Setup

This document provides comprehensive instructions for running Toka background agents using Docker, optimized for security, performance, and efficient context caching.

## 🏗️ Architecture Overview

The Docker setup includes:
- **Multi-stage Dockerfile** for secure, optimized builds
- **Background agent orchestration** with resource limits and monitoring
- **Redis caching** for efficient context and response caching
- **Prometheus + Grafana** for comprehensive monitoring
- **SQLite persistence** for agent state and event storage

## 🚀 Quick Start

### Prerequisites
- Docker Engine 20.10+
- Docker Compose 2.0+
- At least 4GB RAM available
- LLM API keys (Anthropic or OpenAI)

### 1. Environment Setup

Create a `.env` file in the project root:

```bash
# Copy the example environment file
cp .env.example .env

# Edit with your configuration
nano .env
```

**Required environment variables:**
```env
# LLM Provider Configuration
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Security
JWT_SECRET=your_secure_jwt_secret_here
GRAFANA_PASSWORD=secure_admin_password
REDIS_PASSWORD=tokaagents_secure_password
```

### 2. Build and Run

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f toka-agents

# Check agent status
docker-compose ps
```

### 3. Access Services

- **Agent API**: http://localhost:8080
- **Agent Management**: http://localhost:9000
- **Grafana Dashboard**: http://localhost:3001 (admin/yourpassword)
- **Prometheus**: http://localhost:9090

## 🔧 Configuration

### Agent Configuration

Edit `config/agents.toml` to define your background agents:

```toml
[[agents]]
name = "custom-agent"
version = "1.0.0"
domain = "custom-domain"
priority = "high"
workstream = "main"
branch = "feature/custom-work"

[agents.capabilities]
primary = ["capability1", "capability2"]
secondary = ["capability3"]

[[agents.objectives]]
description = "Custom agent objective"
deliverable = "Expected output"
validation = "Validation criteria"
```

### Performance Tuning

Key performance settings in `docker-compose.yml`:

```yaml
environment:
  # Agent orchestration
  - AGENT_POOL_SIZE=10
  - MAX_CONCURRENT_AGENTS=8
  - TOKIO_WORKER_THREADS=4
  
  # Context caching
  - CONTEXT_CACHE_SIZE=2048
  - CONTEXT_CACHE_TTL=7200
  
  # Resource limits
  - MAX_MEMORY_PER_AGENT=512MB
  - MAX_CPU_PER_AGENT=0.5
```

## 🛡️ Security Features

### Built-in Security

1. **Multi-stage builds** with minimal attack surface
2. **Non-root user** execution
3. **Secure memory handling** with automatic cleanup
4. **Sandboxed agent execution**
5. **Rate limiting** on LLM API calls
6. **Input sanitization** and output validation

### Security Configuration

```yaml
environment:
  # Security settings
  - RUST_SECURE_ZERO_MEMORY=1
  - AGENT_SANDBOX_ENABLED=true
  - CAPABILITY_VALIDATION=strict
  
  # Rate limiting
  - RATE_LIMIT_REQUESTS=100
  - RATE_LIMIT_WINDOW=60
```

## 📊 Monitoring & Observability

### Metrics Collection

The setup includes comprehensive monitoring:

- **Agent Performance**: Task completion rates, execution times
- **Resource Usage**: CPU, memory, network per agent
- **LLM Integration**: API call latency, token usage, error rates
- **System Health**: Container health, storage usage

### Grafana Dashboards

Pre-configured dashboards monitor:
- Agent orchestration overview
- Individual agent performance
- LLM provider metrics
- System resource utilization

### Health Checks

```bash
# Check agent health
curl http://localhost:8080/health

# View metrics
curl http://localhost:8080/metrics

# Check specific agent status
curl http://localhost:9000/agents/status
```

## 🔄 Context Caching

### Cache Configuration

Efficient context caching is configured for:

1. **LLM Response Cache**: Stores frequently used responses
2. **Context Cache**: Maintains agent conversation context
3. **Redis Backend**: Persistent, distributed caching

### Cache Settings

```yaml
environment:
  # Context caching
  - CONTEXT_CACHE_SIZE=2048        # Max contexts cached
  - CONTEXT_CACHE_TTL=7200         # Cache lifetime (2 hours)
  - RESPONSE_CACHE_SIZE=1024       # Max responses cached
  - RESPONSE_CACHE_TTL=3600        # Response cache TTL (1 hour)
```

## 🐛 Troubleshooting

### Common Issues

1. **Agent spawn timeout**
   ```bash
   # Check logs
   docker-compose logs toka-agents
   
   # Increase timeout
   # Edit docker-compose.yml: AGENT_SPAWN_TIMEOUT=60
   ```

2. **Out of memory errors**
   ```bash
   # Monitor memory usage
   docker stats toka-background-agents
   
   # Reduce concurrent agents
   # Edit docker-compose.yml: MAX_CONCURRENT_AGENTS=4
   ```

3. **LLM API rate limits**
   ```bash
   # Check rate limit settings
   curl http://localhost:8080/metrics | grep llm_rate
   
   # Adjust limits in docker-compose.yml
   ```

### Debug Mode

Enable debug mode for detailed logging:

```yaml
environment:
  - RUST_LOG=debug
  - LLM_DEBUG=true
  - DEBUG_AGENTS=true
```

## 📈 Performance Optimization

### Resource Allocation

**For production environments:**

```yaml
deploy:
  resources:
    limits:
      cpus: '4.0'
      memory: 8G
    reservations:
      cpus: '2.0'
      memory: 4G
```

### Cache Optimization

**High-performance caching:**

```yaml
environment:
  - CONTEXT_CACHE_SIZE=4096
  - CONTEXT_CACHE_TTL=14400
  - RESPONSE_CACHE_SIZE=2048
```

### Scaling

**Horizontal scaling:**

```bash
# Scale agent instances
docker-compose up -d --scale toka-agents=3

# Load balancer configuration needed for multiple instances
```

## 🔒 Production Deployment

### Security Checklist

- [ ] Change default passwords
- [ ] Use secure JWT secrets
- [ ] Enable TLS/SSL
- [ ] Configure firewall rules
- [ ] Set up log rotation
- [ ] Enable audit logging

### Resource Requirements

**Minimum:**
- 4 CPU cores
- 8GB RAM
- 20GB storage

**Recommended:**
- 8 CPU cores
- 16GB RAM
- 100GB SSD storage

### Environment Variables

**Production-ready `.env`:**

```env
# Security
JWT_SECRET=$(openssl rand -hex 32)
GRAFANA_PASSWORD=$(openssl rand -base64 32)
REDIS_PASSWORD=$(openssl rand -base64 32)

# Performance
AGENT_POOL_SIZE=20
MAX_CONCURRENT_AGENTS=16
TOKIO_WORKER_THREADS=8

# Monitoring
METRICS_ENABLED=true
TRACING_ENABLED=true
LOG_LEVEL=info
```

## 📚 API Reference

### Agent Management Endpoints

- `GET /agents` - List all agents
- `POST /agents` - Create new agent
- `GET /agents/{id}` - Get agent details
- `PUT /agents/{id}` - Update agent
- `DELETE /agents/{id}` - Delete agent

### Monitoring Endpoints

- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /status` - System status

## 🤝 Contributing

When contributing to the Docker setup:

1. Test changes with `docker-compose up --build`
2. Update documentation for new features
3. Add monitoring for new components
4. Ensure security best practices

## 📄 License

This Docker setup is part of the Toka project and follows the same Apache 2.0 license. 