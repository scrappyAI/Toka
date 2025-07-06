# Environment Configuration

> **Category**: Configuration  
> **Location**: `config/environments/`  
> **Status**: Stable

This directory contains environment-specific configuration templates for the Toka OS project.

## 📋 Quick Navigation

- [**Environment Templates**](#environment-templates) - Configuration templates
- [**Usage Instructions**](#usage-instructions) - How to use these configurations
- [**Configuration Standards**](#configuration-standards) - Configuration guidelines

## 🔧 Environment Templates

| File | Description | Target Environment |
|------|-------------|-------------------|
| [env.dev](env.dev) | Development environment configuration | Development |
| [env.prod](env.prod) | Production environment configuration | Production |
| [env.cursor](env.cursor) | Cursor IDE background agents | Cursor |

### Environment Overview

#### Development Environment (`env.dev`)
- **Purpose**: Development and testing with debugging enabled
- **Features**: Debug logging, development-friendly timeouts, SQLite database
- **Security**: Permissive settings for development convenience
- **Monitoring**: Development monitoring stack

#### Production Environment (`env.prod`)
- **Purpose**: Production deployment with security hardening
- **Features**: Info-level logging, PostgreSQL database, strict security
- **Security**: Strict validation and secure defaults
- **Monitoring**: Production monitoring and health checks

#### Cursor Environment (`env.cursor`)
- **Purpose**: Minimal configuration for Cursor IDE background agents
- **Features**: Warning-level logging, minimal resource usage
- **Security**: Cursor-managed security model
- **Monitoring**: No external monitoring (Cursor manages lifecycle)

## 📋 Usage Instructions

### Initial Setup
```bash
# Copy environment templates to working files
cp config/environments/env.dev .env.dev
cp config/environments/env.prod .env.prod
cp config/environments/env.cursor .env.cursor

# Edit with your specific values
nano .env.dev
nano .env.prod
nano .env.cursor
```

### Docker Environment Usage
```bash
# Development
docker-compose -f docker-compose.dev.yml --env-file .env.dev up -d

# Production
docker-compose -f docker-compose.prod.yml --env-file .env.prod up -d

# Cursor
docker-compose -f docker-compose.cursor.yml --env-file .env.cursor up
```

### Direct Application Usage
```bash
# Development
export $(cat .env.dev | xargs)
cargo run --bin toka-orchestration

# Production
export $(cat .env.prod | xargs)
cargo run --release --bin toka-orchestration
```

## 🔒 Configuration Standards

### Security Requirements
- **Secrets Management**: Never commit actual secrets to version control
- **Environment Separation**: Use different configurations for different environments
- **Secure Defaults**: Start with secure default values
- **Validation**: Validate configuration before deployment

### Configuration Structure
```bash
# Database Configuration
DATABASE_URL=sqlite:///app/data/agents_dev.db
STORAGE_TYPE=sqlite

# LLM Configuration
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=your_api_key_here
LLM_MODEL=claude-3-5-sonnet-20241022

# Agent Settings
AGENT_POOL_SIZE=5
MAX_CONCURRENT_AGENTS=3
AGENT_SPAWN_TIMEOUT=60

# Performance Settings
RUST_LOG=debug
RUST_BACKTRACE=1
TOKIO_WORKER_THREADS=2

# Security Settings
JWT_SECRET=your-secure-jwt-secret
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict
```

### Required Variables
- **LLM Configuration**: API keys and model settings
- **Database Configuration**: Connection strings and storage type
- **Agent Settings**: Pool size, timeouts, and limits
- **Security Settings**: JWT secrets and validation levels
- **Performance Settings**: Logging levels and resource limits

## 🔧 Configuration Management

### Environment-Specific Overrides
```bash
# Development overrides
RUST_LOG=debug
AGENT_SANDBOX_ENABLED=false
CAPABILITY_VALIDATION=permissive

# Production overrides
RUST_LOG=info
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict
```

### Validation Scripts
```bash
# Validate configuration
./scripts/validation/validate_env.sh .env.dev

# Check for missing required variables
./scripts/validation/check_required_vars.sh .env.prod
```

## 📊 Configuration Metrics

### Development Environment
- **Logging**: Debug level with backtraces
- **Database**: SQLite for simplicity
- **Security**: Permissive for development
- **Performance**: Optimized for debugging

### Production Environment
- **Logging**: Info level for performance
- **Database**: PostgreSQL for scalability
- **Security**: Strict validation and sandboxing
- **Performance**: Optimized for production workloads

### Cursor Environment
- **Logging**: Warning level for minimal overhead
- **Database**: None (Cursor manages state)
- **Security**: Cursor-managed security model
- **Performance**: Minimal resource usage

## 🔗 Related Documentation

- [Operations Guide](../../docs/operations/) - Deployment and monitoring
- [Development Guide](../../docs/development/) - Development workflows
- [Docker Environments](../../docs/operations/README-Docker-Environments.md) - Docker configuration

---

*This configuration directory is maintained as part of the Toka project's commitment to clear, accurate, and well-organized system configuration.* 