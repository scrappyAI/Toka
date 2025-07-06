#!/bin/bash

# Toka Agentic System - First-Time Setup Script
# This script sets up a minimal testing environment for Toka OS

set -e

echo "🚀 Setting up Toka Agentic System Testing Environment"
echo "=================================================="

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to prompt for input with default
prompt_with_default() {
    local prompt="$1"
    local default="$2"
    local var_name="$3"
    
    echo -n "$prompt [$default]: "
    read -r input
    if [[ -z "$input" ]]; then
        export "$var_name"="$default"
    else
        export "$var_name"="$input"
    fi
}

# Function to prompt for sensitive input (no echo)
prompt_sensitive() {
    local prompt="$1"
    local var_name="$2"
    
    echo -n "$prompt: "
    read -s input
    echo
    export "$var_name"="$input"
}

echo "Step 1: Checking Prerequisites"
echo "------------------------------"

# Check Rust installation
if ! command_exists cargo; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust/Cargo found: $(cargo --version)"

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "✅ Rust version: $RUST_VERSION"

echo ""
echo "Step 2: LLM Provider Configuration"
echo "-----------------------------------"

echo "Toka supports multiple LLM providers. Choose one:"
echo "1. Anthropic Claude (recommended)"
echo "2. OpenAI GPT-4"
echo "3. Skip LLM setup (limited functionality)"

while true; do
    echo -n "Enter your choice (1-3): "
    read -r choice
    case $choice in
        1)
            LLM_PROVIDER="anthropic"
            LLM_MODEL="claude-3-5-sonnet-20241022"
            echo "Selected: Anthropic Claude"
            echo ""
            echo "Please get your API key from: https://console.anthropic.com/"
            prompt_sensitive "Enter your Anthropic API key" ANTHROPIC_API_KEY
            if [[ -z "$ANTHROPIC_API_KEY" ]]; then
                echo "❌ API key is required for LLM functionality"
                exit 1
            fi
            echo "✅ Anthropic API key configured"
            break
            ;;
        2)
            LLM_PROVIDER="openai"
            LLM_MODEL="gpt-4"
            echo "Selected: OpenAI GPT-4"
            echo ""
            echo "Please get your API key from: https://platform.openai.com/api-keys"
            prompt_sensitive "Enter your OpenAI API key" OPENAI_API_KEY
            if [[ -z "$OPENAI_API_KEY" ]]; then
                echo "❌ API key is required for LLM functionality"
                exit 1
            fi
            echo "✅ OpenAI API key configured"
            break
            ;;
        3)
            echo "⚠️  Skipping LLM setup - agents will have limited functionality"
            LLM_PROVIDER=""
            break
            ;;
        *)
            echo "Invalid choice. Please enter 1, 2, or 3."
            ;;
    esac
done

echo ""
echo "Step 3: Environment Configuration"
echo "---------------------------------"

# Create .env file
echo "Creating .env file..."
cat > .env << EOF
# Toka Testing Environment Configuration
# Generated on $(date)

# =============================================================================
# LLM Provider Configuration
# =============================================================================
EOF

if [[ "$LLM_PROVIDER" == "anthropic" ]]; then
    echo "ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY" >> .env
elif [[ "$LLM_PROVIDER" == "openai" ]]; then
    echo "OPENAI_API_KEY=$OPENAI_API_KEY" >> .env
fi

if [[ -n "$LLM_PROVIDER" ]]; then
    cat >> .env << EOF
LLM_PROVIDER=$LLM_PROVIDER
LLM_MODEL=$LLM_MODEL
LLM_RATE_LIMIT=50
LLM_TIMEOUT=30
LLM_DEBUG=false
EOF
fi

cat >> .env << EOF

# =============================================================================
# Database Configuration
# =============================================================================
DATABASE_URL=sqlite:///app/data/agents.db
STORAGE_TYPE=sqlite

# =============================================================================
# Agent Orchestration Settings
# =============================================================================
AGENT_POOL_SIZE=3
MAX_CONCURRENT_AGENTS=2
AGENT_SPAWN_TIMEOUT=30
WORKSTREAM_TIMEOUT=1800

# =============================================================================
# Development Settings
# =============================================================================
RUST_LOG=info
RUST_BACKTRACE=1
TOKIO_WORKER_THREADS=2

# Security
JWT_SECRET=toka-testing-secret-$(date +%s)
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict

# Monitoring
METRICS_ENABLED=true
TRACING_ENABLED=true
LOG_LEVEL=info

# Development directories
AGENT_DATA_DIR=./data
AGENT_LOG_DIR=./logs
AGENT_CONFIG_DIR=./config

# Testing mode
AGENT_DEV_MODE=true
AGENT_DEBUG_ENABLED=true
EOF

echo "✅ Environment file created (.env)"

echo ""
echo "Step 4: Directory Setup"
echo "----------------------"

# Create required directories
mkdir -p data logs config/testing

echo "✅ Required directories created"

echo ""
echo "Step 5: Building Toka System"
echo "----------------------------"

echo "Building the Toka CLI and orchestration components..."
if cargo build --release --bin toka --bin toka-config --bin toka-orchestration; then
    echo "✅ Toka CLI and orchestration components built successfully"
else
    echo "❌ Build failed. Please check the errors above."
    exit 1
fi

echo ""
echo "Step 6: Creating Test Agent Configurations"
echo "-----------------------------------------"

# Create test agent configurations
cat > config/testing/agents.toml << 'EOF'
# Toka Testing Agent Configuration
# Minimal configuration for testing basic functionality

[orchestration]
max_concurrent_agents = 2
agent_spawn_timeout = 30
workstream_timeout = 1800
agent_pool_size = 3

[security]
sandbox_enabled = true
capability_validation = "strict"

[security.resource_limits]
max_memory = "256MB"
max_cpu = "0.5"
timeout = "900"

[storage]
backend = "sqlite"
database_url = "sqlite:///app/data/agents.db"
persistence_buffer_size = 128
persistence_interval = 30

[llm]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
rate_limit = 50
timeout = 30
debug_mode = false

[monitoring]
metrics_enabled = true
tracing_enabled = true
log_level = "info"

[caching]
context_cache_size = 1024
context_cache_ttl = 3600
response_cache_enabled = true
response_cache_size = 512
response_cache_ttl = 1800

# Test Agent 1: File Operations
[[agents]]
name = "file-ops-agent"
version = "1.0.0"
domain = "file-operations"
priority = "high"
workstream = "testing"
branch = "main"

[agents.capabilities]
primary = ["filesystem-read", "filesystem-write", "text-processing"]
secondary = ["file-manipulation", "content-analysis"]

[[agents.objectives]]
description = "Test file reading and writing operations"
deliverable = "Processed file with analysis results"
validation = "Output file contains expected content"

[[agents.tasks.default]]
description = "Read input file, process content, and write summary"
priority = "high"
dependencies = []

# Test Agent 2: System Monitoring
[[agents]]
name = "system-monitor-agent"
version = "1.0.0"
domain = "system-monitoring"
priority = "high"
workstream = "testing"
branch = "main"

[agents.capabilities]
primary = ["system-monitoring", "command-execution", "report-generation"]
secondary = ["resource-tracking", "status-reporting"]

[[agents.objectives]]
description = "Monitor system status and generate reports"
deliverable = "System status report"
validation = "Report contains system metrics"

[[agents.tasks.default]]
description = "Check system status and create monitoring report"
priority = "high"
dependencies = []

# Test Agent 3: API Research
[[agents]]
name = "api-research-agent"
version = "1.0.0"
domain = "api-research"
priority = "high"
workstream = "testing"
branch = "main"

[agents.capabilities]
primary = ["http-requests", "data-processing", "json-parsing"]
secondary = ["api-integration", "data-analysis"]

[[agents.objectives]]
description = "Fetch data from public API and analyze results"
deliverable = "API data analysis report"
validation = "Report contains fetched and processed data"

[[agents.tasks.default]]
description = "Fetch data from JSONPlaceholder API and generate analysis"
priority = "high"
dependencies = []
EOF

echo "✅ Test agent configurations created"

echo ""
echo "Step 7: Creating Test Input Files"
echo "--------------------------------"

# Create test input file for file operations agent
cat > data/test_input.txt << 'EOF'
# Test Input File for Toka Agent Testing

This is a sample text file that will be processed by the file operations agent.
The agent should read this file, analyze its content, and create a summary.

Key points to analyze:
1. File contains multiple lines of text
2. Has both heading and content sections
3. Includes numbered lists
4. Contains various types of content

The agent should demonstrate:
- File reading capabilities
- Text processing and analysis
- File writing for output
- Basic content understanding

This test validates the core file manipulation capabilities of the Toka system.
EOF

echo "✅ Test input file created (data/test_input.txt)"

echo ""
echo "Step 8: Health Check"
echo "-------------------"

echo "Checking if the system builds and basic tests pass..."
if cargo test --lib --bin toka-orchestration-service 2>/dev/null; then
    echo "✅ Basic tests passing"
else
    echo "⚠️  Some tests may have issues, but system should still work"
fi

echo ""
echo "🎉 Setup Complete!"
echo "=================="
echo ""
echo "Your Toka testing environment is ready!"
echo ""
echo "Next steps:"
echo "1. Start the interactive Toka CLI:"
echo "   ./toka_interactive.sh"
echo ""
echo "2. Or use the CLI directly:"
echo "   ./target/release/toka --help"
echo ""
echo "3. Generate a token:"
echo "   ./target/release/toka generate-token"
echo ""
echo "4. Run in daemon mode:"
echo "   ./target/release/toka daemon"
echo ""
echo "Configuration files created:"
echo "- .env (environment variables)"
echo "- config/testing/agents.toml (agent configurations)"
echo "- data/test_input.txt (test input file)"
echo ""
echo "Happy testing! 🚀" 