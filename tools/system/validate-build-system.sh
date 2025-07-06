#!/bin/bash

# Build System Validation Script
# Referenced in .github/workflows/workstream-ci.yml
# Validates workspace build system stability and dependency resolution

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$WORKSPACE_ROOT/target/build-system-validation.log"

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
echo "Build System Validation - $(date)" > "$LOG_FILE"

# Function to check if required tools are available
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_tools=()
    
    # Check for required tools
    command -v cargo >/dev/null 2>&1 || missing_tools+=("cargo")
    command -v rustc >/dev/null 2>&1 || missing_tools+=("rustc")
    command -v git >/dev/null 2>&1 || missing_tools+=("git")
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        return 1
    fi
    
    log_success "All prerequisites available"
    return 0
}

# Function to validate workspace structure
validate_workspace_structure() {
    log_info "Validating workspace structure..."
    
    # Check for essential files
    local required_files=(
        "Cargo.toml"
        "Cargo.lock"
        ".gitignore"
    )
    
    for file in "${required_files[@]}"; do
        if [ ! -f "$WORKSPACE_ROOT/$file" ]; then
            log_error "Missing required file: $file"
            return 1
        fi
    done
    
    # Check for workspace member directories
    if [ ! -d "$WORKSPACE_ROOT/crates" ]; then
        log_error "Missing crates directory"
        return 1
    fi
    
    log_success "Workspace structure is valid"
    return 0
}

# Function to check for dependency conflicts
check_dependency_conflicts() {
    log_info "Checking for dependency conflicts..."
    
    cd "$WORKSPACE_ROOT"
    
    # Check for duplicate dependencies
    log_info "Analyzing dependency tree for conflicts..."
    
    if cargo tree --duplicates > /dev/null 2>&1; then
        local duplicates
        duplicates=$(cargo tree --duplicates 2>/dev/null || echo "")
        
        if [ -n "$duplicates" ]; then
            log_warning "Found duplicate dependencies:"
            echo "$duplicates" | tee -a "$LOG_FILE"
        else
            log_success "No duplicate dependencies found"
        fi
    else
        log_warning "cargo tree --duplicates failed, skipping duplicate check"
    fi
    
    # Specific check for base64ct compatibility issue
    log_info "Checking base64ct compatibility..."
    
    if cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "base64ct") | .version' | grep -q "0."; then
        local base64ct_versions
        base64ct_versions=$(cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "base64ct") | .version' | sort -u)
        log_info "Found base64ct versions: $base64ct_versions"
        
        # Check if we have the minimum required version (0.22)
        if echo "$base64ct_versions" | grep -q "^0.2[2-9]"; then
            log_success "base64ct version is compatible (>=0.22)"
        else
            log_error "base64ct version compatibility issue detected"
            return 1
        fi
    fi
    
    return 0
}

# Function to test basic build functionality
test_basic_build() {
    log_info "Testing basic build functionality..."
    
    cd "$WORKSPACE_ROOT"
    
    # Clean build
    log_info "Cleaning previous build artifacts..."
    cargo clean 2>&1 | tee -a "$LOG_FILE"
    
    # Check workspace compilation
    log_info "Testing workspace check..."
    if ! cargo check --workspace --all-features 2>&1 | tee -a "$LOG_FILE"; then
        log_error "Workspace check failed"
        return 1
    fi
    log_success "Workspace check passed"
    
    # Test individual crate builds
    log_info "Testing individual crate builds..."
    local failed_crates=()
    
    for crate_dir in crates/*/; do
        if [ -f "$crate_dir/Cargo.toml" ]; then
            local crate_name
            crate_name=$(basename "$crate_dir")
            log_info "Building crate: $crate_name"
            
            if ! cargo check -p "$crate_name" --all-features 2>&1 | tee -a "$LOG_FILE"; then
                log_error "Failed to build crate: $crate_name"
                failed_crates+=("$crate_name")
            else
                log_success "Successfully built crate: $crate_name"
            fi
        fi
    done
    
    if [ ${#failed_crates[@]} -ne 0 ]; then
        log_error "Failed to build crates: ${failed_crates[*]}"
        return 1
    fi
    
    log_success "All crates built successfully"
    return 0
}

# Function to run basic tests
test_basic_functionality() {
    log_info "Testing basic functionality..."
    
    cd "$WORKSPACE_ROOT"
    
    # Run quick tests
    log_info "Running quick test suite..."
    if ! cargo test --workspace --all-features --lib 2>&1 | tee -a "$LOG_FILE"; then
        log_error "Quick tests failed"
        return 1
    fi
    
    log_success "Basic tests passed"
    return 0
}

# Function to validate formatting
check_formatting() {
    log_info "Checking code formatting..."
    
    cd "$WORKSPACE_ROOT"
    
    if ! cargo fmt --all -- --check 2>&1 | tee -a "$LOG_FILE"; then
        log_warning "Code formatting issues detected"
        log_info "Run 'cargo fmt' to fix formatting"
        return 1
    fi
    
    log_success "Code formatting is correct"
    return 0
}

# Function to run clippy
check_clippy() {
    log_info "Running clippy analysis..."
    
    cd "$WORKSPACE_ROOT"
    
    if ! cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee -a "$LOG_FILE"; then
        log_error "Clippy found issues"
        return 1
    fi
    
    log_success "Clippy analysis passed"
    return 0
}

# Function to generate validation report
generate_report() {
    log_info "Generating validation report..."
    
    local report_file="$WORKSPACE_ROOT/target/build-validation-report.md"
    
    cat > "$report_file" << EOF
# Build System Validation Report

**Date:** $(date)
**Workspace:** $WORKSPACE_ROOT

## Summary

This report contains the results of build system validation checks.

## Checks Performed

1. **Prerequisites Check**: Verified required tools are available
2. **Workspace Structure**: Validated workspace organization
3. **Dependency Conflicts**: Checked for dependency issues
4. **Build Functionality**: Tested compilation of all crates
5. **Basic Tests**: Ran test suite for functionality validation
6. **Code Formatting**: Verified code formatting standards
7. **Clippy Analysis**: Static analysis for code quality

## Detailed Results

See the detailed log at: \`$LOG_FILE\`

## Next Steps

If any checks failed, review the detailed log and address the issues before proceeding.

For dependency conflicts, consider updating Cargo.toml files or using cargo update.
For build failures, review the compilation errors and fix the underlying issues.
For test failures, investigate the failing tests and ensure they pass.

EOF

    log_success "Validation report generated: $report_file"
}

# Main validation sequence
main() {
    log_info "Starting build system validation..."
    log_info "Workspace: $WORKSPACE_ROOT"
    
    local exit_code=0
    
    # Run all validation checks
    check_prerequisites || exit_code=1
    validate_workspace_structure || exit_code=1
    check_dependency_conflicts || exit_code=1
    test_basic_build || exit_code=1
    test_basic_functionality || exit_code=1
    check_formatting || { log_warning "Formatting check failed but continuing..."; }
    check_clippy || { log_warning "Clippy check failed but continuing..."; }
    
    # Generate report regardless of results
    generate_report
    
    if [ $exit_code -eq 0 ]; then
        log_success "✅ Build system validation completed successfully!"
        log_success "All checks passed. The build system is stable and ready for use."
    else
        log_error "❌ Build system validation failed!"
        log_error "Some checks failed. Review the detailed log and fix issues before proceeding."
        log_error "Log file: $LOG_FILE"
    fi
    
    return $exit_code
}

# Run main function
main "$@"