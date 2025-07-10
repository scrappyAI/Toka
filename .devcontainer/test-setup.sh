#!/bin/bash

# Dev Container Setup Test Script
# This script validates that the dev container is properly configured

set -e

echo "🧪 Testing Toka Dev Container Setup..."
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}✅${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ️${NC} $1"
}

# Check if we're running inside a dev container
check_container_environment() {
    print_info "Checking execution environment..."
    
    # Check for dev container indicators
    if [[ "$TOKA_CONTAINER_MODE" == "true" ]] || [[ -n "$DEVCONTAINER" ]] || [[ "$REMOTE_CONTAINERS" == "true" ]]; then
        print_status "Running inside dev container"
        return 0
    fi
    
    # Check for container-like environment
    if [[ -f "/.dockerenv" ]] || [[ -n "$container" ]]; then
        print_warning "Running in container environment (not confirmed as dev container)"
        return 0
    fi
    
    # Check common dev container patterns
    if [[ "$USER" == "vscode" ]] || [[ "$LOGNAME" == "vscode" ]]; then
        print_status "Running as vscode user (likely in dev container)"
        return 0
    fi
    
    # Seems like we're on host machine
    print_error "This script is designed to run INSIDE the dev container!"
    echo ""
    echo "🚨 You appear to be running this on your host machine."
    echo ""
    echo "📝 To test the dev container setup:"
    echo "  1. Open this project in VS Code"
    echo "  2. Press Ctrl+Shift+P (Cmd+Shift+P on Mac)"
    echo "  3. Select 'Dev Containers: Reopen in Container'"
    echo "  4. Wait for container to build and start"
    echo "  5. Run this script again inside the container:"
    echo "     bash .devcontainer/test-setup.sh"
    echo ""
    echo "🖥️  To test your host setup instead, use:"
    echo "     bash .devcontainer/test-host.sh"
    echo ""
    return 1
}

# Test function template
test_feature() {
    local feature_name="$1"
    local test_command="$2"
    local required="$3"
    
    print_info "Testing $feature_name..."
    
    if eval "$test_command" &> /dev/null; then
        print_status "$feature_name is working"
        return 0
    else
        if [[ "$required" == "required" ]]; then
            print_error "$feature_name is required but not working"
            return 1
        else
            print_warning "$feature_name is optional and not working"
            return 0
        fi
    fi
}

# Test results tracking
total_tests=0
passed_tests=0
failed_tests=0
warnings=0

run_test() {
    local name="$1"
    local command="$2"
    local required="${3:-optional}"
    
    total_tests=$((total_tests + 1))
    
    if test_feature "$name" "$command" "$required"; then
        passed_tests=$((passed_tests + 1))
    else
        if [[ "$required" == "required" ]]; then
            failed_tests=$((failed_tests + 1))
        else
            warnings=$((warnings + 1))
        fi
    fi
}

# Main execution starts here
echo ""

# First, check if we're in the right environment
if ! check_container_environment; then
    exit 1
fi

echo ""
echo "🔧 Testing Core Development Tools"
echo "---------------------------------"

run_test "Rust toolchain" "rustc --version" "required"
run_test "Cargo" "cargo --version" "required"
run_test "rustfmt" "rustfmt --version" "required"
run_test "clippy" "clippy-driver --version" "required"

echo ""
echo "🔐 Testing GitHub Integration"
echo "-----------------------------"

run_test "GitHub CLI" "gh --version" "required"
run_test "GitHub authentication" "gh auth status" "optional"
run_test "Git configuration" "git config --global user.name" "optional"

echo ""
echo "🐍 Testing Python Integration"
echo "-----------------------------"

run_test "Python 3" "python3 --version" "required"
run_test "pip" "python3 -m pip --version" "required"

echo ""
echo "🛠️ Testing Development Utilities"
echo "--------------------------------"

run_test "exa (ls replacement)" "exa --version" "optional"
run_test "bat (cat replacement)" "bat --version" "optional"
run_test "ripgrep (grep replacement)" "rg --version" "optional"
run_test "fd (find replacement)" "fd --version" "optional"
run_test "jq (JSON processor)" "jq --version" "optional"

echo ""
echo "📦 Testing Workspace"
echo "--------------------"

run_test "Workspace Cargo.toml" "test -f Cargo.toml" "required"
run_test "Cargo workspace metadata" "cargo metadata --no-deps --format-version 1" "required"
run_test "Target directory" "test -d /tmp/target" "required"
run_test "Data directory" "test -d data" "optional"

echo ""
echo "🏃 Testing Quick Commands"
echo "-------------------------"

# Test if our aliases and functions work
run_test "cargo check (alias: cc)" "type cc" "optional"
run_test "cargo build (alias: cb)" "type cb" "optional"
run_test "cargo test (alias: ct)" "type ct" "optional"
run_test "toka-quick-test function" "type toka-quick-test" "optional"

echo ""
echo "🌍 Testing Environment Configuration"
echo "------------------------------------"

run_test "Container mode flag" "test \"\$TOKA_CONTAINER_MODE\" = \"true\"" "optional"
run_test "Rust backtrace enabled" "test \"\$RUST_BACKTRACE\" = \"1\"" "optional"
run_test "Cargo target directory" "test \"\$CARGO_TARGET_DIR\" = \"/tmp/target\"" "required"

# GitHub token test (but don't fail if not present)
if [[ -n "$GITHUB_TOKEN" ]]; then
    run_test "GitHub token environment" "test -n \"\$GITHUB_TOKEN\"" "optional"
else
    print_info "GitHub token not set (will use device flow authentication)"
fi

# LLM configuration test
if [[ -n "$ANTHROPIC_API_KEY" ]] || [[ -n "$OPENAI_API_KEY" ]]; then
    run_test "LLM API key present" "test -n \"\$ANTHROPIC_API_KEY\" -o -n \"\$OPENAI_API_KEY\"" "optional"
    run_test "LLM provider configured" "test -n \"\$LLM_PROVIDER\"" "optional"
else
    print_info "LLM API keys not set (AI features will be limited)"
fi

echo ""
echo "📋 Test Results Summary"
echo "======================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $failed_tests"
echo "Warnings: $warnings"

echo ""
if [[ $failed_tests -eq 0 ]]; then
    print_status "All required tests passed! 🎉"
    echo ""
    echo "🚀 Ready for Development!"
    echo "========================"
    echo ""
    echo "Quick start commands:"
    echo "  toka-quick-test     # Run workspace validation"
    echo "  build-all           # Build entire workspace"
    echo "  test-all           # Run all tests"
    echo "  gh repo view       # View repository info"
    echo ""
    echo "Next steps:"
    echo "1. Run 'toka-quick-test' to validate the workspace"
    echo "2. Try 'cargo build --workspace' to build everything"
    echo "3. Set up GitHub authentication if not done: 'gh auth login'"
    echo ""
    exit 0
else
    print_error "Some required tests failed"
    echo ""
    echo "🔧 Troubleshooting Tips:"
    echo "========================"
    echo ""
    echo "1. Check if Docker container built correctly"
    echo "2. Verify all setup scripts ran successfully"
    echo "3. Try rebuilding the container:"
    echo "   Ctrl+Shift+P → 'Dev Containers: Rebuild Container'"
    echo ""
    echo "4. For GitHub issues:"
    echo "   - Check your GitHub token permissions"
    echo "   - Try: gh auth login --web --scopes \"repo,read:org,read:user,gist\""
    echo ""
    echo "5. For build issues:"
    echo "   - Try: cargo clean && cargo build --workspace"
    echo ""
    exit 1
fi 