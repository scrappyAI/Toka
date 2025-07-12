#!/bin/bash

# Post-create script for Toka Rust development environment
set -e

echo "🦀 Setting up Toka Rust development environment..."

# Configure network resilience
echo "🌐 Configuring network resilience..."
# Ensure DNS resolution works
if ! nslookup index.crates.io > /dev/null 2>&1; then
    echo "⚠️  DNS resolution issues detected, configuring alternative DNS..."
    echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf > /dev/null
    echo "nameserver 8.8.4.4" | sudo tee -a /etc/resolv.conf > /dev/null
fi

# Update package lists (packages are already installed in Dockerfile)
echo "🔧 Updating package lists..."
sudo apt update

# Ensure cargo registry is accessible
mkdir -p ~/.cargo
echo "Creating target directory for faster builds..."
sudo mkdir -p /tmp/target
sudo chmod 755 /tmp/target
sudo chown vscode:vscode /tmp/target

# Update Rust toolchain to latest stable
echo "📦 Updating Rust toolchain..."
rustup update stable
rustup default stable

# Verify Rust installation
echo "🔍 Verifying Rust installation..."
rustc --version
cargo --version
rustfmt --version
clippy-driver --version

# Test cargo tools installation
echo "🔧 Testing cargo tools installation..."
if ! command -v cargo-outdated > /dev/null 2>&1 || ! command -v cargo-tree > /dev/null 2>&1; then
    echo "⚠️  Some cargo tools missing, running fallback installation..."
    if [ -f ".devcontainer/install-cargo-tools.sh" ]; then
        chmod +x .devcontainer/install-cargo-tools.sh
        bash .devcontainer/install-cargo-tools.sh
    else
        echo "⚠️  Fallback installation script not found"
    fi
fi

# Install Python dependencies if requirements.txt exists
if [ -f "requirements.txt" ]; then
    echo "🐍 Installing Python dependencies..."
    python3 -m pip install --user --upgrade pip
    python3 -m pip install --user -r requirements.txt
    echo "✅ Python dependencies installed"
fi

# Setup GitHub authentication
echo "🔐 Setting up GitHub authentication..."
if [ -f ".devcontainer/setup-github-auth.sh" ]; then
    chmod +x .devcontainer/setup-github-auth.sh
    bash .devcontainer/setup-github-auth.sh
else
    echo "⚠️  GitHub auth setup script not found, skipping GitHub authentication"
fi

# Run Toka testing setup if available
echo "⚙️  Running Toka development setup..."
if [ -f "scripts/setup/setup_toka_testing.sh" ]; then
    # Create a non-interactive version for container setup
    export TOKA_CONTAINER_MODE="true"
    export LLM_PROVIDER="${LLM_PROVIDER:-anthropic}"
    export LLM_MODEL="${LLM_MODEL:-claude-3-5-sonnet-20241022}"
    
    # Create basic environment configuration
    if [ ! -f ".env" ]; then
        echo "📝 Creating container environment configuration..."
        cat > .env << EOF
# Toka Container Development Environment
# Generated automatically for dev container

# =============================================================================
# LLM Provider Configuration (from environment variables)
# =============================================================================
EOF
        
        # Add LLM configuration if available
        if [[ -n "$ANTHROPIC_API_KEY" ]]; then
            echo "ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY" >> .env
        fi
        
        if [[ -n "$OPENAI_API_KEY" ]]; then
            echo "OPENAI_API_KEY=$OPENAI_API_KEY" >> .env
        fi
        
        if [[ -n "$LLM_PROVIDER" ]]; then
            cat >> .env << EOF
LLM_PROVIDER=$LLM_PROVIDER
LLM_MODEL=$LLM_MODEL
LLM_RATE_LIMIT=50
LLM_TIMEOUT=30
LLM_DEBUG=false
EOF
        fi
        
        cat >> .env << 'EOF'

# =============================================================================
# Database Configuration
# =============================================================================
DATABASE_URL=sqlite:///app/data/agents.db
STORAGE_TYPE=sqlite

# =============================================================================
# Agent Orchestration Settings
# =============================================================================
AGENT_POOL_SIZE=3
MAX_CONCURRENT_AGENTS=2
AGENT_SPAWN_TIMEOUT=30
WORKSTREAM_TIMEOUT=1800

# =============================================================================
# Development Settings
# =============================================================================
RUST_LOG=info
RUST_BACKTRACE=1
TOKIO_WORKER_THREADS=2

# Security
JWT_SECRET=toka-dev-container-secret
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict

# Monitoring
METRICS_ENABLED=true
TRACING_ENABLED=true
LOG_LEVEL=info

# Development directories
AGENT_DATA_DIR=./data
AGENT_LOG_DIR=./logs
AGENT_CONFIG_DIR=./config

# Container mode
AGENT_DEV_MODE=true
AGENT_DEBUG_ENABLED=true
TOKA_CONTAINER_MODE=true
TOKA_AUTH_METHOD=github
EOF
        echo "✅ Environment configuration created"
    fi
    
    # Create required directories
    mkdir -p data logs config/testing
    echo "✅ Required directories created"
    
else
    echo "⚠️  Toka setup script not found, creating basic configuration..."
    mkdir -p data logs config
fi

# Pre-fetch dependencies for faster first build
echo "⚡ Pre-fetching workspace dependencies..."
# Retry mechanism for cargo operations
for i in {1..3}; do
    if cargo fetch --locked; then
        echo "✅ Dependencies fetched successfully"
        break
    else
        echo "⚠️  Attempt $i failed, retrying..."
        sleep 5
    fi
done || echo "Warning: Could not pre-fetch dependencies after 3 attempts"

# Set up git if not already configured (GitHub auth might have done this)
if ! git config --global user.name > /dev/null 2>&1; then
    echo "🔧 Configuring git with placeholder values..."
    git config --global user.name "Toka Developer"
    git config --global user.email "dev@toka.local"
    git config --global init.defaultBranch main
fi

# Create useful aliases
echo "📝 Setting up shell aliases..."
cat >> ~/.bashrc << 'EOF'

# Toka Rust Development Aliases
alias ll='exa -la'
alias la='exa -la'
alias lt='exa --tree'
alias cat='bat'
alias find='fd'
alias grep='rg'

# Cargo shortcuts
alias cb='cargo build'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'
alias cw='cargo watch'
alias cr='cargo run'

# Workspace shortcuts
alias build-all='cargo build --all'
alias test-all='cargo test --all'
alias check-all='cargo check --all'
alias fmt-all='cargo fmt --all'
alias clippy-all='cargo clippy --all'

# Python shortcuts
alias py='python3'
alias pip='python3 -m pip'

# Toka shortcuts
alias toka='cargo run --bin toka --'
alias toka-cli='cargo run --bin toka-cli --'
alias toka-config='cargo run --bin toka-config --'

# Useful functions
function cargo-tree-deps() {
    cargo tree --depth 1 | grep -E "^\w"
}

function cargo-build-release() {
    cargo build --release --all
}

function cargo-clean-target() {
    echo "Cleaning target directory..."
    rm -rf /tmp/target/*
    echo "Target directory cleaned!"
}

function toka-quick-test() {
    echo "Running quick Toka workspace tests..."
    cargo check --workspace --all-features
    cargo test --workspace --lib
}

EOF

# Verify workspace structure
echo "🏗️  Verifying workspace structure..."
if [ -f "Cargo.toml" ]; then
    echo "✅ Found workspace Cargo.toml"
    if cargo metadata --no-deps --format-version 1 > /dev/null 2>&1; then
        echo "✅ Workspace metadata is valid"
    else
        echo "⚠️  Workspace metadata validation failed (this might be okay for initial setup)"
    fi
else
    echo "❌ No Cargo.toml found in workspace root"
fi

# Set up pre-commit hooks if .git exists
if [ -d ".git" ]; then
    echo "🪝 Setting up git hooks..."
    mkdir -p .git/hooks
    
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for Rust projects

echo "Running pre-commit checks..."

# Check formatting
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Run clippy
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed. Fix the warnings above."
    exit 1
fi

# Run tests
if ! cargo test --all; then
    echo "❌ Tests failed. Fix the failing tests."
    exit 1
fi

echo "✅ All pre-commit checks passed!"
EOF
    chmod +x .git/hooks/pre-commit
    echo "✅ Pre-commit hooks installed"
fi

# Show final status and helpful information
echo ""
echo "🎉 Toka Rust development environment setup complete!"
echo "=================================================="
echo ""
echo "🔧 Development Tools Ready:"
echo "  • Rust toolchain: $(rustc --version | cut -d' ' -f2)"
echo "  • Cargo: $(cargo --version | cut -d' ' -f2)"
echo "  • Python: $(python3 --version | cut -d' ' -f2)"
if command -v gh &> /dev/null; then
    echo "  • GitHub CLI: $(gh --version | head -n1 | cut -d' ' -f3)"
fi
echo ""
echo "🚀 Quick Start Commands:"
echo "  • toka-quick-test     - Run quick workspace validation"
echo "  • build-all           - Build entire workspace"
echo "  • test-all            - Run all tests"
echo "  • gh repo view        - View repository information"
echo "  • gh pr list          - List pull requests"
echo ""
echo "💡 Use 'source ~/.bashrc' to load new aliases, or restart your terminal."
echo "🚀 Happy coding!" 