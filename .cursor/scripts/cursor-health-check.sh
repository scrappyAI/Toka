#!/bin/bash
# Cursor Agent Health Check Script
# Monitors cursor agent health and reports status

set -euo pipefail

# Configuration
CURSOR_API_HOST="localhost"
CURSOR_API_PORT="8080"
CURSOR_MGMT_PORT="9000"
CURSOR_WS_PORT="9001"
CURSOR_HEALTH_PORT="3000"
CURSOR_LOG_DIR="/app/logs"
CURSOR_LOCK_DIR="/app/data/locks"

# Health check thresholds
MAX_RESPONSE_TIME=5000  # milliseconds
MAX_MEMORY_USAGE=80     # percentage
MAX_CPU_USAGE=75        # percentage
MIN_DISK_SPACE=1048576  # bytes (1GB)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Health status
OVERALL_STATUS="HEALTHY"
HEALTH_ISSUES=()

# Quick mode flag
QUICK_MODE=false

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    if [[ "$QUICK_MODE" != "true" ]]; then
        echo -e "${timestamp} [$level] $message"
    fi
    echo "${timestamp} [$level] $message" >> "$CURSOR_LOG_DIR/health-check.log" 2>/dev/null || true
}

# Check ownership status
check_ownership() {
    log "INFO" "Checking ownership status..."
    
    local lock_file="$CURSOR_LOCK_DIR/cursor-agent.lock"
    
    if [[ -f "$lock_file" ]]; then
        local owner=$(cat "$lock_file" 2>/dev/null || echo "unknown")
        local owner_pid=$(echo "$owner" | cut -d'-' -f2 2>/dev/null || echo "0")
        
        # Check if the owner process is still running
        if [[ "$owner_pid" != "0" ]] && kill -0 "$owner_pid" 2>/dev/null; then
            log "INFO" "Valid ownership detected: $owner"
        else
            log "WARN" "Stale ownership detected: $owner"
            HEALTH_ISSUES+=("Stale ownership lock detected")
            OVERALL_STATUS="DEGRADED"
        fi
    else
        log "WARN" "No ownership lock file found"
        HEALTH_ISSUES+=("No ownership lock file")
        OVERALL_STATUS="DEGRADED"
    fi
}

# Check API endpoint health
check_api_health() {
    log "INFO" "Checking cursor API health..."
    
    local start_time=$(date +%s%3N)
    local response_code=$(curl -s -o /dev/null -w "%{http_code}" \
        "http://${CURSOR_API_HOST}:${CURSOR_API_PORT}/health/cursor" \
        --connect-timeout 5 --max-time 10 2>/dev/null || echo "000")
    local end_time=$(date +%s%3N)
    local response_time=$((end_time - start_time))
    
    if [[ "$response_code" == "200" ]]; then
        log "INFO" "API health check passed (${response_time}ms)"
        if [[ $response_time -gt $MAX_RESPONSE_TIME ]]; then
            HEALTH_ISSUES+=("API response time high: ${response_time}ms")
            OVERALL_STATUS="DEGRADED"
        fi
    else
        log "ERROR" "API health check failed (HTTP $response_code)"
        HEALTH_ISSUES+=("API health check failed")
        OVERALL_STATUS="UNHEALTHY"
    fi
}

# Check WebSocket connectivity (quick mode compatible)
check_websocket_health() {
    if [[ "$QUICK_MODE" == "true" ]]; then
        # Quick check - just test if port is open
        timeout 3 bash -c "echo >/dev/tcp/${CURSOR_API_HOST}/${CURSOR_WS_PORT}" 2>/dev/null && return 0 || return 1
    else
        log "INFO" "Checking WebSocket connectivity..."
        
        if timeout 3 bash -c "echo >/dev/tcp/${CURSOR_API_HOST}/${CURSOR_WS_PORT}"; then
            log "INFO" "WebSocket port is accessible"
        else
            log "ERROR" "WebSocket port is not accessible"
            HEALTH_ISSUES+=("WebSocket port not accessible")
            OVERALL_STATUS="UNHEALTHY"
        fi
    fi
}

# Check agent status
check_agent_status() {
    log "INFO" "Checking cursor agent status..."
    
    local agent_status=$(curl -s "http://${CURSOR_API_HOST}:${CURSOR_MGMT_PORT}/agents/status" \
        --connect-timeout 5 --max-time 10 2>/dev/null || echo "ERROR")
    
    if [[ "$agent_status" == "ERROR" ]]; then
        log "ERROR" "Failed to retrieve agent status"
        HEALTH_ISSUES+=("Agent status check failed")
        OVERALL_STATUS="UNHEALTHY"
        return
    fi
    
    # Parse agent status (assuming JSON response)
    local active_agents=$(echo "$agent_status" | jq -r '.active_agents // 0' 2>/dev/null || echo "0")
    local failed_agents=$(echo "$agent_status" | jq -r '.failed_agents // 0' 2>/dev/null || echo "0")
    
    log "INFO" "Active agents: $active_agents, Failed agents: $failed_agents"
    
    if [[ $failed_agents -gt 0 ]]; then
        HEALTH_ISSUES+=("$failed_agents agents have failed")
        OVERALL_STATUS="DEGRADED"
    fi
    
    if [[ $active_agents -eq 0 ]]; then
        HEALTH_ISSUES+=("No active agents running")
        OVERALL_STATUS="UNHEALTHY"
    fi
}

# Check resource usage (skip in quick mode)
check_resource_usage() {
    if [[ "$QUICK_MODE" == "true" ]]; then
        return 0
    fi
    
    log "INFO" "Checking resource usage..."
    
    # Check memory usage
    local memory_usage=$(free | grep Mem | awk '{printf "%.0f", $3/$2 * 100.0}' 2>/dev/null || echo "0")
    if [[ $memory_usage -gt $MAX_MEMORY_USAGE ]]; then
        HEALTH_ISSUES+=("High memory usage: ${memory_usage}%")
        OVERALL_STATUS="DEGRADED"
    fi
    
    # Check CPU usage (5-minute average)
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}' | cut -d',' -f1 2>/dev/null || echo "0")
    if [[ ${cpu_usage%.*} -gt $MAX_CPU_USAGE ]]; then
        HEALTH_ISSUES+=("High CPU usage: ${cpu_usage}%")
        OVERALL_STATUS="DEGRADED"
    fi
    
    # Check disk space
    local disk_available=$(df /app/data 2>/dev/null | tail -1 | awk '{print $4}' || echo "0")
    local disk_available_bytes=$((disk_available * 1024))
    if [[ $disk_available_bytes -lt $MIN_DISK_SPACE ]]; then
        HEALTH_ISSUES+=("Low disk space: $(($disk_available_bytes / 1024 / 1024))MB available")
        OVERALL_STATUS="DEGRADED"
    fi
    
    log "INFO" "Resource usage - Memory: ${memory_usage}%, CPU: ${cpu_usage}%, Disk: $(($disk_available_bytes / 1024 / 1024))MB"
}

# Check database connectivity
check_database_health() {
    log "INFO" "Checking database connectivity..."
    
    if [[ -f "/app/data/cursor-agents.db" ]]; then
        # Test database connection
        local db_check=$(timeout 5 sqlite3 /app/data/cursor-agents.db "SELECT 1;" 2>/dev/null || echo "ERROR")
        if [[ "$db_check" == "1" ]]; then
            log "INFO" "Database connectivity check passed"
        else
            log "ERROR" "Database connectivity check failed"
            HEALTH_ISSUES+=("Database connectivity failed")
            OVERALL_STATUS="UNHEALTHY"
        fi
    else
        log "ERROR" "Database file not found"
        HEALTH_ISSUES+=("Database file missing")
        OVERALL_STATUS="UNHEALTHY"
    fi
}

# Check log files (skip in quick mode)
check_log_health() {
    if [[ "$QUICK_MODE" == "true" ]]; then
        return 0
    fi
    
    log "INFO" "Checking log files..."
    
    local log_files=("cursor-agents.log" "cursor-metrics.log" "cursor-errors.log")
    
    for log_file in "${log_files[@]}"; do
        local log_path="$CURSOR_LOG_DIR/$log_file"
        if [[ -f "$log_path" ]]; then
            local log_size=$(stat -c%s "$log_path" 2>/dev/null || echo "0")
            local log_size_mb=$((log_size / 1024 / 1024))
            
            if [[ $log_size_mb -gt 100 ]]; then
                HEALTH_ISSUES+=("Log file $log_file is large: ${log_size_mb}MB")
                OVERALL_STATUS="DEGRADED"
            fi
            
            # Check for recent errors
            local recent_errors=$(tail -n 100 "$log_path" 2>/dev/null | grep -i error | wc -l)
            if [[ $recent_errors -gt 5 ]]; then
                HEALTH_ISSUES+=("High error rate in $log_file: $recent_errors errors")
                OVERALL_STATUS="DEGRADED"
            fi
        else
            log "WARN" "Log file $log_file not found"
        fi
    done
}

# Generate health report (skip in quick mode)
generate_health_report() {
    if [[ "$QUICK_MODE" == "true" ]]; then
        return 0
    fi
    
    log "INFO" "Generating health report..."
    
    local report_file="$CURSOR_LOG_DIR/health-report.json"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    cat > "$report_file" <<EOF
{
    "timestamp": "$timestamp",
    "overall_status": "$OVERALL_STATUS",
    "health_issues": [
        $(printf '"%s",' "${HEALTH_ISSUES[@]}" | sed 's/,$//')
    ],
    "checks": {
        "ownership": "$(check_ownership_status)",
        "api_health": "$(check_api_health_status)",
        "websocket_health": "$(check_websocket_health_status)",
        "agent_status": "$(check_agent_status_health)",
        "resource_usage": "$(check_resource_usage_health)",
        "database_health": "$(check_database_health_status)",
        "log_health": "$(check_log_health_status)"
    }
}
EOF
    
    log "INFO" "Health report generated: $report_file"
}

# Print health summary (skip in quick mode)
print_health_summary() {
    if [[ "$QUICK_MODE" == "true" ]]; then
        return 0
    fi
    
    echo
    echo "=================================="
    echo "   CURSOR AGENT HEALTH SUMMARY"
    echo "=================================="
    echo
    
    case "$OVERALL_STATUS" in
        "HEALTHY")
            echo -e "Overall Status: ${GREEN}HEALTHY${NC}"
            ;;
        "DEGRADED")
            echo -e "Overall Status: ${YELLOW}DEGRADED${NC}"
            ;;
        "UNHEALTHY")
            echo -e "Overall Status: ${RED}UNHEALTHY${NC}"
            ;;
    esac
    
    echo
    
    if [[ ${#HEALTH_ISSUES[@]} -gt 0 ]]; then
        echo "Health Issues:"
        for issue in "${HEALTH_ISSUES[@]}"; do
            echo -e "  ${RED}•${NC} $issue"
        done
    else
        echo -e "${GREEN}No health issues detected${NC}"
    fi
    
    echo
    echo "=================================="
}

# Display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --quick      Quick health check (minimal output, faster execution)"
    echo "  --help       Show this help message"
    echo ""
    echo "Exit codes:"
    echo "  0 - Healthy"
    echo "  1 - Degraded"
    echo "  2 - Unhealthy"
}

# Main health check function
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick)
                QUICK_MODE=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    log "INFO" "Starting cursor agent health check (quick mode: $QUICK_MODE)..."
    
    # Run health checks
    check_ownership
    check_api_health
    check_websocket_health
    check_agent_status
    check_resource_usage
    check_database_health
    check_log_health
    
    generate_health_report
    print_health_summary
    
    log "INFO" "Health check completed with status: $OVERALL_STATUS"
    
    # Exit with appropriate code
    case "$OVERALL_STATUS" in
        "HEALTHY")
            exit 0
            ;;
        "DEGRADED")
            exit 1
            ;;
        "UNHEALTHY")
            exit 2
            ;;
    esac
}

# Helper functions for status checks
check_ownership_status() {
    local lock_file="$CURSOR_LOCK_DIR/cursor-agent.lock"
    if [[ -f "$lock_file" ]]; then
        local owner_pid=$(cat "$lock_file" | cut -d'-' -f2 2>/dev/null || echo "0")
        if [[ "$owner_pid" != "0" ]] && kill -0 "$owner_pid" 2>/dev/null; then
            echo "PASS"
        else
            echo "STALE"
        fi
    else
        echo "MISSING"
    fi
}

check_api_health_status() {
    local response_code=$(curl -s -o /dev/null -w "%{http_code}" \
        "http://${CURSOR_API_HOST}:${CURSOR_API_PORT}/health/cursor" \
        --connect-timeout 5 --max-time 10 2>/dev/null || echo "000")
    [[ "$response_code" == "200" ]] && echo "PASS" || echo "FAIL"
}

check_websocket_health_status() {
    timeout 3 bash -c "echo >/dev/tcp/${CURSOR_API_HOST}/${CURSOR_WS_PORT}" 2>/dev/null && echo "PASS" || echo "FAIL"
}

check_agent_status_health() {
    local agent_status=$(curl -s "http://${CURSOR_API_HOST}:${CURSOR_MGMT_PORT}/agents/status" \
        --connect-timeout 5 --max-time 10 2>/dev/null || echo "ERROR")
    [[ "$agent_status" != "ERROR" ]] && echo "PASS" || echo "FAIL"
}

check_resource_usage_health() {
    local memory_usage=$(free | grep Mem | awk '{printf "%.0f", $3/$2 * 100.0}' 2>/dev/null || echo "0")
    [[ $memory_usage -le $MAX_MEMORY_USAGE ]] && echo "PASS" || echo "FAIL"
}

check_database_health_status() {
    local db_check=$(timeout 5 sqlite3 /app/data/cursor-agents.db "SELECT 1;" 2>/dev/null || echo "ERROR")
    [[ "$db_check" == "1" ]] && echo "PASS" || echo "FAIL"
}

check_log_health_status() {
    [[ -f "$CURSOR_LOG_DIR/cursor-agents.log" ]] && echo "PASS" || echo "FAIL"
}

# Run main function
main "$@" 