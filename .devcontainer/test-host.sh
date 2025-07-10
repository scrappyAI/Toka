#!/bin/bash

# Host Environment Test Script
# This script validates your local development environment

set -e

echo "🖥️  Testing Host Development Environment..."
echo "==========================================="

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

# Test function template
test_feature() {
    local feature_name="$1"
    local test_command="$2"
    local required="$3"
    
    print_info "Testing $feature_name..."
    
    if eval "$test_command" &> /dev/null; then
        print_status "$feature_name is available"
        return 0
    else
        if [[ "$required" == "required" ]]; then
            print_error "$feature_name is required but not available"
            return 1
        else
            print_warning "$feature_name is recommended but not available"
            return 0
        fi
    fi
}

# Test for dev container compatible IDE
test_ide_support() {
    print_info "Testing IDE support..."
    
    local has_vscode=false
    local has_cursor=false
    
    # Check for VS Code
    if command -v code &> /dev/null; then
        has_vscode=true
        local vscode_version=$(code --version 2>/dev/null | head -n1)
        print_status "VS Code is available (${vscode_version})"
    fi
    
    # Check for Cursor
    if command -v cursor &> /dev/null; then
        has_cursor=true
        local cursor_version=$(cursor --version 2>/dev/null | head -n1)
        print_status "Cursor is available (${cursor_version})"
    fi
    
    # Check if either is available
    if [[ "$has_vscode" == true ]] || [[ "$has_cursor" == true ]]; then
        print_status "Dev container compatible IDE found"
        return 0
    else
        print_error "No dev container compatible IDE found"
        return 1
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
    local required="${3:-recommended}"
    
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

run_ide_test() {
    total_tests=$((total_tests + 1))
    
    if test_ide_support; then
        passed_tests=$((passed_tests + 1))
    else
        failed_tests=$((failed_tests + 1))
    fi
}

echo ""
echo "📋 Checking Prerequisites for Dev Container"
echo "-------------------------------------------"

run_test "Docker" "docker --version" "required"
run_test "Docker Compose" "docker-compose --version" "recommended"
run_ide_test

echo ""
echo "🦀 Checking Rust Development Tools (Host)"
echo "-----------------------------------------"

run_test "Rust toolchain" "rustc --version" "recommended"
run_test "Cargo" "cargo --version" "recommended"

echo ""
echo "🔐 Checking GitHub Tools (Host)"
echo "-------------------------------"

run_test "Git" "git --version" "required"
run_test "GitHub CLI" "gh --version" "recommended"

echo ""
echo "📦 Checking Project Structure"
echo "-----------------------------"

run_test "Workspace Cargo.toml" "test -f Cargo.toml" "required"
run_test "Dev container config" "test -f .devcontainer/devcontainer.json" "required"
run_test "Dev container Dockerfile" "test -f .devcontainer/Dockerfile" "required"
run_test "Post-create script" "test -f .devcontainer/post-create.sh" "required"

echo ""
echo "🌐 Checking Environment Setup"
echo "-----------------------------"

# Check for environment files
if [[ -f ".env.local" ]]; then
    run_test "Local environment file" "test -f .env.local" "optional"
    print_info "Environment configuration found"
else
    print_warning "No .env.local file found"
    print_info "Consider copying .devcontainer/env.local.template to .env.local"
fi

# Check for GitHub token in environment
if [[ -n "$GITHUB_TOKEN" ]]; then
    run_test "GitHub token in environment" "test -n \"\$GITHUB_TOKEN\"" "optional"
else
    print_info "No GITHUB_TOKEN found (will use device flow in container)"
fi

echo ""
echo "🔧 Testing Docker Setup"
echo "-----------------------"

if command -v docker &> /dev/null; then
    if docker info &> /dev/null; then
        print_status "Docker daemon is running"
    else
        print_error "Docker is installed but daemon is not running"
        print_info "Please start Docker Desktop"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test basic Docker functionality
    if docker run --rm hello-world &> /dev/null; then
        print_status "Docker can run containers"
    else
        print_warning "Docker container execution test failed"
        warnings=$((warnings + 1))
    fi
else
    print_error "Docker is not installed"
    failed_tests=$((failed_tests + 1))
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
    print_status "Host environment is ready for dev containers! 🎉"
    echo ""
    echo "🚀 Next Steps:"
    echo "============="
    echo ""
    echo "1. **Setup GitHub Authentication (recommended):**"
    echo "   - Copy template: cp .devcontainer/env.local.template .env.local"
    echo "   - Get GitHub token: https://github.com/settings/tokens"
    echo "   - Edit .env.local with your credentials"
    echo ""
    echo "2. **Start the Dev Container:**"
    
    # Provide IDE-specific instructions
    if command -v cursor &> /dev/null; then
        echo "   **Using Cursor:**"
        echo "   - Open project: cursor ."
        echo "   - Press Ctrl+Shift+P (Cmd+Shift+P on Mac)"
        echo "   - Select 'Dev Containers: Reopen in Container'"
    fi
    
    if command -v code &> /dev/null; then
        echo "   **Using VS Code:**"
        echo "   - Open project: code ."
        echo "   - Press Ctrl+Shift+P (Cmd+Shift+P on Mac)" 
        echo "   - Select 'Dev Containers: Reopen in Container'"
    fi
    
    echo "   - Wait for container to build and start"
    echo ""
    echo "3. **Test the Container:**"
    echo "   - Once inside container, run: bash .devcontainer/test-setup.sh"
    echo ""
    echo "📖 For detailed instructions, see: .devcontainer/README.md"
    echo ""
    exit 0
else
    print_error "Host environment has issues that need to be resolved"
    echo ""
    echo "🔧 Required Fixes:"
    echo "=================="
    echo ""
    
    if ! command -v docker &> /dev/null; then
        echo "❌ **Install Docker Desktop:**"
        echo "   - macOS: https://docs.docker.com/desktop/install/mac-install/"
        echo "   - Windows: https://docs.docker.com/desktop/install/windows-install/"
        echo "   - Linux: https://docs.docker.com/desktop/install/linux-install/"
        echo ""
    fi
    
    if ! command -v code &> /dev/null && ! command -v cursor &> /dev/null; then
        echo "❌ **Install a Dev Container Compatible IDE:**"
        echo ""
        echo "   **Option 1: Cursor (Recommended for AI development)**"
        echo "   - Download: https://cursor.sh/"
        echo "   - Supports dev containers natively"
        echo "   - Built-in AI assistance"
        echo ""
        echo "   **Option 2: VS Code**"
        echo "   - Download: https://code.visualstudio.com/"
        echo "   - Install the Dev Containers extension"
        echo ""
    fi
    
    if ! command -v git &> /dev/null; then
        echo "❌ **Install Git:**"
        echo "   - macOS: brew install git"
        echo "   - Windows: https://git-scm.com/download/win"
        echo "   - Linux: sudo apt install git"
        echo ""
    fi
    
    if [[ ! -f "Cargo.toml" ]]; then
        echo "❌ **Wrong Directory:**"
        echo "   - Make sure you're in the project root directory"
        echo "   - Look for Cargo.toml in the current directory"
        echo ""
    fi
    
    echo "💡 **Recommendations:**"
    echo "====================="
    echo ""
    
    if ! command -v gh &> /dev/null; then
        echo "⚠️  **Install GitHub CLI (recommended):**"
        echo "   - macOS: brew install gh"
        echo "   - Windows: winget install GitHub.cli"
        echo "   - Linux: See https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
        echo ""
    fi
    
    if ! command -v rustc &> /dev/null; then
        echo "⚠️  **Install Rust (optional for host):**"
        echo "   - The dev container includes Rust, but you might want it locally too"
        echo "   - Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
    fi
    
    echo "After fixing the required issues, run this script again:"
    echo "bash .devcontainer/test-host.sh"
    echo ""
    exit 1
fi 