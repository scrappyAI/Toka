#!/bin/bash

# Toka Workflow - Proper Usage of the Rust CLI
# This script demonstrates the corrected agentic system workflow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DB_PATH="./data/toka-workflow.db"
LOG_LEVEL="info"
JWT_SECRET="toka-workflow-secret-change-in-production"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Toka Workflow - Demonstrating the Agentic System${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo

# Create data directory
mkdir -p data logs

# Build the required binaries
echo -e "${YELLOW}🔧 Building Toka binaries...${NC}"
if ! cargo build --release --bin toka --bin toka-config --bin toka-test; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build completed${NC}"
echo

# Clean up previous database
if [ -f "$DB_PATH" ]; then
    echo -e "${YELLOW}🗑️  Cleaning up previous database...${NC}"
    rm -f "$DB_PATH"
fi

# Step 1: Generate authentication tokens
echo -e "${BLUE}Step 1: Generate Authentication Tokens${NC}"
echo -e "${CYAN}----------------------------------------${NC}"

echo -e "${YELLOW}🔑 Generating admin token...${NC}"
ADMIN_TOKEN=$(./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    generate-token \
    --subject "0" \
    --vault "system-vault" \
    --permissions "read,write,admin" | grep "Token:" | sed 's/.*Token: //')

echo -e "${YELLOW}🔑 Generating user token...${NC}"
USER_TOKEN=$(./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    generate-token \
    --subject "1000" \
    --vault "user-vault" \
    --permissions "read,write" | grep "Token:" | sed 's/.*Token: //')

echo -e "${GREEN}✅ Tokens generated successfully${NC}"
echo

# Step 2: Spawn agents using proper authentication
echo -e "${BLUE}Step 2: Spawn Agents with Authentication${NC}"
echo -e "${CYAN}----------------------------------------${NC}"

echo -e "${YELLOW}🤖 Spawning FileAgent...${NC}"
./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    spawn-agent \
    --name "FileAgent" \
    --token "$ADMIN_TOKEN"

echo -e "${YELLOW}🤖 Spawning MonitorAgent...${NC}"
./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    spawn-agent \
    --name "MonitorAgent" \
    --token "$ADMIN_TOKEN"

echo -e "${GREEN}✅ Agents spawned successfully${NC}"
echo

# Step 3: Schedule tasks for agents
echo -e "${BLUE}Step 3: Schedule Tasks${NC}"
echo -e "${CYAN}----------------------${NC}"

echo -e "${YELLOW}📋 Scheduling tasks for FileAgent...${NC}"
./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    schedule-task \
    --agent 1 \
    --description "Read and analyze project configuration files" \
    --token "$ADMIN_TOKEN"

echo -e "${YELLOW}📋 Scheduling tasks for MonitorAgent...${NC}"
./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    schedule-task \
    --agent 2 \
    --description "Monitor system resource usage and generate report" \
    --token "$ADMIN_TOKEN"

echo -e "${GREEN}✅ Tasks scheduled successfully${NC}"
echo

# Step 4: Query system state
echo -e "${BLUE}Step 4: Query System State${NC}"
echo -e "${CYAN}---------------------------${NC}"

echo -e "${YELLOW}🌍 Querying current world state...${NC}"
./target/release/toka \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    query-state

echo

# Step 5: Demonstrate configuration management
echo -e "${BLUE}Step 5: Configuration Management${NC}"
echo -e "${CYAN}--------------------------------${NC}"

echo -e "${YELLOW}⚙️  Creating test configuration...${NC}"
./target/release/toka-config create \
    --file ./data/test-config.json \
    --format json \
    --content '{"agents": {"FileAgent": {"enabled": true}, "MonitorAgent": {"enabled": true}}, "settings": {"log_level": "info", "storage": "sqlite"}}'

echo -e "${YELLOW}📖 Reading configuration...${NC}"
./target/release/toka-config read --file ./data/test-config.json

echo -e "${YELLOW}✏️  Updating configuration...${NC}"
./target/release/toka-config update \
    --file ./data/test-config.json \
    --key settings.log_level \
    --value '"debug"'

echo -e "${YELLOW}✅ Validating configuration...${NC}"
./target/release/toka-config validate --file ./data/test-config.json

echo -e "${GREEN}✅ Configuration management completed${NC}"
echo

# Step 6: Run the interactive testing environment
echo -e "${BLUE}Step 6: Interactive Testing Environment${NC}"
echo -e "${CYAN}--------------------------------------${NC}"

echo -e "${YELLOW}🎮 Running demo scenarios...${NC}"
./target/release/toka-test \
    --storage sqlite \
    --db-path "$DB_PATH" \
    --log-level "$LOG_LEVEL" \
    --jwt-secret "$JWT_SECRET" \
    --demo

echo

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Workflow completed successfully!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo
echo -e "${CYAN}What was demonstrated:${NC}"
echo -e "  🔑 Proper JWT token generation and authentication"
echo -e "  🤖 Agent spawning with authenticated operations"
echo -e "  📋 Task scheduling with proper authorization"
echo -e "  🌍 System state querying"
echo -e "  ⚙️  Configuration management (JSON/YAML/TOML)"
echo -e "  🎮 Interactive testing environment"
echo
echo -e "${CYAN}Generated files:${NC}"
echo -e "  📄 Database: ${DB_PATH}"
echo -e "  📄 Config: ./data/test-config.json"
echo
echo -e "${CYAN}Next steps:${NC}"
echo -e "  🎮 Run interactive mode: ./target/release/toka-test"
echo -e "  🚀 Start daemon mode: ./target/release/toka daemon"
echo -e "  📚 Explore CLI help: ./target/release/toka --help"
echo 