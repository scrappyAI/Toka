#!/bin/bash
# Toka Auto-Setup Script for Cursor Agents
# Automatically configures environment, secrets, and project setup

set -euo pipefail

# Configuration
TOKA_WORKSPACE="${TOKA_WORKSPACE:-/home/vscode}"
TOKA_CONFIG_DIR="$TOKA_WORKSPACE/config"
TOKA_DATA_DIR="$TOKA_WORKSPACE/data"
TOKA_LOG_DIR="$TOKA_WORKSPACE/logs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "INFO")  echo -e "${GREEN}[AUTO-SETUP]${NC} $message" ;;
        "WARN")  echo -e "${YELLOW}[AUTO-SETUP]${NC} $message" ;;
        "ERROR") echo -e "${RED}[AUTO-SETUP]${NC} $message" ;;
        "DEBUG") echo -e "${BLUE}[AUTO-SETUP]${NC} $message" ;;
        "SUCCESS") echo -e "${PURPLE}[AUTO-SETUP]${NC} $message" ;;
    esac
}

# Auto-detect and setup secrets
setup_secrets() {
    log "INFO" "Setting up secrets and API keys..."
    
    # Create secrets directory
    mkdir -p "$TOKA_WORKSPACE/.secrets"
    
    # Auto-detect GitHub token from environment or common locations
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        echo "$GITHUB_TOKEN" > "$TOKA_WORKSPACE/.secrets/github_token"
        log "SUCCESS" "GitHub token configured from environment"
    elif [[ -f "/home/vscode/.config/gh/hosts.yml" ]]; then
        # Extract token from GitHub CLI config if available
        local gh_token=$(grep -oP 'oauth_token: \K.*' /home/vscode/.config/gh/hosts.yml 2>/dev/null || true)
        if [[ -n "$gh_token" ]]; then
            echo "$gh_token" > "$TOKA_WORKSPACE/.secrets/github_token"
            log "SUCCESS" "GitHub token extracted from GitHub CLI"
        fi
    fi
    
    # Setup OpenAI API key
    if [[ -n "${OPENAI_API_KEY:-}" ]]; then
        echo "$OPENAI_API_KEY" > "$TOKA_WORKSPACE/.secrets/openai_key"
        log "SUCCESS" "OpenAI API key configured"
    fi
    
    # Setup Anthropic API key
    if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
        echo "$ANTHROPIC_API_KEY" > "$TOKA_WORKSPACE/.secrets/anthropic_key"
        log "SUCCESS" "Anthropic API key configured"
    fi
    
    # Generate Toka secret key if not provided
    if [[ -n "${TOKA_SECRET_KEY:-}" ]]; then
        echo "$TOKA_SECRET_KEY" > "$TOKA_WORKSPACE/.secrets/toka_secret"
    else
        # Generate a secure random key
        openssl rand -base64 32 > "$TOKA_WORKSPACE/.secrets/toka_secret"
        log "SUCCESS" "Generated new Toka secret key"
    fi
    
    # Set proper permissions
    chmod 600 "$TOKA_WORKSPACE/.secrets/"* 2>/dev/null || true
    
    log "INFO" "Secret setup completed"
}

# Auto-generate configuration files
setup_config() {
    log "INFO" "Setting up auto-generated configuration..."
    
    mkdir -p "$TOKA_CONFIG_DIR"
    
    # Generate cursor-agents.toml with auto-detected settings
    cat > "$TOKA_CONFIG_DIR/cursor-agents.toml" << 'EOF'
[agent]
name = "toka-cursor-agent"
version = "0.3.0"
mode = "cursor"
workspace = "/home/vscode"
auto_setup = true

[runtime]
max_memory = "2GB"
max_cpu_cores = 4
single_owner = true
database_path = "/home/vscode/data/cursor-agents.db"

[logging]
level = "info"
output = "/home/vscode/logs/cursor-agents.log"
metrics = "/home/vscode/logs/cursor-metrics.log"
errors = "/home/vscode/logs/cursor-errors.log"

[security]
secret_key_file = "/home/vscode/.secrets/toka_secret"
capability_validation = true
rate_limiting = true

[integrations]
github_enabled = true
github_token_file = "/home/vscode/.secrets/github_token"
llm_gateway_enabled = true
openai_key_file = "/home/vscode/.secrets/openai_key"
anthropic_key_file = "/home/vscode/.secrets/anthropic_key"

[performance]
monitoring_enabled = true
metrics_interval = 30
cache_enabled = true
cache_size = "256MB"
EOF
    
    # Generate environment-specific config
    cat > "$TOKA_CONFIG_DIR/env.cursor" << 'EOF'
# Cursor Agent Environment Configuration
TOKA_ENV=development
TOKA_MODE=cursor
TOKA_WORKSPACE=/home/vscode
TOKA_AUTO_SETUP=true
RUST_LOG=info
CARGO_INCREMENTAL=1
CARGO_TARGET_DIR=/home/vscode/target
EOF
    
    log "SUCCESS" "Configuration files generated"
}

# Setup directories and permissions
setup_directories() {
    log "INFO" "Setting up directory structure..."
    
    # Create required directories
    local dirs=(
        "$TOKA_CONFIG_DIR"
        "$TOKA_DATA_DIR"
        "$TOKA_LOG_DIR"
        "$TOKA_WORKSPACE/target"
        "$TOKA_WORKSPACE/.cache"
        "$TOKA_DATA_DIR/sessions"
        "$TOKA_DATA_DIR/context"
        "$TOKA_DATA_DIR/locks"
    )
    
    for dir in "${dirs[@]}"; do
        mkdir -p "$dir"
        log "DEBUG" "Created directory: $dir"
    done
    
    # Set proper ownership
    chown -R vscode:vscode "$TOKA_WORKSPACE" 2>/dev/null || true
    
    log "SUCCESS" "Directory structure setup completed"
}

# Auto-detect project type and setup build environment
setup_build_environment() {
    log "INFO" "Setting up build environment..."
    
    # Wait for project to be cloned (in case we're running before clone)
    local wait_count=0
    while [[ ! -f "$TOKA_WORKSPACE/Cargo.toml" && $wait_count -lt 30 ]]; do
        sleep 1
        ((wait_count++))
    done
    
    if [[ -f "$TOKA_WORKSPACE/Cargo.toml" ]]; then
        log "SUCCESS" "Toka project detected"
        
        # Pre-fetch dependencies for faster builds
        cd "$TOKA_WORKSPACE"
        log "INFO" "Pre-fetching Rust dependencies..."
        cargo fetch --locked 2>/dev/null || cargo fetch || true
        
        # Setup git hooks if in a git repo
        if [[ -d ".git" ]]; then
            log "INFO" "Setting up git hooks..."
            git config --local core.hooksPath .githooks 2>/dev/null || true
        fi
        
        log "SUCCESS" "Build environment ready"
    else
        log "WARN" "No Cargo.toml found - project may not be cloned yet"
    fi
}

# Health check setup
setup_health_monitoring() {
    log "INFO" "Setting up health monitoring..."
    
    # Create health check script
    cat > "$TOKA_WORKSPACE/.health-check" << 'EOF'
#!/bin/bash
# Quick health check for cursor agents
set -e

# Check if toka-cli is responsive
if command -v cargo >/dev/null 2>&1; then
    if [[ -f "Cargo.toml" ]]; then
        cargo check --quiet >/dev/null 2>&1 || exit 1
    fi
fi

# Check if logs are being written (agent is active)
if [[ -f "/home/vscode/logs/cursor-agents.log" ]]; then
    # Check if log has been updated in last 5 minutes
    if [[ $(find /home/vscode/logs/cursor-agents.log -mmin -5 2>/dev/null | wc -l) -eq 0 ]]; then
        exit 1
    fi
fi

echo "healthy"
EOF
    
    chmod +x "$TOKA_WORKSPACE/.health-check"
    log "SUCCESS" "Health monitoring configured"
}

# Main setup function
main() {
    log "INFO" "Starting Toka Cursor Agent auto-setup..."
    log "INFO" "Workspace: $TOKA_WORKSPACE"
    
    # Run setup steps
    setup_directories
    setup_secrets
    setup_config
    setup_build_environment
    setup_health_monitoring
    
    log "SUCCESS" "Auto-setup completed successfully!"
    log "INFO" "Cursor agent environment is ready for agentic operation"
    
    # Output useful information
    echo ""
    log "INFO" "=== Setup Summary ==="
    log "INFO" "Workspace: $TOKA_WORKSPACE"
    log "INFO" "Config: $TOKA_CONFIG_DIR/cursor-agents.toml"
    log "INFO" "Secrets: $TOKA_WORKSPACE/.secrets/"
    log "INFO" "Logs: $TOKA_LOG_DIR/"
    echo ""
    log "INFO" "To start interactive mode: cargo run --bin toka-cli -- interactive"
    log "INFO" "To run tests: cargo nextest run --workspace"
    echo ""
}

# Run main function
main "$@" 