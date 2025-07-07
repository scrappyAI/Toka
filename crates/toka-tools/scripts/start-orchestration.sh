#!/bin/bash
# Toka Orchestration Startup Script
# This script initializes and starts the Toka orchestration system

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_DIR="$PROJECT_ROOT/config"
DATA_DIR="$PROJECT_ROOT/data"
LOGS_DIR="$PROJECT_ROOT/logs"
ENV_FILE="$PROJECT_ROOT/.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running in Docker
is_docker() {
    [ -f /.dockerenv ] || [ -n "${DOCKER_CONTAINER:-}" ]
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    mkdir -p "$DATA_DIR" "$LOGS_DIR"
    
    if is_docker; then
        mkdir -p /app/data /app/logs /app/config
    fi
    
    log_success "Directories created successfully"
}

# Check environment variables
check_environment() {
    log_info "Checking environment configuration..."
    
    # Check for .env file
    if [ ! -f "$ENV_FILE" ]; then
        log_warning ".env file not found. Using env.example as template."
        if [ -f "$PROJECT_ROOT/env.example" ]; then
            cp "$PROJECT_ROOT/env.example" "$ENV_FILE"
            log_info "Created .env file from template. Please edit it with your API keys."
        else
            log_error "No environment configuration found. Please create a .env file."
            exit 1
        fi
    fi
    
    # Load environment variables
    if [ -f "$ENV_FILE" ]; then
        log_info "Loading environment variables from .env file..."
        set -a
        # shellcheck source=/dev/null
        source "$ENV_FILE"
        set +a
    fi
    
    # Check critical environment variables
    local missing_vars=()
    
    if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
        missing_vars+=("ANTHROPIC_API_KEY or OPENAI_API_KEY")
    fi
    
    if [ ${#missing_vars[@]} -gt 0 ]; then
        log_error "Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            log_error "  - $var"
        done
        log_error "Please update your .env file with the required values."
        exit 1
    fi
    
    log_success "Environment configuration validated"
}

# Check configuration files
check_configuration() {
    log_info "Checking configuration files..."
    
    local config_file="${CONFIG_FILE:-$CONFIG_DIR/agents.toml}"
    
    if [ ! -f "$config_file" ]; then
        log_error "Configuration file not found: $config_file"
        exit 1
    fi
    
    log_success "Configuration file validated: $config_file"
}

# Build the project
build_project() {
    if is_docker; then
        log_info "Running in Docker - skipping build step"
        return 0
    fi
    
    log_info "Building Toka orchestration service..."
    
    cd "$PROJECT_ROOT"
    
    # Check if we need to build
    if [ "${SKIP_BUILD:-false}" = "true" ]; then
        log_info "Skipping build (SKIP_BUILD=true)"
        return 0
    fi
    
    # Build the orchestration service
    cargo build --release --bin toka-orchestration
    
    log_success "Build completed successfully"
}

# Start the orchestration service
start_orchestration() {
    log_info "Starting Toka orchestration service..."
    
    # Set default configuration
    local config_file="${CONFIG_FILE:-$CONFIG_DIR/agents.toml}"
    local log_level="${LOG_LEVEL:-info}"
    local port="${PORT:-8080}"
    local storage="${STORAGE:-sqlite}"
    local db_path="${DB_PATH:-$DATA_DIR/orchestration.db}"
    
    # Additional flags
    local extra_flags=()
    
    if [ "${CURSOR_MODE:-false}" = "true" ]; then
        extra_flags+=(--cursor-mode)
        config_file="${CONFIG_FILE:-$CONFIG_DIR/cursor-agents.toml}"
    fi
    
    if [ "${DEV_MODE:-false}" = "true" ]; then
        extra_flags+=(--dev)
    fi
    
    # Build command
    local cmd=(
        "$PROJECT_ROOT/target/release/toka-orchestration"
        --config "$config_file"
        --log-level "$log_level"
        --port "$port"
        --storage "$storage"
        --db-path "$db_path"
        "${extra_flags[@]}"
    )
    
    # In Docker, use the installed binary
    if is_docker; then
        cmd[0]="/app/bin/toka-orchestration"
    fi
    
    log_info "Starting orchestration service with configuration: $config_file"
    log_info "Health check will be available at: http://localhost:$port/health"
    log_info "Status endpoint will be available at: http://localhost:$port/status"
    
    # Start the service
    exec "${cmd[@]}"
}

# Show usage information
show_usage() {
    cat << EOF
Toka Orchestration Startup Script

Usage: $0 [OPTIONS]

Options:
  -h, --help          Show this help message
  -c, --config FILE   Configuration file path (default: config/agents.toml)
  -l, --log-level     Log level (default: info)
  -p, --port PORT     HTTP server port (default: 8080)
  --cursor-mode       Enable Cursor mode for background agents
  --dev               Enable development mode
  --skip-build        Skip the build step
  --check-only        Only check configuration and environment, don't start

Environment Variables:
  CONFIG_FILE         Override configuration file path
  LOG_LEVEL          Override log level
  PORT               Override HTTP server port
  STORAGE            Storage backend (memory, sled, sqlite)
  DB_PATH            Database path for persistent storage
  CURSOR_MODE        Enable Cursor mode (true/false)
  DEV_MODE           Enable development mode (true/false)
  SKIP_BUILD         Skip build step (true/false)

Examples:
  $0                                    # Start with default configuration
  $0 --cursor-mode                      # Start in Cursor mode
  $0 --config config/custom.toml        # Use custom configuration
  $0 --dev --log-level debug            # Start in development mode with debug logging
  $0 --check-only                       # Only validate configuration
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -c|--config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            -l|--log-level)
                LOG_LEVEL="$2"
                shift 2
                ;;
            -p|--port)
                PORT="$2"
                shift 2
                ;;
            --cursor-mode)
                CURSOR_MODE=true
                shift
                ;;
            --dev)
                DEV_MODE=true
                shift
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --check-only)
                CHECK_ONLY=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Main execution
main() {
    log_info "Starting Toka Orchestration System..."
    
    parse_args "$@"
    
    create_directories
    check_environment
    check_configuration
    
    if [ "${CHECK_ONLY:-false}" = "true" ]; then
        log_success "Configuration and environment validation completed"
        exit 0
    fi
    
    build_project
    start_orchestration
}

# Execute main function
main "$@" 