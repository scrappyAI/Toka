# Toka Agent Configuration Guide

## Overview
This guide covers the configuration and deployment of Toka agents in the v0.3.0 orchestration system.

## Agent Specifications
Agent configurations are defined in YAML files located in `agents-specs/v0.3/workstreams/`:

- `build-system-stabilization.yaml` - Dependency management and build validation
- `testing-infrastructure.yaml` - Automated testing and quality assurance
- `kernel-events-enhancement.yaml` - Kernel event system improvements
- `storage-advancement.yaml` - Storage layer enhancements
- `security-extension.yaml` - Security hardening and compliance
- `performance-observability.yaml` - Performance monitoring and optimization
- `github-cicd-issues-resolution.yaml` - CI/CD pipeline improvements
- `document-organization.yaml` - Documentation management
- `llm-credentials-setup.yaml` - LLM provider configuration

## Configuration Format
```yaml
metadata:
  name: "agent-name"
  version: "v0.3.0"
  workstream: "workstream-name"
  branch: "feature/workstream-name"
  
spec:
  name: "Human-readable agent name"
  domain: "infrastructure|development|security"
  priority: "critical|high|medium|low"
  
capabilities:
  primary: ["capability1", "capability2"]
  secondary: ["capability3", "capability4"]
  
objectives:
  - description: "Objective description"
    deliverable: "Expected deliverable"
    validation: "Validation criteria"
    
tasks:
  default:
    - description: "Task description"
      priority: "high|medium|low"
      
security:
  sandbox: true
  capabilities_required: ["filesystem-read", "filesystem-write"]
  resource_limits:
    max_memory: "512MB"
    max_cpu: "75%"
    timeout: "2h"
```

## Deployment

### Start Orchestration Service
```bash
# Start the orchestration service
cargo run --bin toka-orchestration-service

# Start with specific configuration
cargo run --bin toka-orchestration-service -- --config config/agents.toml
```

### Deploy Specific Workstream
```bash
# Deploy single workstream
cargo run --bin toka-orchestration-service -- --workstream build-system-stabilization

# Deploy multiple workstreams
cargo run --bin toka-orchestration-service -- --workstream build-system-stabilization,testing-infrastructure
```

### Monitor Agent Status
```bash
# Check agent status
cargo run --bin toka-orchestration-service -- --status

# View agent logs
tail -f logs/agents/build-system-stabilization.log
```

## Agent Development

### Creating New Agents
1. Create agent specification YAML file
2. Define capabilities and objectives
3. Set security constraints
4. Configure resource limits
5. Test agent configuration

### Testing Agents
```bash
# Test agent configuration
cargo test --package toka-orchestration test_agent_config

# Test agent execution
cargo run --example agent_test -- --agent build-system-stabilization
```

## Troubleshooting
- Check agent logs in `logs/agents/`
- Verify configuration syntax
- Validate capability permissions
- Monitor resource usage
- Review dependency resolution
