# Toka Cursor Development Environment

A comprehensive, containerized development environment for the Toka operating system with full toolchain support, security scanning, fuzzing, monitoring, and collaborative coding capabilities.

## 🚀 Quick Start

### Prerequisites

- **Cursor IDE** with environment support
- **Docker Engine** 20.10+
- **Docker Compose** 2.0+
- **Git** for version control
- **4GB+ RAM** available for containers

### 1. Environment Setup

Create your environment file:

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
# OR
OPENAI_API_KEY=your_openai_api_key_here

# Database Passwords
POSTGRES_PASSWORD=secure_postgres_password
REDIS_PASSWORD=secure_redis_password

# Monitoring
GRAFANA_PASSWORD=secure_grafana_password
```

### 2. Launch Development Environment

```bash
# Using Docker Compose
cd .cursor
docker-compose up -d

# Or using Cursor's built-in environment support
# Cursor will automatically detect and use environment.json
```

### 3. Access Services

- **Development Container**: `docker exec -it toka-cursor-dev bash`
- **Toka API**: http://localhost:8080
- **Agent Management**: http://localhost:9000
- **WebSocket**: ws://localhost:9001
- **Grafana Dashboard**: http://localhost:3001 (admin/yourpassword)
- **Prometheus**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686
- **Redis**: localhost:6379
- **PostgreSQL**: localhost:5432

## 🛠️ Development Tools

### Rust Toolchain

- **Rust 1.86** (pinned for consistency)
- **Clippy** for linting
- **Rustfmt** for formatting
- **Rust Analyzer** for IDE support

### Cargo Tools

```bash
# Available tools (pre-installed)
cargo-edit          # Add/remove dependencies
cargo-watch         # Watch for changes
cargo-audit         # Security auditing
cargo-deny          # Dependency policy enforcement
cargo-tarpaulin     # Code coverage
cargo-fuzz          # Fuzzing
cargo-nextest       # Next-gen test runner
cargo-udeps         # Unused dependencies
cargo-expand        # Macro expansion
cargo-tree          # Dependency tree
cargo-machete       # Dead code detection
cargo-outdated      # Outdated dependencies
cargo-bloat         # Binary size analysis
cargo-flamegraph    # Performance profiling
cargo-llvm-cov      # LLVM code coverage
```

### Security Tools

```bash
# Security scanning
cargo audit         # Vulnerability scanning
cargo deny check    # Policy enforcement
semgrep --config=auto .  # Static analysis
bandit -r .         # Python security (if applicable)

# Fuzzing
cargo fuzz list     # List fuzz targets
cargo fuzz run target  # Run fuzzer
```

### Testing & Coverage

```bash
# Run tests
cargo test --workspace
cargo nextest run   # Faster test execution

# Coverage analysis
cargo tarpaulin --workspace --out Html --out Json
cargo llvm-cov --workspace --html
```

### Performance & Profiling

```bash
# Performance profiling
cargo flamegraph --bin toka-cli
perf record -g cargo bench
heaptrack cargo test

# Binary analysis
cargo bloat --release
cargo deps --all-deps
```

## 📊 Monitoring & Observability

### Metrics Collection

The environment includes a complete monitoring stack:

- **Prometheus** for metrics collection
- **Grafana** for visualization
- **Jaeger** for distributed tracing
- **Redis** for caching and session management

### Health Checks

```bash
# Container health
docker-compose ps
docker-compose logs -f toka-dev

# Application health
curl http://localhost:8080/health
/app/.cursor/scripts/cursor-health-check.sh
```

### Performance Monitoring

```bash
# Build performance
cargo build --workspace --timings

# Test performance
cargo test --workspace --timings

# Runtime monitoring
htop
docker stats
```

## 🔒 Security Features

### Built-in Security

- **Non-root user execution** (vscode user)
- **Capability-based security** model
- **Sandboxed execution** environment
- **Network isolation** between services
- **Secure defaults** for all tools

### Security Scanning

```bash
# Comprehensive security audit
./security-audit.sh

# Manual security checks
cargo audit
cargo deny check
semgrep --config=auto .
```

### Fuzzing

```bash
# List available fuzz targets
cargo fuzz list

# Run fuzzing
cargo fuzz run <target>

# AFL++ fuzzing
afl-fuzz -i input -o output target
```

## 🤝 Collaborative Development

### GitHub Integration

```bash
# Set up GitHub authentication
git config --global user.name "Your Name"
git config --global user.email "your@email.com"

# GitHub CLI (gh) is available for advanced workflows
gh auth login
gh repo clone your-org/your-repo
gh pr create --title "Feature: New capability"
```

### Code Quality

```bash
# Pre-commit hooks (automatic)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Manual quality checks
cargo machete    # Dead code detection
cargo udeps      # Unused dependencies
cargo outdated   # Outdated dependencies
```

## 📝 Common Workflows

### Development Workflow

```bash
# 1. Start development environment
docker-compose up -d

# 2. Enter development container
docker exec -it toka-cursor-dev bash

# 3. Make changes and test
cargo check --workspace
cargo test --workspace

# 4. Run quality checks
cargo fmt --all
cargo clippy --workspace --all-targets

# 5. Security audit
cargo audit
cargo deny check

# 6. Commit changes
git add .
git commit -m "feat: add new capability"
git push origin feature-branch
```

### Agent Development

```bash
# 1. Create new agent
toka spawn-agent --name my-agent --token $(toka generate-token)

# 2. Test agent functionality
toka schedule-task --agent 123 --description "Test task"

# 3. Monitor agent execution
toka query-state
curl http://localhost:9000/agents/status
```

### Performance Testing

```bash
# 1. Build release version
cargo build --workspace --release

# 2. Run benchmarks
cargo bench --workspace

# 3. Profile performance
cargo flamegraph --bin toka-cli

# 4. Analyze results
firefox flamegraph.svg
```

## 🐛 Debugging & Troubleshooting

### Common Issues

**Container won't start:**
```bash
# Check logs
docker-compose logs toka-dev

# Rebuild if needed
docker-compose build --no-cache
```

**Build failures:**
```bash
# Clean build cache
cargo clean
rm -rf target/

# Check dependencies
cargo tree
cargo outdated
```

**Test failures:**
```bash
# Run specific test
cargo test --package toka-core --test integration_test

# Debug with full backtrace
RUST_BACKTRACE=full cargo test failing_test
```

### Debugging Tools

```bash
# GDB debugging
gdb target/debug/toka-cli
(gdb) run --help

# LLDB debugging
lldb target/debug/toka-cli
(lldb) run --help

# Valgrind memory analysis
valgrind --leak-check=full ./target/debug/toka-cli
```

## 📚 Resources

### Documentation

- [Toka Architecture](../docs/architecture/README.md)
- [Development Guide](../docs/development/README.md)
- [Security Model](../docs/security/README.md)
- [Testing Guide](../docs/testing/README.md)

### External Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Security Best Practices](https://anssi-fr.github.io/rust-guide/)

## 🔧 Customization

### Environment Variables

Edit `.env` to customize:

```env
# Rust compilation
RUSTFLAGS="-D warnings -C target-cpu=native"
CARGO_BUILD_JOBS=4

# Development mode
TOKA_ENV=development
TOKA_LOG_LEVEL=debug
RUST_BACKTRACE=1

# Resource limits
CARGO_BUILD_JOBS=4
MAKEFLAGS="-j4"
```

### Docker Customization

Modify `.cursor/Dockerfile` to add tools:

```dockerfile
# Add custom tools
RUN cargo install your-custom-tool

# Add system packages
RUN apt-get update && apt-get install -y \
    your-package \
    && rm -rf /var/lib/apt/lists/*
```

### VS Code Settings

Customize `.cursor/environment.json` for VS Code integration:

```json
{
  "customizations": {
    "vscode": {
      "extensions": [
        "your-custom-extension"
      ],
      "settings": {
        "your.custom.setting": "value"
      }
    }
  }
}
```

## 🚀 Advanced Usage

### Multi-Stage Development

```bash
# Development stage
docker-compose up -d

# Testing stage
docker-compose -f docker-compose.yml -f docker-compose.test.yml up

# Production simulation
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up
```

### Container Orchestration

```bash
# Scale services
docker-compose up -d --scale toka-dev=3

# Update services
docker-compose pull
docker-compose up -d

# Backup data
docker-compose exec postgres pg_dump -U toka toka > backup.sql
```

---

**Happy coding with Toka! 🎉**

For issues or questions, please check the [troubleshooting guide](../docs/troubleshooting.md) or open an issue on GitHub. 