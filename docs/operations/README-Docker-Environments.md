# Toka Docker Environment Configuration

This document describes the new Docker environment separation for Toka, providing different configurations for development, production, and Cursor background agents.

## Overview

The Docker setup has been reconfigured to separate environments and optimize for different use cases:

- **Development**: Full debugging tools, hot reloading, and development-friendly settings
- **Production**: Security hardening, performance optimization, and production-ready configuration
- **Cursor**: Minimal setup for Cursor IDE background agents with no networking or orchestration

## File Structure

```
├── Dockerfile              # Original Dockerfile (kept for backward compatibility)
├── Dockerfile.dev          # Development-focused Dockerfile
├── Dockerfile.prod         # Production-optimized Dockerfile
├── Dockerfile.cursor       # Minimal Dockerfile for Cursor agents
├── docker-compose.yml      # Base configuration (minimal)
├── docker-compose.dev.yml  # Development environment
├── docker-compose.prod.yml # Production environment
├── docker-compose.cursor.yml # Cursor background agents
├── env.dev                 # Development environment variables
├── env.prod                # Production environment variables
└── env.cursor              # Cursor environment variables
```

## Environment-Specific Dockerfiles

### Dockerfile.dev
- **Purpose**: Development with debugging and hot reloading
- **Features**:
  - Development tools (vim, htop, procps)
  - Cargo watch for hot reloading
  - Debug logging enabled
  - Source code mounting for live development
  - Incremental compilation
  - Development-friendly timeouts and limits

### Dockerfile.prod
- **Purpose**: Production deployment with security and performance
- **Features**:
  - Multi-stage build for minimal runtime image
  - Security hardening (non-root user, secure memory)
  - Optimized for performance
  - Stripped binaries
  - Production logging levels
  - Resource limits and health checks

### Dockerfile.cursor
- **Purpose**: Minimal setup for Cursor IDE background agents
- **Features**:
  - Ultra-minimal runtime (only essential dependencies)
  - No networking or orchestration
  - Minimal resource usage (512MB memory limit)
  - No health checks or init system
  - Cursor-managed lifecycle
  - Only builds the `toka-cli` binary

## Environment-Specific Compose Files

### docker-compose.dev.yml
- **Use Case**: Development and testing
- **Features**:
  - Hot reloading with source code mounting
  - Debug logging and backtraces
  - Development-friendly timeouts
  - Separate ports (8080, 9091, 3002, 6380)
  - Permissive security settings
  - Development monitoring stack

### docker-compose.prod.yml
- **Use Case**: Production deployment
- **Features**:
  - PostgreSQL database
  - Production monitoring stack
  - Resource limits and reservations
  - Strict security settings
  - Production ports (8080, 9090, 3001, 5432, 6379)
  - Health checks and restart policies

### docker-compose.cursor.yml
- **Use Case**: Cursor IDE background agents
- **Features**:
  - No external ports (Cursor manages communication)
  - Minimal resource usage
  - No health checks or networking
  - Workspace mounting for agent access
  - Cursor-specific environment variables

## Environment Variables

### Development (env.dev)
- Debug logging enabled
- Development-friendly timeouts
- SQLite database
- Permissive security settings
- Development monitoring

### Production (env.prod)
- Info-level logging
- Production timeouts and limits
- PostgreSQL database
- Strict security settings
- Production monitoring

### Cursor (env.cursor)
- Warning-level logging
- Minimal resource usage
- No database or networking
- Cursor-specific optimizations

## Usage Examples

### Development Environment
```bash
# Copy environment file
cp env.dev .env.dev

# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop development environment
docker-compose -f docker-compose.dev.yml down
```

### Production Environment
```bash
# Copy and configure environment file
cp env.prod .env.prod
# Edit .env.prod with your production values

# Start production environment
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose -f docker-compose.prod.yml logs -f

# Stop production environment
docker-compose -f docker-compose.prod.yml down
```

### Cursor Background Agents
```bash
# Copy environment file
cp env.cursor .env.cursor

# Build cursor agent image
docker-compose -f docker-compose.cursor.yml build

# Run cursor agent (typically managed by Cursor IDE)
docker-compose -f docker-compose.cursor.yml up cursor-agent
```

## Port Mappings

### Development
- Toka Agents: 8080
- Prometheus: 9091
- Grafana: 3002
- Redis: 6380

### Production
- Toka Agents: 8080
- Prometheus: 9090
- Grafana: 3001
- PostgreSQL: 5432
- Redis: 6379

### Cursor
- No external ports (managed by Cursor IDE)

## Resource Requirements

### Development
- Memory: ~2GB
- CPU: 1-2 cores
- Storage: ~5GB

### Production
- Memory: 4GB (with limits)
- CPU: 2 cores (with limits)
- Storage: ~10GB

### Cursor
- Memory: 512MB (with limits)
- CPU: 0.5 cores (with limits)
- Storage: ~1GB

## Security Considerations

### Development
- Permissive security settings for debugging
- Debug logging enabled
- Source code mounted for hot reloading
- Development passwords and secrets

### Production
- Strict security validation
- Non-root user execution
- Secure memory handling
- Production secrets and keys
- Resource limits and health checks

### Cursor
- Minimal attack surface
- No external networking
- Cursor-managed security
- Minimal resource usage

## Monitoring and Logging

### Development
- Debug-level logging
- Development monitoring stack
- Hot reloading support
- Development-friendly timeouts

### Production
- Info-level logging
- Production monitoring stack
- Health checks and alerts
- Performance metrics

### Cursor
- Warning-level logging
- No external monitoring
- Cursor-managed lifecycle
- Minimal logging overhead

## Migration from Original Setup

If you're migrating from the original Docker setup:

1. **Backup your data**:
   ```bash
   docker-compose down
   docker volume ls
   # Backup relevant volumes
   ```

2. **Choose your environment**:
   - For development: Use `docker-compose.dev.yml`
   - For production: Use `docker-compose.prod.yml`
   - For Cursor: Use `docker-compose.cursor.yml`

3. **Update environment variables**:
   - Copy the appropriate env file
   - Configure with your values
   - Update any custom configurations

4. **Start the new environment**:
   ```bash
   # Development
   docker-compose -f docker-compose.dev.yml up -d
   
   # Production
   docker-compose -f docker-compose.prod.yml up -d
   
   # Cursor
   docker-compose -f docker-compose.cursor.yml up
   ```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Each environment uses different ports to avoid conflicts
2. **Environment variables**: Ensure you've copied and configured the appropriate env file
3. **Resource limits**: Cursor agents have strict resource limits
4. **Network isolation**: Each environment uses separate networks

### Debugging

- **Development**: Use `docker-compose -f docker-compose.dev.yml logs -f`
- **Production**: Use `docker-compose -f docker-compose.prod.yml logs -f`
- **Cursor**: Use `docker-compose -f docker-compose.cursor.yml logs cursor-agent`

### Performance Tuning

- **Development**: Adjust timeouts and limits in `env.dev`
- **Production**: Modify resource limits in `docker-compose.prod.yml`
- **Cursor**: Minimal tuning needed due to resource constraints

## Best Practices

1. **Environment separation**: Use different environments for different purposes
2. **Security**: Always use production environment for production deployments
3. **Resource management**: Monitor resource usage and adjust limits as needed
4. **Backup**: Regularly backup your data volumes
5. **Updates**: Keep base images and dependencies updated
6. **Documentation**: Document any custom configurations

## Support

For issues or questions:
1. Check the logs for error messages
2. Verify environment variable configuration
3. Ensure ports are not in use by other services
4. Check resource usage and limits
5. Review the troubleshooting section above 