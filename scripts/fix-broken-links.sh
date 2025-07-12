#!/bin/bash
# fix-broken-links.sh
# Automated script to fix broken documentation links in Toka workspace

set -e

echo "🔧 Fixing broken documentation links..."

# Phase 1: Critical missing files
echo "📝 Creating missing root-level files..."

# Create CONTRIBUTING.md symlink
if [ ! -f "CONTRIBUTING.md" ]; then
    ln -sf docs/development/CONTRIBUTING.md CONTRIBUTING.md
    echo "✅ Created CONTRIBUTING.md symlink"
fi

# Create TESTS.md
cat > docs/TESTS.md << 'EOF'
# Toka Testing Guide

## Overview
Comprehensive testing strategies for the Toka workspace.

## Test Categories
- **Unit Tests**: `cargo test --lib` - Test individual modules and functions
- **Integration Tests**: `cargo test --test` - Test component interactions  
- **End-to-End Tests**: `cargo test --package toka-orchestration` - Test full workflows
- **Performance Tests**: `cargo bench` - Benchmark critical paths

## Coverage Reports
```bash
# Generate HTML coverage report
cargo tarpaulin --workspace --out Html --output-dir target/coverage

# View coverage in browser
open target/coverage/tarpaulin-report.html
```

## Test Configuration
- Test environments in `config/testing/`
- Test data in `tests/data/`
- Mock services in `tests/mocks/`

## Running Specific Tests
```bash
# Test specific crate
cargo test --package toka-agent-runtime

# Test with logging
RUST_LOG=debug cargo test

# Test with specific features
cargo test --features llm-integration
```

## Test Best Practices
- Use descriptive test names
- Test both success and failure cases
- Mock external dependencies
- Use property-based testing for complex logic
- Maintain test data integrity
EOF

echo "✅ Created docs/TESTS.md"

# Phase 2: Create missing directories
echo "📁 Creating missing documentation directories..."

# Create security documentation
mkdir -p docs/security
cat > docs/security/README.md << 'EOF'
# Toka Security Model

## Capability-Based Security
- **Token-based access control**: JWT tokens with specific permissions
- **Least privilege principle**: Agents only get required capabilities
- **Runtime enforcement**: Kernel validates all operations
- **Audit logging**: All security events are logged

## Security Components

### 1. JWT Token Validation
- HS256 algorithm for internal tokens
- Configurable token expiration
- Replay protection with JTI claims
- Secure token storage and transmission

### 2. Rate Limiting
- Per-agent request limits
- Global system rate limits
- Configurable thresholds
- Automatic backoff mechanisms

### 3. Input Sanitization
- LLM input validation
- File path sanitization
- Command injection prevention
- SQL injection protection

### 4. Resource Constraints
- Memory limits per agent
- CPU usage monitoring
- Timeout enforcement
- Disk space quotas

## Security Best Practices
- Regular security audits
- Dependency vulnerability scanning
- Secure configuration management
- Incident response procedures
EOF

echo "✅ Created docs/security/README.md"

# Create testing documentation
mkdir -p docs/testing
cat > docs/testing/README.md << 'EOF'
# Toka Testing Infrastructure

## Test Framework
- **Rust testing with cargo**: Built-in test framework
- **Property-based testing with proptest**: Automated test case generation
- **Integration test suites**: Cross-component testing
- **Performance benchmarks**: Criterion-based benchmarking

## Test Environments

### Development Testing
- Local development environment
- Mock services for external dependencies
- Fast feedback loop
- Debug-friendly configuration

### CI/CD Pipeline Testing
- GitHub Actions workflows
- Automated test execution
- Coverage reporting
- Security scanning

### Production Validation
- Smoke tests in production
- Health checks and monitoring
- Performance regression testing
- Error rate monitoring

## Test Categories

### Unit Tests
- Individual function testing
- Module isolation
- Fast execution
- High coverage

### Integration Tests
- Component interaction testing
- Database integration
- API endpoint testing
- Service communication

### End-to-End Tests
- Full workflow testing
- User journey validation
- System integration
- Performance validation

## Test Data Management
- Test fixtures in `tests/fixtures/`
- Mock data generation
- Database seeding
- Clean test isolation
EOF

echo "✅ Created docs/testing/README.md"

# Create troubleshooting guide
mkdir -p docs/troubleshooting
cat > docs/troubleshooting/README.md << 'EOF'
# Toka Troubleshooting Guide

## Common Issues

### 1. Build Failures
**Symptoms**: Compilation errors, dependency conflicts
**Solutions**:
- Check Rust version: `rustc --version`
- Update dependencies: `cargo update`
- Clean build: `cargo clean && cargo build`
- Check for circular dependencies

### 2. Environment Setup
**Symptoms**: Missing API keys, configuration errors
**Solutions**:
- Verify API keys in environment variables
- Check configuration files in `config/`
- Validate environment-specific settings
- Review setup documentation

### 3. Agent Orchestration Issues
**Symptoms**: Agents not starting, coordination failures
**Solutions**:
- Check agent configurations in `agents-specs/`
- Verify dependency resolution
- Review orchestration logs
- Validate capability permissions

### 4. LLM Integration Problems
**Symptoms**: API failures, rate limiting, timeouts
**Solutions**:
- Validate API credentials
- Check rate limit settings
- Review LLM gateway configuration
- Monitor token usage

## Diagnostic Commands

```bash
# Check system health
cargo check --workspace

# Run diagnostics
cargo test --package toka-orchestration test_health_check

# View logs
tail -f logs/toka-orchestration.log

# Debug specific component
RUST_LOG=debug cargo run --bin toka-orchestration-service

# Check dependencies
cargo tree --duplicates
```

## Performance Issues

### Memory Leaks
- Use `valgrind` for memory analysis
- Monitor heap usage with `heaptrack`
- Check for circular references
- Review resource cleanup

### CPU Usage
- Profile with `perf` or `cargo flamegraph`
- Identify hot paths
- Optimize algorithms
- Consider async optimizations

## Getting Help
- Check documentation in `docs/`
- Review GitHub issues
- Join community discussions
- Contact maintainers
EOF

echo "✅ Created docs/troubleshooting/README.md"

# Phase 3: Fix relative paths
echo "🔗 Fixing relative path issues..."

# Fix README-Orchestration.md reference
if [ -f "docs/agents/README.md" ]; then
    sed -i.bak 's|README-Orchestration.md](../../README-Orchestration.md)|README-Orchestration.md](README-Orchestration.md)|g' docs/agents/README.md
    echo "✅ Fixed README-Orchestration.md reference"
fi

# Fix README-Docker.md reference
if [ -f "docs/operations/README.md" ]; then
    sed -i.bak 's|README-Docker.md](../../README-Docker.md)|README-Docker.md](README-Docker.md)|g' docs/operations/README.md
    echo "✅ Fixed README-Docker.md reference"
fi

# Fix other relative path issues
find docs -name "*.md" -exec sed -i.bak 's|](../../README-|](README-|g' {} \;
echo "✅ Fixed relative path issues"

# Update toka-toolkit-core references to toka-tools
find docs -name "*.md" -exec sed -i.bak 's|toka-toolkit-core|toka-tools|g' {} \;
echo "✅ Updated crate references"

# Phase 4: Create agent documentation
echo "🤖 Creating agent documentation..."
mkdir -p agents/v0.3.0
cat > agents/v0.3.0/README.md << 'EOF'
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
EOF

echo "✅ Created agents/v0.3.0/README.md"

# Clean up backup files
find docs -name "*.bak" -delete
find agents -name "*.bak" -delete 2>/dev/null || true

echo ""
echo "✅ Broken link fixes completed!"
echo "📊 Summary:"
echo "   - Created CONTRIBUTING.md symlink"
echo "   - Created docs/TESTS.md"
echo "   - Created docs/security/ directory and README"
echo "   - Created docs/testing/ directory and README"
echo "   - Created docs/troubleshooting/ directory and README"
echo "   - Fixed relative path issues"
echo "   - Updated crate references"
echo "   - Created agents/v0.3.0/README.md"
echo ""
echo "🔍 Next steps:"
echo "   1. Review created files"
echo "   2. Run link validation"
echo "   3. Update CI/CD pipeline"
echo "   4. Test documentation navigation" 