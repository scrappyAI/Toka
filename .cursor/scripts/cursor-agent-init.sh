#!/bin/bash
# Cursor Agent Initialization Script
# Sets up the environment and starts cursor agents

set -euo pipefail

# Configuration
CURSOR_CONFIG_DIR="/app/config"
CURSOR_DATA_DIR="/app/data"
CURSOR_LOG_DIR="/app/logs"
CURSOR_SCRIPT_DIR="/app/scripts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "INFO")
            echo -e "${GREEN}[INFO]${NC} ${timestamp} - $message" | tee -a "$CURSOR_LOG_DIR/init.log"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} ${timestamp} - $message" | tee -a "$CURSOR_LOG_DIR/init.log"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} ${timestamp} - $message" | tee -a "$CURSOR_LOG_DIR/init.log"
            ;;
        "DEBUG")
            if [[ "${CURSOR_DEBUG:-false}" == "true" ]]; then
                echo -e "${BLUE}[DEBUG]${NC} ${timestamp} - $message" | tee -a "$CURSOR_LOG_DIR/init.log"
            fi
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log "INFO" "Checking cursor agent prerequisites..."
    
    # Check required directories
    local required_dirs=("$CURSOR_CONFIG_DIR" "$CURSOR_DATA_DIR" "$CURSOR_LOG_DIR")
    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            log "ERROR" "Required directory missing: $dir"
            exit 1
        fi
    done
    
    # Check required files
    if [[ ! -f "$CURSOR_CONFIG_DIR/cursor-agents.toml" ]]; then
        log "ERROR" "Cursor agent configuration missing: $CURSOR_CONFIG_DIR/cursor-agents.toml"
        exit 1
    fi
    
    # Check binaries
    if [[ ! -x "/app/bin/toka-cli" ]]; then
        log "ERROR" "Toka CLI binary not found or not executable"
        exit 1
    fi
    
    log "INFO" "Prerequisites check passed"
}

# Initialize cursor agent environment
initialize_environment() {
    log "INFO" "Initializing cursor agent environment..."
    
    # Create log files
    touch "$CURSOR_LOG_DIR/cursor-agents.log"
    touch "$CURSOR_LOG_DIR/cursor-metrics.log"
    touch "$CURSOR_LOG_DIR/cursor-errors.log"
    
    # Set up database
    if [[ ! -f "$CURSOR_DATA_DIR/cursor-agents.db" ]]; then
        log "INFO" "Initializing cursor agent database..."
        /app/bin/toka-config-cli init-db --cursor-mode --db-path "$CURSOR_DATA_DIR/cursor-agents.db"
    fi
    
    # Initialize cursor-specific directories
    mkdir -p "$CURSOR_DATA_DIR/cache"
    mkdir -p "$CURSOR_DATA_DIR/sessions"
    mkdir -p "$CURSOR_DATA_DIR/context"
    
    log "INFO" "Environment initialization complete"
}

# Validate configuration
validate_configuration() {
    log "INFO" "Validating cursor agent configuration..."
    
    /app/bin/toka-config-cli validate --config "$CURSOR_CONFIG_DIR/cursor-agents.toml" --cursor-mode
    
    if [[ $? -ne 0 ]]; then
        log "ERROR" "Configuration validation failed"
        exit 1
    fi
    
    log "INFO" "Configuration validation passed"
}

# Start cursor agents
start_cursor_agents() {
    log "INFO" "Starting cursor agents..."
    
    # Start the main orchestrator
    exec /app/bin/toka-cli orchestrate \
        --config "$CURSOR_CONFIG_DIR/cursor-agents.toml" \
        --cursor-mode \
        --daemon \
        --log-file "$CURSOR_LOG_DIR/cursor-agents.log" \
        --metrics-file "$CURSOR_LOG_DIR/cursor-metrics.log" \
        --error-file "$CURSOR_LOG_DIR/cursor-errors.log"
}

# Main execution
main() {
    log "INFO" "Starting cursor agent initialization..."
    
    check_prerequisites
    initialize_environment
    validate_configuration
    start_cursor_agents
    
    log "INFO" "Cursor agent initialization complete"
}

# Handle signals
trap 'log "INFO" "Received shutdown signal, stopping cursor agents..."; pkill -f "toka-cli"; exit 0' SIGTERM SIGINT

# Run main function
main "$@" 