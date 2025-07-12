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
