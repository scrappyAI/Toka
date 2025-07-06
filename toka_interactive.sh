#!/bin/bash

# Toka Interactive CLI - Stateful Agent Management
# Uses the actual toka CLI for interactive agent operations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
TOKA_CLI="./target/release/toka"
TOKA_CONFIG="./target/release/toka-config"
TOKA_ORCHESTRATION="./target/release/toka-orchestration"
DB_PATH="./data/toka.db"
DAEMON_PID_FILE=".toka_daemon.pid"
LOG_FILE="./logs/toka-interactive.log"

# Ensure directories exist
mkdir -p data logs

# Function to print colored output
print_status() {
    echo -e "${1}${2}${NC}"
}

# Function to print section headers
print_section() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}════════════════════════════════════════${NC}"
}

# Function to print menu options
print_menu() {
    echo ""
    echo -e "${BLUE}Available Commands:${NC}"
    echo -e "${YELLOW}1.${NC} Start Daemon Mode"
    echo -e "${YELLOW}2.${NC} Stop Daemon Mode"
    echo -e "${YELLOW}3.${NC} Spawn Agent"
    echo -e "${YELLOW}4.${NC} Schedule Task"
    echo -e "${YELLOW}5.${NC} Query State"
    echo -e "${YELLOW}6.${NC} Generate Token"
    echo -e "${YELLOW}7.${NC} Check System Status"
    echo -e "${YELLOW}8.${NC} View Logs"
    echo -e "${YELLOW}9.${NC} Configuration Management"
    echo -e "${YELLOW}10.${NC} Run Tests"
    echo -e "${YELLOW}0.${NC} Exit"
    echo ""
}

# Function to check if CLI binaries exist
check_binaries() {
    local missing=false
    
    if [ ! -f "$TOKA_CLI" ]; then
        print_status $RED "❌ Toka CLI not found at $TOKA_CLI"
        missing=true
    fi
    
    if [ ! -f "$TOKA_CONFIG" ]; then
        print_status $RED "❌ Toka Config CLI not found at $TOKA_CONFIG"
        missing=true
    fi
    
    if [ ! -f "$TOKA_ORCHESTRATION" ]; then
        print_status $RED "❌ Toka Orchestration not found at $TOKA_ORCHESTRATION"
        missing=true
    fi
    
    if [ "$missing" = true ]; then
        print_status $YELLOW "Please run the setup script first: ./setup_toka_testing.sh"
        exit 1
    fi
    
    print_status $GREEN "✅ All Toka binaries found"
}

# Function to check if daemon is running
is_daemon_running() {
    if [ -f "$DAEMON_PID_FILE" ]; then
        local pid=$(cat "$DAEMON_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        else
            rm -f "$DAEMON_PID_FILE"
            return 1
        fi
    fi
    return 1
}

# Function to start daemon mode
start_daemon() {
    print_section "Starting Toka Daemon"
    
    if is_daemon_running; then
        print_status $YELLOW "⚠️  Daemon is already running"
        return 0
    fi
    
    print_status $BLUE "🚀 Starting Toka daemon with SQLite storage..."
    
    # Start daemon in background
    nohup $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        --log-level info \
        daemon \
        > "$LOG_FILE" 2>&1 &
    
    echo $! > "$DAEMON_PID_FILE"
    
    # Wait a moment for daemon to start
    sleep 2
    
    if is_daemon_running; then
        print_status $GREEN "✅ Daemon started successfully (PID: $(cat $DAEMON_PID_FILE))"
        print_status $BLUE "📊 Database: $DB_PATH"
        print_status $BLUE "📝 Logs: $LOG_FILE"
    else
        print_status $RED "❌ Failed to start daemon"
        rm -f "$DAEMON_PID_FILE"
        return 1
    fi
}

# Function to stop daemon mode
stop_daemon() {
    print_section "Stopping Toka Daemon"
    
    if ! is_daemon_running; then
        print_status $YELLOW "⚠️  Daemon is not running"
        return 0
    fi
    
    local pid=$(cat "$DAEMON_PID_FILE")
    print_status $BLUE "🛑 Stopping daemon (PID: $pid)..."
    
    kill "$pid" 2>/dev/null || true
    rm -f "$DAEMON_PID_FILE"
    
    # Wait for process to stop
    sleep 2
    
    if ! is_daemon_running; then
        print_status $GREEN "✅ Daemon stopped successfully"
    else
        print_status $YELLOW "⚠️  Daemon may still be running"
    fi
}

# Function to spawn an agent
spawn_agent() {
    print_section "Spawning Agent"
    
    echo -n "Enter agent name: "
    read -r agent_name
    
    if [ -z "$agent_name" ]; then
        print_status $RED "❌ Agent name cannot be empty"
        return 1
    fi
    
    print_status $BLUE "🤖 Spawning agent: $agent_name"
    
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        --log-level info \
        spawn-agent \
        --name "$agent_name"
    
    if [ $? -eq 0 ]; then
        print_status $GREEN "✅ Agent '$agent_name' spawned successfully"
    else
        print_status $RED "❌ Failed to spawn agent '$agent_name'"
    fi
}

# Function to schedule a task
schedule_task() {
    print_section "Scheduling Task"
    
    echo -n "Enter agent ID (numeric): "
    read -r agent_id
    
    if [ -z "$agent_id" ] || ! [[ "$agent_id" =~ ^[0-9]+$ ]]; then
        print_status $RED "❌ Agent ID must be a number"
        return 1
    fi
    
    echo -n "Enter task description: "
    read -r task_description
    
    if [ -z "$task_description" ]; then
        print_status $RED "❌ Task description cannot be empty"
        return 1
    fi
    
    print_status $BLUE "📋 Scheduling task for agent $agent_id: $task_description"
    
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        --log-level info \
        schedule-task \
        --agent "$agent_id" \
        --description "$task_description"
    
    if [ $? -eq 0 ]; then
        print_status $GREEN "✅ Task scheduled successfully"
    else
        print_status $RED "❌ Failed to schedule task"
    fi
}

# Function to query system state
query_state() {
    print_section "Querying System State"
    
    print_status $BLUE "🔍 Querying current system state..."
    
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        --log-level info \
        query-state
    
    if [ $? -eq 0 ]; then
        print_status $GREEN "✅ State query completed"
    else
        print_status $RED "❌ Failed to query state"
    fi
}

# Function to generate a development token
generate_token() {
    print_section "Generating Development Token"
    
    echo -n "Enter subject [dev-user]: "
    read -r subject
    subject=${subject:-dev-user}
    
    echo -n "Enter vault [dev-vault]: "
    read -r vault
    vault=${vault:-dev-vault}
    
    echo -n "Enter permissions [read,write]: "
    read -r permissions
    permissions=${permissions:-read,write}
    
    print_status $BLUE "🔑 Generating JWT token..."
    
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        --log-level info \
        generate-token \
        --subject "$subject" \
        --vault "$vault" \
        --permissions "$permissions"
    
    if [ $? -eq 0 ]; then
        print_status $GREEN "✅ Token generated successfully"
    else
        print_status $RED "❌ Failed to generate token"
    fi
}

# Function to check system status
check_status() {
    print_section "System Status"
    
    print_status $BLUE "🔍 Checking system components..."
    
    # Check daemon status
    if is_daemon_running; then
        print_status $GREEN "✅ Daemon: Running (PID: $(cat $DAEMON_PID_FILE))"
    else
        print_status $YELLOW "⚠️  Daemon: Not running"
    fi
    
    # Check database
    if [ -f "$DB_PATH" ]; then
        local db_size=$(du -h "$DB_PATH" | cut -f1)
        print_status $GREEN "✅ Database: $DB_PATH ($db_size)"
    else
        print_status $YELLOW "⚠️  Database: Not found"
    fi
    
    # Check logs
    if [ -f "$LOG_FILE" ]; then
        local log_lines=$(wc -l < "$LOG_FILE")
        print_status $GREEN "✅ Logs: $LOG_FILE ($log_lines lines)"
    else
        print_status $YELLOW "⚠️  Logs: Not found"
    fi
    
    # Check configuration
    if [ -f "config/testing/agents.toml" ]; then
        print_status $GREEN "✅ Configuration: config/testing/agents.toml"
    else
        print_status $YELLOW "⚠️  Configuration: Not found"
    fi
}

# Function to view logs
view_logs() {
    print_section "Viewing Logs"
    
    if [ ! -f "$LOG_FILE" ]; then
        print_status $YELLOW "⚠️  No log file found at $LOG_FILE"
        return 1
    fi
    
    echo -n "Show last N lines [20]: "
    read -r lines
    lines=${lines:-20}
    
    if ! [[ "$lines" =~ ^[0-9]+$ ]]; then
        lines=20
    fi
    
    print_status $BLUE "📝 Showing last $lines lines from $LOG_FILE:"
    echo ""
    tail -n "$lines" "$LOG_FILE"
    echo ""
    print_status $BLUE "📍 Full log path: $LOG_FILE"
}

# Function for configuration management
config_management() {
    print_section "Configuration Management"
    
    print_status $BLUE "Configuration Management Options:"
    echo "1. List configuration files"
    echo "2. Create configuration file"
    echo "3. Read configuration file"
    echo "4. Update configuration value"
    echo "5. Validate configuration file"
    echo "0. Back to main menu"
    
    echo -n "Choose option [0-5]: "
    read -r config_choice
    
    case "$config_choice" in
        1)
            print_status $BLUE "📂 Listing configuration files..."
            $TOKA_CONFIG list --directory config/
            ;;
        2)
            echo -n "Enter file path: "
            read -r file_path
            echo -n "Enter format [yaml/json/toml]: "
            read -r format
            echo -n "Enter initial content [{}]: "
            read -r content
            content=${content:-{}}
            
            $TOKA_CONFIG create --file "$file_path" --format "$format" --content "$content"
            ;;
        3)
            echo -n "Enter file path: "
            read -r file_path
            
            $TOKA_CONFIG read --file "$file_path"
            ;;
        4)
            echo -n "Enter file path: "
            read -r file_path
            echo -n "Enter key (use dot notation for nested): "
            read -r key
            echo -n "Enter value (JSON format): "
            read -r value
            
            $TOKA_CONFIG update --file "$file_path" --key "$key" --value "$value"
            ;;
        5)
            echo -n "Enter file path: "
            read -r file_path
            
            $TOKA_CONFIG validate --file "$file_path"
            ;;
        0)
            return 0
            ;;
        *)
            print_status $RED "❌ Invalid option"
            ;;
    esac
}

# Function to run tests
run_tests() {
    print_section "Running Tests"
    
    print_status $BLUE "🧪 Running Toka agent tests..."
    
    # Create test files
    mkdir -p data/tests
    
    # Test 1: Create a test input file
    cat > data/tests/test_input.txt << 'EOF'
# Test Input File

This is a test file for Toka agent functionality.
It contains structured content for processing.

Key features:
1. Multiple sections
2. Structured data
3. Test validation

The agents should process this file successfully.
EOF
    
    # Test 2: Generate a token
    print_status $BLUE "🔑 Generating test token..."
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        generate-token \
        --subject "test-user" \
        --vault "test-vault" \
        --permissions "read,write" \
        > data/tests/test_token.txt
    
    # Test 3: Query state
    print_status $BLUE "🔍 Querying system state..."
    $TOKA_CLI \
        --storage sqlite \
        --db-path "$DB_PATH" \
        query-state > data/tests/state_output.txt
    
    # Test 4: Configuration validation
    if [ -f "config/testing/agents.toml" ]; then
        print_status $BLUE "⚙️  Validating configuration..."
        $TOKA_CONFIG validate --file "config/testing/agents.toml"
    fi
    
    print_status $GREEN "✅ Tests completed - results saved in data/tests/"
    echo ""
    echo "Generated files:"
    ls -la data/tests/
}

# Function to show help
show_help() {
    print_section "Toka Interactive CLI Help"
    
    echo "This interactive CLI provides a stateful interface to the Toka agentic system."
    echo ""
    echo "Key Features:"
    echo "• Stateful operation with persistent SQLite storage"
    echo "• Interactive agent spawning and task scheduling"
    echo "• Real-time system state queries"
    echo "• Configuration management"
    echo "• Development token generation"
    echo "• Daemon mode for continuous operation"
    echo ""
    echo "Storage: $DB_PATH"
    echo "Logs: $LOG_FILE"
    echo ""
    echo "For direct CLI usage:"
    echo "  $TOKA_CLI --help"
    echo "  $TOKA_CONFIG --help"
}

# Main interactive loop
main_loop() {
    while true; do
        print_menu
        echo -n "Choose option [0-10]: "
        read -r choice
        
        case "$choice" in
            1)
                start_daemon
                ;;
            2)
                stop_daemon
                ;;
            3)
                spawn_agent
                ;;
            4)
                schedule_task
                ;;
            5)
                query_state
                ;;
            6)
                generate_token
                ;;
            7)
                check_status
                ;;
            8)
                view_logs
                ;;
            9)
                config_management
                ;;
            10)
                run_tests
                ;;
            0)
                print_status $BLUE "👋 Exiting Toka Interactive CLI"
                stop_daemon
                break
                ;;
            "help" | "h")
                show_help
                ;;
            *)
                print_status $RED "❌ Invalid option. Try again or type 'help'"
                ;;
        esac
        
        echo ""
        echo -n "Press Enter to continue..."
        read -r
    done
}

# Main script execution
main() {
    print_section "Toka Interactive CLI"
    print_status $BLUE "🚀 Welcome to the Toka Agentic Operating System"
    
    # Check if binaries exist
    check_binaries
    
    # Show initial status
    check_status
    
    # Show help
    show_help
    
    # Start interactive loop
    main_loop
    
    print_status $GREEN "✅ Toka Interactive CLI session ended"
}

# Run main function
main "$@" 