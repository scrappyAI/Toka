#!/bin/bash
# Toka Development Environment Setup
# Configures development environment with helpful aliases and tools

set -euo pipefail

# Configuration
TOKA_WORKSPACE="${TOKA_WORKSPACE:-/home/vscode}"
TOKA_CONFIG_DIR="$TOKA_WORKSPACE/config"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() {
    echo -e "${PURPLE}[DEV-SETUP]${NC} $*"
}

# Setup development aliases and functions
setup_dev_aliases() {
    log "Setting up development aliases..."
    
    cat >> ~/.bashrc << 'EOF'

# Toka Development Aliases
alias tk='cargo run --bin toka-cli --'
alias tki='cargo run --bin toka-cli -- interactive'
alias tktest='cargo nextest run --workspace'
alias tkcheck='cargo check --workspace'
alias tkfmt='cargo fmt --all'
alias tklint='cargo clippy --workspace --all-targets --all-features'
alias tkbuild='cargo build --workspace --release'
alias tkwatch='cargo watch -x "check --workspace"'

# Quick navigation
alias tkconfig='cd /home/vscode/config && ls -la'
alias tklogs='cd /home/vscode/logs && tail -f cursor-agents.log'
alias tkdata='cd /home/vscode/data && ls -la'

# Development utilities
alias tkenv='source /home/vscode/config/env.cursor'
alias tkstatus='ps aux | grep -E "(toka|cursor)" | grep -v grep'
alias tkmem='cargo bloat --release --crates'
alias tkdeps='cargo tree --workspace'

# Testing shortcuts
alias tkunit='cargo test --workspace --lib'
alias tkintegration='cargo test --workspace --test "*"'
alias tkcoverage='cargo llvm-cov --workspace --html'
alias tkbench='cargo criterion'

# Quick development functions
tk_quick_test() {
    echo "Running quick Toka tests..."
    cargo check --workspace --all-targets
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --lib
}

tk_full_test() {
    echo "Running full Toka test suite..."
    cargo fmt --all -- --check
    cargo check --workspace --all-targets
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo nextest run --workspace
    cargo test --workspace --doc
}

tk_dev_status() {
    echo "=== Toka Development Status ==="
    echo "Workspace: $PWD"
    echo "Git branch: $(git branch --show-current 2>/dev/null || echo 'unknown')"
    echo "Git status:"
    git status --porcelain 2>/dev/null || echo "Not in git repo"
    echo ""
    echo "Cargo workspace info:"
    cargo metadata --format-version 1 2>/dev/null | jq -r '.workspace_members[]' 2>/dev/null || echo "cargo metadata failed"
    echo ""
    echo "Running processes:"
    ps aux | grep -E "(toka|cursor)" | grep -v grep || echo "No Toka processes running"
}

tk_clean_build() {
    echo "Performing clean Toka build..."
    cargo clean
    cargo build --workspace --release
    echo "Clean build completed!"
}

# Export functions
export -f tk_quick_test tk_full_test tk_dev_status tk_clean_build
EOF
    
    # Also add to .zshrc if it exists
    if [[ -f ~/.zshrc ]]; then
        cat >> ~/.zshrc << 'EOF'

# Source Toka aliases from bashrc
source ~/.bashrc
EOF
    fi
    
    log "Development aliases configured"
}

# Setup development environment variables
setup_dev_environment() {
    log "Setting up development environment..."
    
    # Source the cursor environment if available
    if [[ -f "$TOKA_CONFIG_DIR/env.cursor" ]]; then
        source "$TOKA_CONFIG_DIR/env.cursor"
        log "Loaded Cursor environment configuration"
    fi
    
    # Set development-specific variables
    export RUST_LOG="${RUST_LOG:-info}"
    export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
    export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-1}"
    export TOKA_ENV="development"
    
    # Setup PATH for local binaries
    export PATH="$TOKA_WORKSPACE/target/release:$TOKA_WORKSPACE/target/debug:$PATH"
    
    log "Development environment configured"
}

# Setup git development configuration
setup_git_config() {
    log "Setting up git configuration for development..."
    
    # Set up useful git aliases for Toka development
    git config --global alias.tk-log "log --oneline --graph --decorate --all"
    git config --global alias.tk-status "status --short --branch"
    git config --global alias.tk-diff "diff --name-status"
    git config --global alias.tk-clean "clean -fd"
    
    # Set up commit template if it doesn't exist
    if [[ ! -f ~/.gitmessage ]]; then
        cat > ~/.gitmessage << 'EOF'
# [type](scope): brief description
#
# Longer description if needed
#
# - bullet points for details
#
# Types: feat, fix, docs, style, refactor, test, chore
# Scopes: kernel, auth, cli, storage, security, etc.
EOF
        git config --global commit.template ~/.gitmessage
    fi
    
    log "Git configuration setup completed"
}

# Setup development tools
setup_dev_tools() {
    log "Setting up development tools..."
    
    # Create useful development scripts
    mkdir -p "$TOKA_WORKSPACE/.dev-tools"
    
    # Create a quick project overview script
    cat > "$TOKA_WORKSPACE/.dev-tools/overview.sh" << 'EOF'
#!/bin/bash
echo "=== Toka Project Overview ==="
echo "Project root: $(pwd)"
echo "Total lines of Rust code:"
find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1
echo ""
echo "Crates in workspace:"
ls -1 crates/ 2>/dev/null || echo "No crates directory found"
echo ""
echo "Recent commits:"
git log --oneline -5 2>/dev/null || echo "Not in git repo"
EOF
    chmod +x "$TOKA_WORKSPACE/.dev-tools/overview.sh"
    
    # Create a dependency analysis script
    cat > "$TOKA_WORKSPACE/.dev-tools/deps.sh" << 'EOF'
#!/bin/bash
echo "=== Dependency Analysis ==="
echo "External dependencies:"
cargo tree --workspace --edges normal 2>/dev/null | grep -v "├─" | grep -v "│" | sort | uniq || echo "cargo tree failed"
echo ""
echo "Workspace dependencies:"
cargo metadata --format-version 1 2>/dev/null | jq -r '.workspace_members[]' | sed 's/.*\///' | sed 's/ .*//' || echo "cargo metadata failed"
EOF
    chmod +x "$TOKA_WORKSPACE/.dev-tools/deps.sh"
    
    log "Development tools setup completed"
}

# Show helpful information
show_dev_info() {
    echo ""
    log "=== Development Environment Ready ==="
    echo ""
    echo -e "${GREEN}Available aliases:${NC}"
    echo "  tk           - Run toka-cli with arguments"
    echo "  tki          - Interactive Toka CLI"
    echo "  tktest       - Run test suite"
    echo "  tkcheck      - Quick check"
    echo "  tkfmt        - Format code"
    echo "  tklint       - Run clippy"
    echo ""
    echo -e "${GREEN}Available functions:${NC}"
    echo "  tk_quick_test     - Fast development tests"
    echo "  tk_full_test      - Complete test suite"
    echo "  tk_dev_status     - Show development status"
    echo "  tk_clean_build    - Clean rebuild"
    echo ""
    echo -e "${GREEN}Navigation aliases:${NC}"
    echo "  tkconfig     - Go to config directory"
    echo "  tklogs       - Go to logs and tail"
    echo "  tkdata       - Go to data directory"
    echo ""
    echo -e "${YELLOW}To start development:${NC}"
    echo "  1. Run 'tk_dev_status' to see current state"
    echo "  2. Run 'tki' for interactive mode"
    echo "  3. Run 'tktest' to validate everything works"
    echo ""
    
    # Show current project status if available
    if [[ -f "Cargo.toml" ]]; then
        echo -e "${BLUE}Current project status:${NC}"
        tk_dev_status 2>/dev/null || echo "Status check failed"
    else
        echo -e "${YELLOW}Waiting for project to be cloned...${NC}"
    fi
}

# Main setup function
main() {
    log "Starting Toka development environment setup..."
    
    cd "$TOKA_WORKSPACE"
    
    setup_dev_environment
    setup_dev_aliases
    setup_git_config
    setup_dev_tools
    
    # Source the new aliases
    source ~/.bashrc 2>/dev/null || true
    
    show_dev_info
    
    log "Development environment setup completed!"
}

# Run main function
main "$@" 