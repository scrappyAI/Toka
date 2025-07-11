#!/bin/bash

# Toka Environment Validation Script
# This script validates environment configuration files for completeness and security

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_error() {
    echo -e "${RED}ERROR: $1${NC}"
}

print_success() {
    echo -e "${GREEN}SUCCESS: $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

print_info() {
    echo -e "${BLUE}INFO: $1${NC}"
}

print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

# Validation functions
validate_file_exists() {
    local file=$1
    if [[ ! -f "$file" ]]; then
        print_error "File '$file' does not exist"
        return 1
    fi
    return 0
}

validate_no_secrets_in_template() {
    local file=$1
    
    # Check for common secret patterns
    if grep -q "sk-ant-" "$file" 2>/dev/null; then
        print_error "Found Anthropic API key in template '$file'"
        return 1
    fi
    
    if grep -q "sk-" "$file" 2>/dev/null; then
        print_error "Found OpenAI API key in template '$file'"
        return 1
    fi
    
    if grep -q "ghp_" "$file" 2>/dev/null; then
        print_error "Found GitHub token in template '$file'"
        return 1
    fi
    
    if grep -q "ghs_" "$file" 2>/dev/null; then
        print_error "Found GitHub secret in template '$file'"
        return 1
    fi
    
    return 0
}

validate_required_vars() {
    local file=$1
    shift
    local required_vars=("$@")
    
    for var in "${required_vars[@]}"; do
        if ! grep -q "^${var}=" "$file" 2>/dev/null; then
            print_error "Missing required variable '$var' in '$file'"
            return 1
        fi
    done
    
    return 0
}

validate_env_file() {
    local file=$1
    local context=$2
    
    print_header "Validating $file ($context)"
    
    if ! validate_file_exists "$file"; then
        return 1
    fi
    
    local errors=0
    
    # Context-specific validation
    case "$context" in
        "main")
            local required_vars=("LLM_PROVIDER" "DATABASE_URL" "AGENT_POOL_SIZE" "JWT_SECRET")
            if ! validate_required_vars "$file" "${required_vars[@]}"; then
                ((errors++))
            fi
            ;;
        "cursor")
            local required_vars=("CURSOR_AGENT_MODE" "LLM_PROVIDER" "AGENT_POOL_SIZE")
            if ! validate_required_vars "$file" "${required_vars[@]}"; then
                ((errors++))
            fi
            ;;
        "development")
            local required_vars=("TOKA_ENV" "RUST_LOG" "DEBUG_MODE")
            if ! validate_required_vars "$file" "${required_vars[@]}"; then
                ((errors++))
            fi
            ;;
        "production")
            local required_vars=("TOKA_ENV" "RUST_LOG" "ENABLE_SECURITY_SCANNING")
            if ! validate_required_vars "$file" "${required_vars[@]}"; then
                ((errors++))
            fi
            
            # Check for production-specific security
            if grep -q "DEBUG_MODE=true" "$file" 2>/dev/null; then
                print_warning "Debug mode is enabled in production file"
                ((errors++))
            fi
            ;;
        "auth")
            local required_vars=("GITHUB_CLIENT_ID" "GITHUB_CLIENT_SECRET" "JWT_SECRET")
            if ! validate_required_vars "$file" "${required_vars[@]}"; then
                ((errors++))
            fi
            ;;
    esac
    
    # Check for template files
    if [[ "$file" == *.example ]]; then
        if ! validate_no_secrets_in_template "$file"; then
            ((errors++))
        fi
    fi
    
    # Check for weak secrets in non-template files
    if [[ "$file" != *.example ]] && [[ "$context" != "template" ]]; then
        if grep -q "your_.*_here" "$file" 2>/dev/null; then
            print_warning "Found placeholder values in '$file' - please replace with actual values"
        fi
        
        if grep -q "change-in-production" "$file" 2>/dev/null; then
            print_warning "Found 'change-in-production' placeholder in '$file'"
        fi
    fi
    
    if [[ $errors -eq 0 ]]; then
        print_success "File '$file' validation passed"
    else
        print_error "File '$file' validation failed with $errors errors"
        return 1
    fi
    
    return 0
}

# Main validation logic
main() {
    local exit_code=0
    
    print_header "Toka Environment Validation"
    
    # Validate template files
    print_info "Validating template files..."
    
    if validate_env_file "config/environments/toka.env.example" "template"; then
        print_success "Main template validation passed"
    else
        ((exit_code++))
    fi
    
    if validate_env_file "config/environments/cursor.env.example" "template"; then
        print_success "Cursor template validation passed"
    else
        ((exit_code++))
    fi
    
    if validate_env_file "config/environments/dev.env.example" "template"; then
        print_success "Development template validation passed"
    else
        ((exit_code++))
    fi
    
    if validate_env_file "config/environments/prod.env.example" "template"; then
        print_success "Production template validation passed"
    else
        ((exit_code++))
    fi
    
    # Validate actual environment files if they exist
    print_info "Validating actual environment files..."
    
    if [[ -f ".env.local" ]]; then
        if validate_env_file ".env.local" "main"; then
            print_success "Local environment validation passed"
        else
            ((exit_code++))
        fi
    else
        print_info "No .env.local file found (optional)"
    fi
    
    if [[ -f ".env.cursor" ]]; then
        if validate_env_file ".env.cursor" "cursor"; then
            print_success "Cursor environment validation passed"
        else
            ((exit_code++))
        fi
    else
        print_info "No .env.cursor file found (optional)"
    fi
    
    if [[ -f ".env.dev" ]]; then
        if validate_env_file ".env.dev" "development"; then
            print_success "Development environment validation passed"
        else
            ((exit_code++))
        fi
    else
        print_info "No .env.dev file found (optional)"
    fi
    
    if [[ -f ".env.prod" ]]; then
        if validate_env_file ".env.prod" "production"; then
            print_success "Production environment validation passed"
        else
            ((exit_code++))
        fi
    else
        print_info "No .env.prod file found (optional)"
    fi
    
    # Validate component-specific files
    if [[ -f "crates/toka-collaborative-auth/auth.env" ]]; then
        if validate_env_file "crates/toka-collaborative-auth/auth.env" "auth"; then
            print_success "Auth environment validation passed"
        else
            ((exit_code++))
        fi
    else
        print_info "No auth.env file found (optional)"
    fi
    
    print_header "Validation Summary"
    
    if [[ $exit_code -eq 0 ]]; then
        print_success "All environment validations passed!"
    else
        print_error "Environment validation failed with $exit_code errors"
        echo
        print_info "To fix issues:"
        echo "1. Copy templates: cp config/environments/toka.env.example .env.local"
        echo "2. Fill in your API keys and secrets"
        echo "3. Run validation again: ./scripts/validate-env.sh"
    fi
    
    return $exit_code
}

# Run main function
main "$@" 