#!/bin/bash
# Cursor Agent Cleanup Script
# Cleans up ownership conflicts and resets cursor agent environment

set -euo pipefail

# Configuration
CURSOR_DATA_DIR="/app/data"
CURSOR_LOG_DIR="/app/logs"
CURSOR_LOCK_DIR="/app/data/locks"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "INFO")
            echo -e "${GREEN}[INFO]${NC} ${timestamp} - $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} ${timestamp} - $message"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} ${timestamp} - $message"
            ;;
    esac
}

# Force cleanup of all cursor agent resources
force_cleanup() {
    log "INFO" "Starting force cleanup of cursor agent resources..."
    
    # Stop all cursor agent processes
    log "INFO" "Stopping cursor agent processes..."
    pkill -f "toka-cli.*cursor-mode" || true
    pkill -f "cursor-agent" || true
    
    # Wait for processes to terminate
    sleep 2
    
    # Force kill if still running
    pkill -9 -f "toka-cli.*cursor-mode" || true
    pkill -9 -f "cursor-agent" || true
    
    log "INFO" "All cursor agent processes stopped"
}

# Clean up lock files
cleanup_locks() {
    log "INFO" "Cleaning up lock files..."
    
    if [[ -d "$CURSOR_LOCK_DIR" ]]; then
        local lock_files=$(find "$CURSOR_LOCK_DIR" -name "*.lock" 2>/dev/null || true)
        if [[ -n "$lock_files" ]]; then
            for lock_file in $lock_files; do
                log "INFO" "Removing lock file: $lock_file"
                rm -f "$lock_file"
            done
        else
            log "INFO" "No lock files found"
        fi
    else
        log "INFO" "Lock directory does not exist"
    fi
}

# Clean up database locks
cleanup_database() {
    log "INFO" "Cleaning up database locks..."
    
    local db_file="$CURSOR_DATA_DIR/cursor-agents.db"
    
    if [[ -f "$db_file" ]]; then
        # Remove SQLite WAL and SHM files
        if [[ -f "${db_file}-wal" ]]; then
            log "INFO" "Removing WAL file: ${db_file}-wal"
            rm -f "${db_file}-wal"
        fi
        
        if [[ -f "${db_file}-shm" ]]; then
            log "INFO" "Removing SHM file: ${db_file}-shm"
            rm -f "${db_file}-shm"
        fi
        
        # Reset database permissions
        chmod 644 "$db_file"
        chown toka:toka "$db_file"
        
        log "INFO" "Database cleanup completed"
    else
        log "INFO" "Database file does not exist"
    fi
}

# Clean up cache and temporary files
cleanup_cache() {
    log "INFO" "Cleaning up cache and temporary files..."
    
    local cache_dirs=(
        "$CURSOR_DATA_DIR/cache"
        "$CURSOR_DATA_DIR/sessions"
        "$CURSOR_DATA_DIR/context"
    )
    
    for cache_dir in "${cache_dirs[@]}"; do
        if [[ -d "$cache_dir" ]]; then
            log "INFO" "Cleaning cache directory: $cache_dir"
            rm -rf "$cache_dir"/*
        fi
    done
    
    # Clean up old log files
    if [[ -d "$CURSOR_LOG_DIR" ]]; then
        log "INFO" "Rotating old log files..."
        find "$CURSOR_LOG_DIR" -name "*.log" -mtime +7 -delete || true
    fi
}

# Reset permissions
reset_permissions() {
    log "INFO" "Resetting file permissions..."
    
    # Reset ownership of data and log directories
    if [[ -d "$CURSOR_DATA_DIR" ]]; then
        chown -R toka:toka "$CURSOR_DATA_DIR"
        chmod -R 755 "$CURSOR_DATA_DIR"
    fi
    
    if [[ -d "$CURSOR_LOG_DIR" ]]; then
        chown -R toka:toka "$CURSOR_LOG_DIR"
        chmod -R 755 "$CURSOR_LOG_DIR"
    fi
    
    log "INFO" "Permissions reset completed"
}

# Validate cleanup
validate_cleanup() {
    log "INFO" "Validating cleanup..."
    
    # Check for running processes
    local running_procs=$(pgrep -f "toka-cli.*cursor-mode" || true)
    if [[ -n "$running_procs" ]]; then
        log "WARN" "Some cursor agent processes are still running: $running_procs"
        return 1
    fi
    
    # Check for lock files
    local remaining_locks=$(find "$CURSOR_LOCK_DIR" -name "*.lock" 2>/dev/null || true)
    if [[ -n "$remaining_locks" ]]; then
        log "WARN" "Some lock files remain: $remaining_locks"
        return 1
    fi
    
    # Check for database locks
    local db_file="$CURSOR_DATA_DIR/cursor-agents.db"
    if [[ -f "${db_file}-wal" ]] || [[ -f "${db_file}-shm" ]]; then
        log "WARN" "Database lock files still exist"
        return 1
    fi
    
    log "INFO" "Cleanup validation passed"
    return 0
}

# Display usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --force          Force cleanup without confirmation"
    echo "  --reset-db       Reset database (removes all data)"
    echo "  --help           Show this help message"
    echo ""
    echo "This script cleans up cursor agent ownership conflicts and resets the environment."
}

# Main cleanup function
main() {
    local force_mode=false
    local reset_db=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --force)
                force_mode=true
                shift
                ;;
            --reset-db)
                reset_db=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                log "ERROR" "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    log "INFO" "Starting cursor agent cleanup..."
    
    # Confirmation unless force mode
    if [[ "$force_mode" != "true" ]]; then
        echo -e "${YELLOW}This will clean up all cursor agent resources and stop running processes.${NC}"
        read -p "Continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "INFO" "Cleanup cancelled"
            exit 0
        fi
    fi
    
    # Perform cleanup steps
    force_cleanup
    cleanup_locks
    cleanup_database
    cleanup_cache
    reset_permissions
    
    # Reset database if requested
    if [[ "$reset_db" == "true" ]]; then
        log "WARN" "Resetting database..."
        rm -f "$CURSOR_DATA_DIR/cursor-agents.db"
        log "INFO" "Database reset completed"
    fi
    
    # Validate cleanup
    if validate_cleanup; then
        log "INFO" "Cursor agent cleanup completed successfully"
        echo
        echo -e "${GREEN}✓ Cleanup completed successfully${NC}"
        echo "You can now start cursor agents without ownership conflicts."
    else
        log "ERROR" "Cleanup validation failed"
        echo
        echo -e "${RED}✗ Cleanup completed but some issues remain${NC}"
        echo "Manual intervention may be required."
        exit 1
    fi
}

# Run main function
main "$@" 