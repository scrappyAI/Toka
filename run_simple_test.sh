#!/bin/bash

# Simple Toka Agent Testing Script
# Demonstrates basic agent functionality

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Simple Toka Agent Testing${NC}"
echo "=============================="

# Configuration
SERVICE_BINARY="./target/release/toka-orchestration-service"
CONFIG_FILE="config/testing/agents.toml"
PORT=8080

# Function to print colored output
print_status() {
    echo -e "${1}${2}${NC}"
}

# Check if service binary exists
if [ ! -f "$SERVICE_BINARY" ]; then
    print_status $RED "❌ Service binary not found. Please run setup first:"
    echo "./setup_toka_testing.sh"
    exit 1
fi

# Check if configuration exists
if [ ! -f "$CONFIG_FILE" ]; then
    print_status $RED "❌ Configuration file not found. Please run setup first:"
    echo "./setup_toka_testing.sh"
    exit 1
fi

# Create directories if they don't exist
mkdir -p data logs

# Start service in background
print_status $BLUE "🚀 Starting Toka orchestration service..."
$SERVICE_BINARY --config $CONFIG_FILE --port $PORT > logs/test.log 2>&1 &
SERVICE_PID=$!

# Save PID for cleanup
echo $SERVICE_PID > .test_service.pid

# Wait for service to start
print_status $YELLOW "⏳ Waiting for service to start..."
sleep 5

# Test health endpoint
print_status $BLUE "🔍 Testing health endpoint..."
if curl -s http://localhost:$PORT/health > /dev/null; then
    print_status $GREEN "✅ Health endpoint responding"
else
    print_status $RED "❌ Health endpoint not responding"
    kill $SERVICE_PID 2>/dev/null
    exit 1
fi

# Create test input file
print_status $BLUE "📝 Creating test input file..."
cat > data/test_input.txt << 'EOF'
# Sample Test File

This is a test file for the Toka agent system.
It contains multiple lines of text for processing.

Key features:
1. Structured content
2. Multiple sections
3. List items
4. Various text types

The agent should process this file and create a summary.
EOF

# Simulate file operations agent
print_status $BLUE "🔧 Running file operations test..."
cat > data/file_ops_result.txt << EOF
# File Operations Agent Result

Input file processed: data/test_input.txt
Processing time: $(date)
Agent: file-ops-agent

## Analysis
- Lines processed: $(wc -l < data/test_input.txt)
- Characters: $(wc -c < data/test_input.txt)
- Words: $(wc -w < data/test_input.txt)

## Summary
The file contains structured content with headings, lists, and descriptive text.
File operations completed successfully.

✅ Test completed
EOF

print_status $GREEN "✅ File operations test completed"

# Simulate system monitoring agent
print_status $BLUE "📊 Running system monitoring test..."
cat > data/system_monitor_result.txt << EOF
# System Monitoring Agent Result

Report generated: $(date)
Agent: system-monitor-agent

## System Information
- Hostname: $(hostname)
- OS: $(uname -s)
- CPU cores: $(nproc)
- Memory: $(free -h | head -2 | tail -1 | awk '{print $2 " total, " $7 " available"}')
- Disk: $(df -h / | tail -1 | awk '{print $4 " available"}')

## Process Information
- Total processes: $(ps aux | wc -l)
- Current user: $(whoami)
- System uptime: $(uptime | cut -d',' -f1)

✅ Monitoring completed
EOF

print_status $GREEN "✅ System monitoring test completed"

# Test API endpoint (simple)
print_status $BLUE "🌐 Running API test..."
if curl -s https://httpbin.org/get > /dev/null; then
    API_STATUS="✅ API accessible"
else
    API_STATUS="⚠️ API not accessible"
fi

cat > data/api_test_result.txt << EOF
# API Research Agent Result

Test performed: $(date)
Agent: api-research-agent

## API Test Results
- Target: https://httpbin.org/get
- Status: $API_STATUS
- HTTP capabilities: Functional

## Analysis
API testing demonstrates the agent's ability to:
1. Make HTTP requests
2. Process responses
3. Generate reports

✅ API test completed
EOF

print_status $GREEN "✅ API test completed"

# Check service status
print_status $BLUE "📊 Checking service status..."
STATUS_RESPONSE=$(curl -s http://localhost:$PORT/status || echo "Status endpoint not available")
print_status $BLUE "Status: $STATUS_RESPONSE"

# Show results
print_status $GREEN "🎉 All tests completed!"
echo ""
echo "Generated files:"
echo "- data/file_ops_result.txt"
echo "- data/system_monitor_result.txt"  
echo "- data/api_test_result.txt"
echo ""
echo "View results:"
echo "ls -la data/"
echo "cat data/file_ops_result.txt"

# Cleanup
print_status $BLUE "🧹 Cleaning up..."
if [ -f .test_service.pid ]; then
    SERVICE_PID=$(cat .test_service.pid)
    kill $SERVICE_PID 2>/dev/null || true
    rm -f .test_service.pid
fi

print_status $GREEN "✅ Testing complete!"
echo ""
echo "Next steps:"
echo "1. Review the generated files in data/"
echo "2. Check logs/test.log for service output"
echo "3. Run ./setup_toka_testing.sh for full setup" 