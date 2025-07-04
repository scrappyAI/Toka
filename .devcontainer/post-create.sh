#!/bin/bash

# Post-create script for Toka Rust development environment
set -e

echo "🦀 Setting up Toka Rust development environment..."

# Install essential build tools
echo "🔧 Installing build dependencies..."
sudo apt update
sudo apt install -y clang lld build-essential

# Ensure cargo registry is accessible
mkdir -p ~/.cargo
echo "Creating target directory for faster builds..."
mkdir -p /tmp/target
chmod 755 /tmp/target

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

# Pre-fetch dependencies for faster first build
echo "⚡ Pre-fetching workspace dependencies..."
cargo fetch --locked || echo "Warning: Could not pre-fetch dependencies"

# Set up git if not already configured
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

EOF

# Verify workspace structure
echo "🏗️  Verifying workspace structure..."
if [ -f "Cargo.toml" ]; then
    echo "✅ Found workspace Cargo.toml"
    cargo metadata --no-deps --format-version 1 > /dev/null && echo "✅ Workspace metadata is valid"
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

echo "🎉 Toka Rust development environment setup complete!"
echo "💡 Use 'source ~/.bashrc' to load new aliases, or restart your terminal."
echo "🚀 Happy coding!" 