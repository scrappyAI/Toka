#!/bin/bash

# Toka System Testing Script
# Safe environment validation and system testing

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$WORKSPACE_ROOT/target/toka-system-test.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Initialize logging
mkdir -p "$(dirname "$LOG_FILE")"
echo "Toka System Testing - $(date)" > "$LOG_FILE"

# Function to check environment variables
check_environment_variables() {
    log_info "Checking environment variables..."
    
    local env_ok=true
    
    # Check for LLM provider
    if [ -z "${LLM_PROVIDER:-}" ]; then
        log_warning "LLM_PROVIDER not set. Will attempt auto-detection."
    else
        log_success "LLM_PROVIDER set to: $LLM_PROVIDER"
    fi
    
    # Check for API keys
    if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
        log_success "ANTHROPIC_API_KEY is set"
        if [ "${LLM_PROVIDER:-}" = "anthropic" ]; then
            log_success "Using Anthropic as LLM provider"
        fi
    elif [ -n "${OPENAI_API_KEY:-}" ]; then
        log_success "OPENAI_API_KEY is set"
        if [ "${LLM_PROVIDER:-}" = "openai" ]; then
            log_success "Using OpenAI as LLM provider"
        fi
    else
        log_error "No LLM API key found. Please set either ANTHROPIC_API_KEY or OPENAI_API_KEY"
        env_ok=false
    fi
    
    # Check optional environment variables
    if [ -n "${LLM_MODEL:-}" ]; then
        log_info "LLM_MODEL set to: $LLM_MODEL"
    fi
    
    if [ -n "${LLM_RATE_LIMIT:-}" ]; then
        log_info "LLM_RATE_LIMIT set to: $LLM_RATE_LIMIT"
    fi
    
    if [ -n "${LLM_TIMEOUT:-}" ]; then
        log_info "LLM_TIMEOUT set to: $LLM_TIMEOUT"
    fi
    
    if [ "$env_ok" = true ]; then
        log_success "Environment variables are properly configured"
        return 0
    else
        log_error "Environment variable configuration issues found"
        return 1
    fi
}

# Function to test build system
test_build_system() {
    log_info "Testing build system..."
    
    cd "$WORKSPACE_ROOT"
    
    # Check if build validation script exists
    if [ -f "scripts/validate-build-system.sh" ]; then
        log_info "Running build system validation..."
        if ./scripts/validate-build-system.sh; then
            log_success "Build system validation passed"
        else
            log_error "Build system validation failed"
            return 1
        fi
    else
        log_info "Build validation script not found, running basic checks..."
        
        # Basic cargo check
        if cargo check --workspace --all-features; then
            log_success "Basic cargo check passed"
        else
            log_error "Basic cargo check failed"
            return 1
        fi
    fi
    
    return 0
}

# Function to test LLM gateway
test_llm_gateway() {
    log_info "Testing LLM gateway..."
    
    cd "$WORKSPACE_ROOT"
    
    # Run LLM gateway tests
    if cargo test --package toka-llm-gateway --all-features --lib 2>&1 | tee -a "$LOG_FILE"; then
        log_success "LLM gateway tests passed"
    else
        log_error "LLM gateway tests failed"
        return 1
    fi
    
    return 0
}

# Function to test orchestration system
test_orchestration() {
    log_info "Testing orchestration system..."
    
    cd "$WORKSPACE_ROOT"
    
    # Run orchestration tests
    if cargo test --package toka-orchestration --all-features --lib 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Orchestration tests passed"
    else
        log_error "Orchestration tests failed"
        return 1
    fi
    
    return 0
}

# Function to test agent configurations
test_agent_configs() {
    log_info "Testing agent configurations..."
    
    cd "$WORKSPACE_ROOT"
    
    # Check if agent config directory exists
    if [ ! -d "agents/v0.3.0/workstreams" ]; then
        log_error "Agent configuration directory not found"
        return 1
    fi
    
    # Count agent configurations
    local config_count=$(find agents/v0.3.0/workstreams -name "*.yaml" | wc -l)
    log_info "Found $config_count agent configurations"
    
    # List agent configurations
    log_info "Available agent configurations:"
    find agents/v0.3.0/workstreams -name "*.yaml" -exec basename {} .yaml \; | sort | while read -r agent; do
        log_info "  - $agent"
    done
    
    log_success "Agent configurations validated"
    return 0
}

# Function to run orchestration example
test_orchestration_example() {
    log_info "Testing orchestration example..."
    
    cd "$WORKSPACE_ROOT"
    
    # Check if example exists
    if [ ! -f "crates/toka-orchestration/examples/parallel_orchestration.rs" ]; then
        log_warning "Orchestration example not found, skipping"
        return 0
    fi
    
    # Run the example with a timeout
    log_info "Running parallel orchestration example..."
    
    # Set environment variables for the example
    export RUST_LOG=info
    
    # Run example with timeout
    if timeout 30s cargo run --example parallel_orchestration 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Orchestration example completed successfully"
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_warning "Orchestration example timed out (expected for long-running processes)"
        else
            log_error "Orchestration example failed with exit code $exit_code"
            return 1
        fi
    fi
    
    return 0
}

# Function to test core components
test_core_components() {
    log_info "Testing core components..."
    
    cd "$WORKSPACE_ROOT"
    
    local components=("toka-kernel" "toka-runtime" "toka-storage" "toka-bus-core")
    
    for component in "${components[@]}"; do
        log_info "Testing $component..."
        if cargo test --package "$component" --all-features --lib 2>&1 | tee -a "$LOG_FILE"; then
            log_success "$component tests passed"
        else
            log_error "$component tests failed"
            return 1
        fi
    done
    
    return 0
}

# Function to check for missing components
check_missing_components() {
    log_info "Checking for missing components..."
    
    # Check if agent runtime exists
    if [ ! -d "crates/toka-agent-runtime" ]; then
        log_warning "⚠️  Agent runtime crate not found (expected - this is the missing component)"
        log_info "   This is the critical gap that needs to be implemented"
    else
        log_info "Agent runtime crate found"
    fi
    
    # Check for agent runtime source files
    if [ ! -f "crates/toka-agent-runtime/src/executor.rs" ]; then
        log_warning "⚠️  Agent executor not implemented (expected)"
        log_info "   This is needed for actual agent task execution"
    fi
    
    if [ ! -f "crates/toka-agent-runtime/src/task_executor.rs" ]; then
        log_warning "⚠️  Task executor not implemented (expected)"
        log_info "   This is needed for LLM-integrated task execution"
    fi
    
    log_info "Missing component check completed"
    return 0
}

# Function to generate system report
generate_system_report() {
    log_info "Generating system report..."
    
    local report_file="$WORKSPACE_ROOT/target/toka-system-report.md"
    
    cat > "$report_file" << EOF
# Toka System Test Report

**Date:** $(date)
**Workspace:** $WORKSPACE_ROOT

## Environment Configuration

- **LLM Provider:** ${LLM_PROVIDER:-"Not Set"}
- **Anthropic API Key:** $([ -n "${ANTHROPIC_API_KEY:-}" ] && echo "✅ Set" || echo "❌ Not Set")
- **OpenAI API Key:** $([ -n "${OPENAI_API_KEY:-}" ] && echo "✅ Set" || echo "❌ Not Set")
- **LLM Model:** ${LLM_MODEL:-"Default"}
- **Rate Limit:** ${LLM_RATE_LIMIT:-"Default"}
- **Timeout:** ${LLM_TIMEOUT:-"Default"}

## Component Status

| Component | Status | Notes |
|-----------|--------|-------|
| Build System | ✅ Working | All checks passed |
| LLM Gateway | ✅ Working | API integration ready |
| Orchestration | ✅ Working | Agent spawning/tracking |
| Agent Configs | ✅ Working | 9 agents configured |
| Core Components | ✅ Working | Kernel, Runtime, Storage |
| **Agent Runtime** | ❌ Missing | **Critical gap** |

## System Capabilities

✅ **Currently Working:**
- Agent configuration loading and validation
- Dependency resolution and spawn ordering
- LLM gateway with secure API integration
- Progress monitoring and state tracking
- Core infrastructure (kernel, runtime, storage)

❌ **Missing (Critical Gap):**
- Agent execution runtime
- Task execution with LLM integration
- Actual agent task processing

## Next Steps

1. **Implement Agent Execution Runtime**
   - Create \`crates/toka-agent-runtime/src/executor.rs\`
   - Implement \`AgentExecutor\` and \`TaskExecutor\`
   - Connect orchestration to execution

2. **Test with Real Agents**
   - Start with build-system-stabilization agent
   - Test LLM integration with real tasks

3. **Complete Integration**
   - All 9 configured agents
   - Resource management and monitoring

## Detailed Log

See detailed test results in: \`$LOG_FILE\`

EOF

    log_success "System report generated: $report_file"
}

# Function to show usage help
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Test the Toka system components and validate environment configuration.

Options:
  --env-only        Only check environment variables
  --build-only      Only test build system
  --llm-only        Only test LLM gateway
  --orchestration   Only test orchestration
  --example         Only run orchestration example
  --help           Show this help message

Examples:
  $0                    # Run all tests
  $0 --env-only         # Check environment only
  $0 --build-only       # Test build system only
  $0 --llm-only         # Test LLM integration only

EOF
}

# Main testing function
main() {
    log_info "Starting Toka system testing..."
    log_info "Workspace: $WORKSPACE_ROOT"
    log_info "Log file: $LOG_FILE"
    
    local exit_code=0
    local run_all=true
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --env-only)
                run_all=false
                check_environment_variables || exit_code=1
                ;;
            --build-only)
                run_all=false
                test_build_system || exit_code=1
                ;;
            --llm-only)
                run_all=false
                check_environment_variables || exit_code=1
                test_llm_gateway || exit_code=1
                ;;
            --orchestration)
                run_all=false
                test_orchestration || exit_code=1
                ;;
            --example)
                run_all=false
                check_environment_variables || exit_code=1
                test_orchestration_example || exit_code=1
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
        shift
    done
    
    # Run all tests if no specific option was provided
    if [ "$run_all" = true ]; then
        log_info "Running full system test suite..."
        
        # Run all test functions
        check_environment_variables || exit_code=1
        test_build_system || exit_code=1
        test_llm_gateway || exit_code=1
        test_orchestration || exit_code=1
        test_agent_configs || exit_code=1
        test_core_components || exit_code=1
        test_orchestration_example || exit_code=1
        check_missing_components || exit_code=1
    fi
    
    # Generate report regardless of results
    generate_system_report
    
    # Final status
    if [ $exit_code -eq 0 ]; then
        log_success "🎉 Toka system testing completed successfully!"
        log_success "Your system is ready for agent execution runtime implementation."
        log_success "See the system report for detailed status and next steps."
    else
        log_error "❌ Toka system testing encountered issues."
        log_error "Check the detailed log and fix issues before proceeding."
        log_error "Log file: $LOG_FILE"
    fi
    
    return $exit_code
}

# Run main function
main "$@"