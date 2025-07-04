# Toka Development Environment Configuration

This document describes the comprehensive development environment setup for Toka OS, optimized for Rust development with Cursor background agents and various development tools.

## 🏗️ Environment Structure

### `.cursor/` - Cursor IDE Configuration
- **`environment.json`**: Comprehensive environment configuration for Cursor background agents
  - Rust toolchain availability and version pinning
  - Environment variables for optimal Rust development
  - Agent capabilities and resource limits
  - Workspace-specific settings for the Toka project

### `.devcontainer/` - VS Code Dev Container
- **`devcontainer.json`**: Full-featured development container configuration
- **`Dockerfile`**: Enhanced Rust development environment with all necessary tools
- **`post-create.sh`**: Automated setup script for the development environment

### `.vscode/` - VS Code Settings
- **`settings.json`**: Comprehensive IDE configuration for optimal Rust development
  - Rust Analyzer settings for the best language server experience
  - Code formatting, linting, and debugging configuration
  - File associations and workspace optimizations

### `.cargo/` - Cargo Configuration
- **`config.toml`**: Enhanced Cargo configuration with:
  - Build optimizations for development and release
  - Custom aliases for common commands
  - Cross-platform compilation settings
  - Memory and performance optimizations

### `.github/workflows/` - CI/CD Pipelines
- **`ci.yml`**: Modern Rust CI pipeline with comprehensive quality checks
- **`cursor-agent-validation.yml`**: Specialized validation for development environments

### Root Configuration Files
- **`rust-toolchain.toml`**: Pinned Rust version and components for consistency

## 🦀 Rust Development Features

### Cursor Agent Support
- **Background Agent Capabilities**: Full Rust toolchain access for AI agents
- **Resource Management**: Configured memory and CPU limits for stable operation
- **Environment Variables**: Optimized settings for Rust compilation and debugging
- **Workspace Detection**: Automatic workspace member discovery and feature management

### Development Container
- **Pre-installed Tools**: Complete Rust toolchain + cargo extensions
- **Modern CLI Tools**: `ripgrep`, `fd`, `bat`, `exa` for enhanced development experience
- **Debugging Support**: `lldb`, `gdb`, `valgrind` for comprehensive debugging
- **Shell Aliases**: Convenient shortcuts for common Cargo commands

### VS Code Integration
- **Rust Analyzer**: Fully configured with optimal settings for large codebases
- **Code Actions**: Automatic formatting, import organization, and quick fixes
- **Testing Support**: Integrated test discovery and execution
- **Debugging**: Ready-to-use debugging configurations

## 🚀 Quick Start

### Using Dev Container
1. Open the project in VS Code
2. Click "Reopen in Container" when prompted
3. Wait for the container to build and the post-create script to run
4. Start coding with a fully configured Rust environment

### Using Cursor
1. Open the project in Cursor
2. Background agents will automatically have access to the Rust toolchain
3. Use Cursor's AI features with full Rust language support

### Manual Setup
1. Install Rust using the pinned version: `rustup toolchain install 1.86`
2. Install required components: `rustup component add clippy rustfmt rust-analyzer rust-src`
3. Configure your IDE using the provided `.vscode/settings.json`

## 🔧 Available Commands

### Cargo Aliases (via `.cargo/config.toml`)
```bash
# Basic commands
cargo b          # build
cargo c          # check  
cargo t          # test
cargo r          # run
cargo f          # fmt
cargo cl         # clippy

# Development workflows
cargo dev-test   # test --all --all-features
cargo dev-check  # check --all --all-features
cargo dev-build  # build --all --all-features
cargo dev-fmt    # fmt --all
cargo dev-clippy # clippy --all-targets --all-features -- -D warnings

# Workspace commands
cargo ws-build   # build --workspace
cargo ws-test    # test --workspace
cargo ws-check   # check --workspace
cargo ws-clean   # clean --workspace
```

### Shell Aliases (in Dev Container)
```bash
# Modern CLI tools
ll               # exa -la
lt               # exa --tree
cat              # bat
find             # fd
grep             # rg

# Cargo shortcuts
cb               # cargo build
ct               # cargo test
cc               # cargo check
cf               # cargo fmt
ccl              # cargo clippy
cw               # cargo watch
cr               # cargo run

# Useful functions
cargo-tree-deps          # Show dependency tree
cargo-build-release      # Release build
cargo-clean-target       # Clean target directory
```

## 🛠️ Environment Variables

### Common Across All Environments
- `RUST_BACKTRACE=1`: Enable backtraces for debugging
- `RUSTFLAGS="-C link-arg=-fuse-ld=lld"`: Use fast LLD linker
- `CARGO_INCREMENTAL=0`: Disable incremental compilation for consistency
- `CARGO_NET_RETRY=10`: Retry network operations
- `CARGO_NET_TIMEOUT=60`: Network timeout for cargo operations

### Development-Specific
- `RUST_LOG=info`: Default logging level for development
- `CARGO_TARGET_DIR=/tmp/target`: Use temporary directory for faster builds

## 🔍 Quality Assurance

### Pre-commit Hooks (Automatically Installed)
- Code formatting check (`cargo fmt --check`)
- Linting with Clippy (`cargo clippy`)
- Test execution (`cargo test`)

### CI/CD Pipeline Features
- **Multi-platform testing**: Linux and macOS
- **Multiple Rust versions**: Stable and beta
- **Security auditing**: Automated dependency security checks
- **Code coverage**: Comprehensive coverage reporting
- **Environment validation**: Ensures all configurations are valid and consistent

## 📊 Performance Optimizations

### Build Performance
- **Fast Linker**: LLD linker for faster builds
- **Reduced Parallelism**: Optimized for memory-constrained environments
- **Split Debug Info**: Reduces memory usage during compilation
- **Optimized Profiles**: Different settings for dev, test, and release builds

### IDE Performance
- **File Exclusions**: Ignores target directories and build artifacts
- **Rust Analyzer**: Configured for optimal performance on large codebases
- **Incremental Features**: Disabled to avoid cache-related issues

## 🔒 Security Features

### Resource Limits
- **Memory Limits**: 2GB maximum for Cursor agents
- **CPU Limits**: 2 cores maximum for background operations
- **Sandboxed Execution**: Controlled environment for agent operations

### Security Scanning
- **Cargo Audit**: Automated dependency vulnerability scanning
- **Clippy Security**: Security-focused linting rules
- **Supply Chain**: Verified toolchain and dependency management

## 🐛 Troubleshooting

### Common Issues

#### Build Errors
- **Out of Memory**: Reduce `jobs` in `.cargo/config.toml`
- **Linker Issues**: Ensure `lld` is installed (`sudo apt install lld`)
- **Permission Issues**: Check file permissions in dev container

#### IDE Issues
- **Rust Analyzer Not Working**: Restart the language server
- **Missing Completions**: Ensure `rust-src` component is installed
- **Slow Performance**: Check file exclusions in `.vscode/settings.json`

#### Agent Issues
- **Toolchain Not Available**: Verify `.cursor/environment.json` configuration
- **Memory Limits**: Adjust `maxMemoryMB` in agent capabilities
- **Permission Denied**: Check agent capabilities in Cursor configuration

### Getting Help
1. Check the CI/CD pipeline logs for build issues
2. Review the dev container post-create script output
3. Validate environment consistency using the validation workflow
4. Consult the individual configuration files for specific settings

## 🎯 Next Steps

### For Developers
1. Familiarize yourself with the available aliases and shortcuts
2. Configure your personal Git settings in the dev container
3. Explore the pre-installed development tools
4. Use the CI/CD pipeline to validate your changes

### For Cursor Users
1. Test background agent capabilities with Rust code generation
2. Utilize the configured environment variables for optimal AI assistance
3. Leverage the workspace detection for multi-crate operations

### For Maintainers
1. Keep Rust toolchain version updated in `rust-toolchain.toml`
2. Monitor CI/CD pipeline performance and adjust as needed
3. Update development container base image regularly
4. Review and update environment validation workflows

---

This development environment provides a comprehensive, consistent, and optimized setup for Rust development with full support for Cursor background agents and modern development workflows. 