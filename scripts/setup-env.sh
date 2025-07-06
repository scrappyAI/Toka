#!/bin/bash

# Toka Environment Setup Script
# Interactive script to safely configure environment variables

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$WORKSPACE_ROOT/.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
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

# Function to show welcome message
show_welcome() {
    cat << EOF
${BLUE}
╔══════════════════════════════════════════════════════════════════════════════╗
║                        Toka OS Environment Setup                            ║
║                                                                              ║
║  This script will help you safely configure environment variables for       ║
║  LLM integration with your Toka system.                                     ║
║                                                                              ║
║  Your API keys will be stored in .env file (already in .gitignore)         ║
╚══════════════════════════════════════════════════════════════════════════════╝
${NC}

EOF
}

# Function to prompt for LLM provider
choose_llm_provider() {
    log_info "Choose your LLM provider:"
    echo "1) Anthropic Claude (Recommended)"
    echo "2) OpenAI GPT-4"
    echo "3) Both (for testing)"
    echo -n "Enter your choice (1-3): "
    
    local choice
    read -r choice
    
    case $choice in
        1)
            echo "anthropic"
            ;;
        2)
            echo "openai"
            ;;
        3)
            echo "both"
            ;;
        *)
            log_error "Invalid choice. Please run the script again."
            exit 1
            ;;
    esac
}

# Function to prompt for API key securely
prompt_api_key() {
    local provider="$1"
    local key_var="$2"
    
    log_info "Enter your $provider API key:"
    echo -n "API Key: "
    
    # Read API key without echoing to terminal
    local api_key
    read -s api_key
    echo  # Add newline after hidden input
    
    # Validate key is not empty
    if [ -z "$api_key" ]; then
        log_error "API key cannot be empty"
        return 1
    fi
    
    # Basic validation for key format
    if [ "$provider" = "Anthropic" ]; then
        if [[ ! "$api_key" =~ ^sk-ant- ]]; then
            log_warning "API key doesn't match expected Anthropic format (sk-ant-...)"
            echo -n "Continue anyway? (y/n): "
            local continue_choice
            read -r continue_choice
            if [ "$continue_choice" != "y" ]; then
                return 1
            fi
        fi
    elif [ "$provider" = "OpenAI" ]; then
        if [[ ! "$api_key" =~ ^sk- ]]; then
            log_warning "API key doesn't match expected OpenAI format (sk-...)"
            echo -n "Continue anyway? (y/n): "
            local continue_choice
            read -r continue_choice
            if [ "$continue_choice" != "y" ]; then
                return 1
            fi
        fi
    fi
    
    echo "$api_key"
}

# Function to create .env file
create_env_file() {
    local provider="$1"
    local anthropic_key="$2"
    local openai_key="$3"
    
    log_info "Creating .env file..."
    
    # Create .env file with proper permissions
    cat > "$ENV_FILE" << EOF
# Toka OS Environment Variables
# Generated on $(date)

# LLM Provider Configuration
LLM_PROVIDER=$provider

EOF
    
    # Add API keys
    if [ -n "$anthropic_key" ]; then
        echo "ANTHROPIC_API_KEY=$anthropic_key" >> "$ENV_FILE"
    fi
    
    if [ -n "$openai_key" ]; then
        echo "OPENAI_API_KEY=$openai_key" >> "$ENV_FILE"
    fi
    
    # Add optional configuration
    cat >> "$ENV_FILE" << EOF

# Optional Configuration
LLM_MODEL=claude-3-5-sonnet-20241022
LLM_RATE_LIMIT=50
LLM_TIMEOUT=30
LLM_DEBUG=false

# Additional Settings
RUST_LOG=info
RUST_BACKTRACE=1
EOF
    
    # Set secure permissions
    chmod 600 "$ENV_FILE"
    
    log_success ".env file created successfully"
    log_info "File location: $ENV_FILE"
    log_info "Permissions: $(ls -la "$ENV_FILE" | awk '{print $1}')"
}

# Function to test environment
test_environment() {
    log_info "Testing environment configuration..."
    
    # Source the .env file
    if [ -f "$ENV_FILE" ]; then
        source "$ENV_FILE"
        log_success "Environment variables loaded from .env file"
    else
        log_error ".env file not found"
        return 1
    fi
    
    # Check if test script exists
    if [ -f "$SCRIPT_DIR/test-toka-system.sh" ]; then
        log_info "Running environment validation..."
        if "$SCRIPT_DIR/test-toka-system.sh" --env-only; then
            log_success "Environment validation passed!"
        else
            log_error "Environment validation failed"
            return 1
        fi
    else
        log_warning "Test script not found, skipping validation"
    fi
    
    return 0
}

# Function to show next steps
show_next_steps() {
    cat << EOF

${GREEN}🎉 Environment Setup Complete!${NC}

${BLUE}Next Steps:${NC}

1. ${YELLOW}Test your setup:${NC}
   ./scripts/test-toka-system.sh

2. ${YELLOW}Run the full system test:${NC}
   ./scripts/test-toka-system.sh --all

3. ${YELLOW}Test just the LLM integration:${NC}
   ./scripts/test-toka-system.sh --llm-only

4. ${YELLOW}Build and test the system:${NC}
   cargo build --workspace
   cargo test --workspace

5. ${YELLOW}Try the orchestration example:${NC}
   cargo run --example parallel_orchestration

${BLUE}Important Notes:${NC}
- Your API keys are stored in .env file (secure permissions set)
- The .env file is already in .gitignore (won't be committed)
- You can edit .env file manually if needed
- For production, consider using environment variables directly

${BLUE}Documentation:${NC}
- See TOKA_TESTING_SETUP_GUIDE.md for complete testing guide
- Check target/toka-system-report.md after running tests

${GREEN}Happy coding with Toka! 🚀${NC}

EOF
}

# Function to check for existing .env file
check_existing_env() {
    if [ -f "$ENV_FILE" ]; then
        log_warning "Existing .env file found!"
        echo -n "Do you want to overwrite it? (y/n): "
        local overwrite
        read -r overwrite
        if [ "$overwrite" != "y" ]; then
            log_info "Keeping existing .env file"
            log_info "You can edit it manually at: $ENV_FILE"
            return 1
        fi
    fi
    return 0
}

# Main function
main() {
    cd "$WORKSPACE_ROOT"
    
    show_welcome
    
    # Check for existing .env file
    if ! check_existing_env; then
        exit 0
    fi
    
    # Choose LLM provider
    local provider_choice
    provider_choice=$(choose_llm_provider)
    
    local anthropic_key=""
    local openai_key=""
    local provider=""
    
    # Get API keys based on choice
    case "$provider_choice" in
        "anthropic")
            provider="anthropic"
            anthropic_key=$(prompt_api_key "Anthropic" "ANTHROPIC_API_KEY")
            ;;
        "openai")
            provider="openai"
            openai_key=$(prompt_api_key "OpenAI" "OPENAI_API_KEY")
            ;;
        "both")
            provider="anthropic"  # Default to Anthropic
            anthropic_key=$(prompt_api_key "Anthropic" "ANTHROPIC_API_KEY")
            openai_key=$(prompt_api_key "OpenAI" "OPENAI_API_KEY")
            ;;
    esac
    
    # Create .env file
    create_env_file "$provider" "$anthropic_key" "$openai_key"
    
    # Test environment
    if test_environment; then
        show_next_steps
    else
        log_error "Environment setup completed but validation failed"
        log_info "Check the .env file manually: $ENV_FILE"
        exit 1
    fi
}

# Run main function
main "$@"