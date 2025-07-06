#!/bin/bash
# Cursor Agent Initialization Script
# Sets up the environment and starts cursor agents with ownership management

set -euo pipefail

# Configuration
CURSOR_CONFIG_DIR="/app/config"
CURSOR_DATA_DIR="/app/data"
CURSOR_LOG_DIR="/app/logs"
CURSOR_SCRIPT_DIR="/app/scripts"
CURSOR_LOCK_DIR="/app/data/locks"
CURSOR_OWNER_ID="${USER:-toka}-$$-$(date +%s)"

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

# Check for existing ownership conflicts
check_ownership() {
    log "INFO" "Checking for ownership conflicts..."
    
    # Create lock directory if it doesn't exist
    mkdir -p "$CURSOR_LOCK_DIR"
    
    # Check for existing lock file
    local lock_file="$CURSOR_LOCK_DIR/cursor-agent.lock"
    
    if [[ -f "$lock_file" ]]; then
        local existing_owner=$(cat "$lock_file")
        local existing_pid=$(echo "$existing_owner" | cut -d'-' -f2)
        
        # Check if the process is still running
        if kill -0 "$existing_pid" 2>/dev/null; then
            log "ERROR" "Another cursor agent instance is already running (PID: $existing_pid)"
            log "ERROR" "Owner: $existing_owner"
            log "ERROR" "To force restart, remove: $lock_file"
            exit 1
        else
            log "WARN" "Found stale lock file, removing..."
            rm -f "$lock_file"
        fi
    fi
    
    # Create our lock file
    echo "$CURSOR_OWNER_ID" > "$lock_file"
    log "INFO" "Acquired ownership with ID: $CURSOR_OWNER_ID"
}

# Cleanup function
cleanup_ownership() {
    log "INFO" "Cleaning up ownership..."
    
    local lock_file="$CURSOR_LOCK_DIR/cursor-agent.lock"
    if [[ -f "$lock_file" ]]; then
        local current_owner=$(cat "$lock_file")
        if [[ "$current_owner" == "$CURSOR_OWNER_ID" ]]; then
            rm -f "$lock_file"
            log "INFO" "Released ownership"
        fi
    fi
    
    # Cleanup any running processes
    pkill -f "toka-cli.*cursor-mode" || true
    
    # Clean up database locks
    if [[ -f "$CURSOR_DATA_DIR/cursor-agents.db-wal" ]]; then
        rm -f "$CURSOR_DATA_DIR/cursor-agents.db-wal"
    fi
    if [[ -f "$CURSOR_DATA_DIR/cursor-agents.db-shm" ]]; then
        rm -f "$CURSOR_DATA_DIR/cursor-agents.db-shm"
    fi
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
    
    # Set up database with single-owner mode
    if [[ ! -f "$CURSOR_DATA_DIR/cursor-agents.db" ]]; then
        log "INFO" "Initializing cursor agent database..."
        /app/bin/toka-config-cli init-db \
            --cursor-mode \
            --single-owner \
            --owner-id "$CURSOR_OWNER_ID" \
            --db-path "$CURSOR_DATA_DIR/cursor-agents.db"
    else
        # Validate database ownership
        log "INFO" "Validating database ownership..."
        /app/bin/toka-config-cli validate-ownership \
            --cursor-mode \
            --owner-id "$CURSOR_OWNER_ID" \
            --db-path "$CURSOR_DATA_DIR/cursor-agents.db" || {
            log "ERROR" "Database ownership validation failed"
            exit 1
        }
    fi
    
    # Initialize cursor-specific directories
    mkdir -p "$CURSOR_DATA_DIR/cache"
    mkdir -p "$CURSOR_DATA_DIR/sessions"
    mkdir -p "$CURSOR_DATA_DIR/context"
    mkdir -p "$CURSOR_LOCK_DIR"
    
    # Set ownership permissions
    chown -R toka:toka "$CURSOR_DATA_DIR" "$CURSOR_LOG_DIR"
    
    log "INFO" "Environment initialization complete"
}

# Validate configuration
validate_configuration() {
    log "INFO" "Validating cursor agent configuration..."
    
    /app/bin/toka-config-cli validate \
        --config "$CURSOR_CONFIG_DIR/cursor-agents.toml" \
        --cursor-mode \
        --single-owner \
        --owner-id "$CURSOR_OWNER_ID"
    
    if [[ $? -ne 0 ]]; then
        log "ERROR" "Configuration validation failed"
        exit 1
    fi
    
    log "INFO" "Configuration validation passed"
}

# Start cursor agents
start_cursor_agents() {
    log "INFO" "Starting cursor agents with owner ID: $CURSOR_OWNER_ID"
    
    # Start the main orchestrator with ownership settings
    exec /app/bin/toka-cli orchestrate \
        --config "$CURSOR_CONFIG_DIR/cursor-agents.toml" \
        --cursor-mode \
        --single-owner \
        --owner-id "$CURSOR_OWNER_ID" \
        --daemon \
        --log-file "$CURSOR_LOG_DIR/cursor-agents.log" \
        --metrics-file "$CURSOR_LOG_DIR/cursor-metrics.log" \
        --error-file "$CURSOR_LOG_DIR/cursor-errors.log"
}

# Health check with ownership validation
health_check() {
    log "INFO" "Performing ownership health check..."
    
    local lock_file="$CURSOR_LOCK_DIR/cursor-agent.lock"
    if [[ -f "$lock_file" ]]; then
        local current_owner=$(cat "$lock_file")
        if [[ "$current_owner" != "$CURSOR_OWNER_ID" ]]; then
            log "ERROR" "Ownership conflict detected. Current owner: $current_owner"
            exit 1
        fi
    else
        log "ERROR" "Lock file missing, ownership lost"
        exit 1
    fi
    
    log "INFO" "Ownership health check passed"
}

# Main execution
main() {
    log "INFO" "Starting cursor agent initialization with ownership management..."
    
    # Set up signal handlers for cleanup
    trap 'cleanup_ownership; exit 0' SIGTERM SIGINT EXIT
    
    check_ownership
    check_prerequisites
    initialize_environment
    validate_configuration
    health_check
    start_cursor_agents
    
    log "INFO" "Cursor agent initialization complete"
}

# Handle signals with proper cleanup
trap 'log "INFO" "Received shutdown signal, cleaning up..."; cleanup_ownership; exit 0' SIGTERM SIGINT

# Run main function
main "$@" 