#!/bin/bash

# Raft Monitoring Service Management Script
# This script provides easy management of the Raft development monitoring system

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR_SCRIPT="${SCRIPT_DIR}/monitor_raft_development.py"
PID_FILE="${SCRIPT_DIR}/raft_monitor.pid"
LOG_FILE="${SCRIPT_DIR}/raft_monitor.log"
REPORTS_DIR="${SCRIPT_DIR}/raft_monitoring_reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

# Check if Python is available
check_python() {
    if ! command -v python3 &> /dev/null; then
        error "Python 3 is not installed or not in PATH"
        exit 1
    fi
}

# Check if monitoring script exists
check_monitor_script() {
    if [[ ! -f "$MONITOR_SCRIPT" ]]; then
        error "Monitoring script not found at: $MONITOR_SCRIPT"
        exit 1
    fi
}

# Check if service is running
is_running() {
    if [[ -f "$PID_FILE" ]]; then
        local pid=$(cat "$PID_FILE")
        if ps -p "$pid" > /dev/null 2>&1; then
            return 0
        else
            # PID file exists but process is not running
            rm -f "$PID_FILE"
            return 1
        fi
    else
        return 1
    fi
}

# Start the monitoring service
start_monitoring() {
    check_python
    check_monitor_script
    
    if is_running; then
        warn "Raft monitoring service is already running (PID: $(cat "$PID_FILE"))"
        return 0
    fi
    
    log "Starting Raft development monitoring service..."
    
    # Create necessary directories
    mkdir -p "$REPORTS_DIR"
    
    # Start the monitoring script in background
    nohup python3 "$MONITOR_SCRIPT" > "$LOG_FILE" 2>&1 &
    local pid=$!
    
    # Save PID to file
    echo "$pid" > "$PID_FILE"
    
    # Wait a moment and check if process is still running
    sleep 2
    if ps -p "$pid" > /dev/null 2>&1; then
        log "Raft monitoring service started successfully (PID: $pid)"
        log "Logs are being written to: $LOG_FILE"
        log "Reports will be saved to: $REPORTS_DIR"
    else
        error "Failed to start monitoring service"
        rm -f "$PID_FILE"
        exit 1
    fi
}

# Stop the monitoring service
stop_monitoring() {
    if ! is_running; then
        warn "Raft monitoring service is not running"
        return 0
    fi
    
    local pid=$(cat "$PID_FILE")
    log "Stopping Raft monitoring service (PID: $pid)..."
    
    # Send SIGTERM to the process
    kill -TERM "$pid" 2>/dev/null || true
    
    # Wait for process to terminate
    local count=0
    while ps -p "$pid" > /dev/null 2>&1 && [[ $count -lt 10 ]]; do
        sleep 1
        ((count++))
    done
    
    # Force kill if still running
    if ps -p "$pid" > /dev/null 2>&1; then
        warn "Process didn't terminate gracefully, force killing..."
        kill -KILL "$pid" 2>/dev/null || true
    fi
    
    # Clean up PID file
    rm -f "$PID_FILE"
    
    log "Raft monitoring service stopped"
}

# Restart the monitoring service
restart_monitoring() {
    log "Restarting Raft monitoring service..."
    stop_monitoring
    sleep 2
    start_monitoring
}

# Show service status
show_status() {
    if is_running; then
        local pid=$(cat "$PID_FILE")
        log "Raft monitoring service is running (PID: $pid)"
        
        # Show process details
        ps -p "$pid" -o pid,ppid,cmd,etime,pcpu,pmem 2>/dev/null || true
        
        # Show recent log entries
        if [[ -f "$LOG_FILE" ]]; then
            echo
            info "Recent log entries:"
            tail -n 10 "$LOG_FILE"
        fi
        
        # Show report count
        if [[ -d "$REPORTS_DIR" ]]; then
            local report_count=$(find "$REPORTS_DIR" -name "*.json" | wc -l)
            info "Total reports generated: $report_count"
        fi
    else
        warn "Raft monitoring service is not running"
    fi
}

# Show recent reports
show_reports() {
    if [[ ! -d "$REPORTS_DIR" ]]; then
        error "Reports directory not found: $REPORTS_DIR"
        return 1
    fi
    
    local reports=($(find "$REPORTS_DIR" -name "*.json" -type f | sort -r))
    
    if [[ ${#reports[@]} -eq 0 ]]; then
        warn "No reports found in $REPORTS_DIR"
        return 0
    fi
    
    info "Recent monitoring reports:"
    for i in "${!reports[@]}"; do
        if [[ $i -ge 10 ]]; then break; fi  # Show only last 10 reports
        local report="${reports[$i]}"
        local filename=$(basename "$report")
        local size=$(stat -c%s "$report" 2>/dev/null || echo "unknown")
        local timestamp=$(stat -c%y "$report" 2>/dev/null || echo "unknown")
        echo "  $((i+1)). $filename (${size} bytes, $timestamp)"
    done
    
    if [[ ${#reports[@]} -gt 10 ]]; then
        info "... and $((${#reports[@]} - 10)) more reports"
    fi
}

# View latest report
view_latest_report() {
    if [[ ! -d "$REPORTS_DIR" ]]; then
        error "Reports directory not found: $REPORTS_DIR"
        return 1
    fi
    
    local latest_report=$(find "$REPORTS_DIR" -name "*.json" -type f | sort -r | head -n 1)
    
    if [[ -z "$latest_report" ]]; then
        warn "No reports found in $REPORTS_DIR"
        return 0
    fi
    
    info "Latest report: $(basename "$latest_report")"
    
    # Use jq if available for pretty printing, otherwise use cat
    if command -v jq &> /dev/null; then
        jq '.' "$latest_report"
    else
        cat "$latest_report"
    fi
}

# Show help
show_help() {
    cat << EOF
Raft Monitoring Service Management Script

Usage: $0 <command>

Commands:
    start       Start the monitoring service
    stop        Stop the monitoring service
    restart     Restart the monitoring service
    status      Show service status
    reports     List recent monitoring reports
    view        View the latest monitoring report
    help        Show this help message

Examples:
    $0 start        # Start monitoring
    $0 status       # Check if monitoring is running
    $0 reports      # List recent reports
    $0 view         # View latest report with details

Configuration:
    Monitor script:  $MONITOR_SCRIPT
    PID file:       $PID_FILE
    Log file:       $LOG_FILE
    Reports dir:    $REPORTS_DIR
EOF
}

# Main function
main() {
    case "${1:-help}" in
        start)
            start_monitoring
            ;;
        stop)
            stop_monitoring
            ;;
        restart)
            restart_monitoring
            ;;
        status)
            show_status
            ;;
        reports)
            show_reports
            ;;
        view)
            view_latest_report
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"