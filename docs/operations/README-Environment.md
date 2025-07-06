# Toka Agent Development Environment

This environment is configured for efficient development and testing of Toka background agents within Cursor's containerized environment.

## Overview

The environment provides:
- **Multi-agent orchestration** with ownership management
- **Real-time monitoring** and metrics collection
- **Cursor-specific optimizations** for background agent workflows
- **Comprehensive tooling** for development and debugging

## Environment Configuration

### Core Services

| Service | Port | Purpose |
|---------|------|---------|
| Agent Orchestration API | 8080 | Main agent management and coordination |
| Agent Communication | 9000 | Inter-agent messaging and coordination |
| Cursor WebSocket | 9001 | Real-time cursor integration |
| Prometheus Metrics | 9090 | Performance monitoring and metrics |
| Grafana Dashboard | 3001 | Visualization and analytics |
| Redis Cache | 6379 | Caching and session management |
| Health Check | 8081 | Service health monitoring |

### Background Services

The environment automatically starts these supporting services:

- **Prometheus**: Metrics collection and storage
- **Grafana**: Metrics visualization and dashboards
- **Redis**: Caching and session management

## Agent Types

### Cursor-Specific Agents

1. **cursor-code-assistant** (Critical Priority)
   - Code completion and review
   - Refactoring and debugging assistance
   - LSP integration and file operations

2. **cursor-git-assistant** (High Priority)
   - Git operations and branch management
   - Merge conflict resolution
   - Repository insights and analysis

3. **cursor-file-manager** (High Priority)
   - File operations and directory management
   - Search and indexing capabilities
   - Backup and synchronization

4. **cursor-performance-monitor** (Medium Priority)
   - Performance monitoring and optimization
   - Resource tracking and analysis
   - Automatic optimization suggestions

## Usage

### Starting the Environment

1. **Automatic Setup**: The environment automatically builds and starts all services
2. **Manual Start**: Use the "Cursor Agent Orchestrator" terminal
3. **Monitoring**: Use the "Agent Monitor" terminal for real-time status

### Development Workflow

1. **Code Changes**: Edit code in the main workspace
2. **Testing**: Use "Build & Test" terminal for validation
3. **Monitoring**: Check "Agent Logs" for execution details
4. **Health**: Use "Agent Health Check" for service status

### Key Features

#### Ownership Management
- **Exclusive Access**: Critical agents require exclusive resource access
- **Conflict Resolution**: Automatic handling of resource conflicts
- **Session Isolation**: Secure separation between agent sessions

#### Performance Optimization
- **Resource Limits**: Memory and CPU constraints for stability
- **Caching Strategy**: Multi-level caching for performance
- **Connection Pooling**: Efficient resource utilization

#### Security Features
- **Sandbox Mode**: Isolated agent execution
- **Capability Validation**: Strict permission checking
- **File Access Control**: Restricted file system access

## Configuration Files

### Primary Configuration
- `config/cursor-agents.toml`: Main agent orchestration configuration
- `config/agents.toml`: Standard agent configuration
- `config/prometheus.yml`: Metrics collection setup

### Environment Variables
Key environment variables are automatically set:
- `DATABASE_URL`: SQLite database for agent state
- `LLM_PROVIDER`: Anthropic for AI capabilities
- `AGENT_POOL_SIZE`: 15 concurrent agents
- `RUST_LOG`: Info-level logging

## Monitoring and Debugging

### Metrics Dashboard
- **URL**: http://localhost:3001 (Grafana)
- **Default Credentials**: admin/admin
- **Key Metrics**: Agent performance, resource usage, error rates

### Logs and Debugging
- **Agent Logs**: Real-time execution logs
- **Health Checks**: Service status monitoring
- **Error Tracking**: Comprehensive error reporting

### Performance Monitoring
- **Resource Usage**: CPU, memory, and disk monitoring
- **Agent Metrics**: Execution time, success rates, queue depth
- **System Health**: Overall system performance and stability

## Troubleshooting

### Common Issues

1. **Agent Startup Failures**
   - Check "Agent Health Check" terminal
   - Verify configuration files are present
   - Ensure all required services are running

2. **Performance Issues**
   - Monitor resource usage in Grafana
   - Check agent pool size and limits
   - Review caching configuration

3. **Connection Problems**
   - Verify all ports are accessible
   - Check Docker service status
   - Review network configuration

### Debug Commands

```bash
# Check service status
docker-compose ps

# View agent logs
tail -f logs/agents.log

# Test health endpoint
curl -f http://localhost:8080/health

# Check metrics
curl http://localhost:9090/metrics
```

## Development Best Practices

### Agent Development
1. **Test Locally**: Use the development shell for testing
2. **Monitor Performance**: Use Grafana dashboards
3. **Validate Changes**: Run tests before deployment
4. **Check Logs**: Monitor execution logs for issues

### Configuration Management
1. **Version Control**: Keep configurations in version control
2. **Environment Separation**: Use different configs for dev/prod
3. **Validation**: Validate configurations before deployment
4. **Documentation**: Document configuration changes

### Performance Optimization
1. **Resource Monitoring**: Regularly check resource usage
2. **Caching Strategy**: Optimize cache settings
3. **Connection Pooling**: Monitor connection usage
4. **Error Handling**: Implement robust error handling

## Security Considerations

### Agent Isolation
- Each agent runs in its own sandbox
- File system access is restricted
- Network access is controlled

### Data Protection
- Sensitive data is encrypted at rest
- API keys are securely managed
- Session data is isolated

### Access Control
- Ownership validation prevents conflicts
- Capability-based access control
- Audit logging for all operations

## Support and Maintenance

### Regular Maintenance
- **Log Rotation**: Automatic log management
- **Database Cleanup**: Periodic cleanup of old data
- **Cache Invalidation**: Automatic cache refresh
- **Health Monitoring**: Continuous health checks

### Updates and Upgrades
- **Configuration Updates**: Version-controlled configs
- **Agent Updates**: Rolling updates for agents
- **Security Patches**: Regular security updates
- **Performance Tuning**: Continuous optimization

For additional support or questions, refer to the main project documentation or create an issue in the repository. 