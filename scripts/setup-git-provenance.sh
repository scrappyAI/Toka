#!/bin/bash
# Setup script for Git-based Documentation Provenance and AI Code Tracking System
# Installs hooks, configures Git, and initializes the tracking system

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    
    case $level in
        "INFO")  echo -e "${GREEN}[INFO]${NC} $message" ;;
        "WARN")  echo -e "${YELLOW}[WARN]${NC} $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $message" ;;
        "DEBUG") echo -e "${BLUE}[DEBUG]${NC} $message" ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log "INFO" "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check for required commands
    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi
    
    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
    fi
    
    if ! command -v uuidgen >/dev/null 2>&1 && ! command -v uuid >/dev/null 2>&1; then
        missing_deps+=("uuidgen or uuid")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log "ERROR" "Missing required dependencies: ${missing_deps[*]}"
        log "INFO" "Please install missing dependencies and run again"
        exit 1
    fi
    
    log "INFO" "All prerequisites satisfied"
}

# Install Git hooks
install_git_hooks() {
    log "INFO" "Installing Git hooks..."
    
    # Create hooks directory if it doesn't exist
    mkdir -p "$PROJECT_ROOT/.git/hooks"
    
    # Copy hooks from .githooks to .git/hooks
    if [[ -d "$PROJECT_ROOT/.githooks" ]]; then
        for hook in pre-commit post-commit; do
            if [[ -f "$PROJECT_ROOT/.githooks/$hook" ]]; then
                cp "$PROJECT_ROOT/.githooks/$hook" "$PROJECT_ROOT/.git/hooks/$hook"
                chmod +x "$PROJECT_ROOT/.git/hooks/$hook"
                log "INFO" "Installed $hook hook"
            fi
        done
    else
        log "WARN" ".githooks directory not found, skipping hook installation"
    fi
    
    # Configure Git to use the hooks
    git config core.hooksPath .git/hooks
    
    log "INFO" "Git hooks installed and configured"
}

# Configure Git settings for AI provenance
configure_git() {
    log "INFO" "Configuring Git for AI provenance tracking..."
    
    # Configure commit trailers
    git config --local trailer.AI-Generated.key "AI-Generated"
    git config --local trailer.AI-Model.key "AI-Model"
    git config --local trailer.AI-Confidence.key "AI-Confidence"
    git config --local trailer.Human-Review.key "Human-Review"
    git config --local trailer.Provenance-Tracked.key "Provenance-Tracked"
    
    # Configure commit template with AI metadata prompts
    cat > "$PROJECT_ROOT/.gitmessage" << 'EOF'
# Commit message template with AI provenance tracking
# 
# Format: <type>(<scope>): <description>
#
# Types: feat, fix, docs, style, refactor, test, chore
# Scopes: kernel, auth, cli, storage, security, docs, etc.
#
# If this commit contains AI-generated content, add these trailers:
# AI-Generated: true
# AI-Model: claude-3.5-sonnet|gpt-4|etc
# AI-Confidence: low|medium|high
# Human-Review: true|false
# Provenance-Tracked: true
#
# Example:
# feat(auth): implement JWT token validation
# 
# AI-Generated: true
# AI-Model: claude-3.5-sonnet
# AI-Confidence: high
# Human-Review: true
# Provenance-Tracked: true
EOF
    
    git config --local commit.template .gitmessage
    
    log "INFO" "Git configuration completed"
}

# Initialize provenance tracking
initialize_provenance() {
    log "INFO" "Initializing provenance tracking system..."
    
    # Make provenance script executable
    if [[ -f "$PROJECT_ROOT/scripts/git-doc-provenance.sh" ]]; then
        chmod +x "$PROJECT_ROOT/scripts/git-doc-provenance.sh"
        
        # Initialize the provenance system
        "$PROJECT_ROOT/scripts/git-doc-provenance.sh" init
        
        log "INFO" "Provenance tracking initialized"
    else
        log "WARN" "Provenance script not found, skipping initialization"
    fi
}

# Create documentation audit configuration
create_audit_config() {
    log "INFO" "Creating documentation audit configuration..."
    
    cat > "$PROJECT_ROOT/.doc-audit.yaml" << 'EOF'
# Documentation Audit Configuration
version: "1.0.0"
description: "Configuration for automated documentation auditing and AI provenance tracking"

# Documentation directories to audit
documentation_paths:
  - "docs/"
  - "README.md"
  - "CHANGELOG.md"
  - "*.md"

# Link validation settings
link_validation:
  # Check internal links
  check_internal: true
  # Check external links (may be slow)
  check_external: false
  # Timeout for external link checks (seconds)
  external_timeout: 10
  # Ignore patterns
  ignore_patterns:
    - "mailto:"
    - "javascript:"
    - "#"

# AI provenance tracking settings
ai_tracking:
  # Automatically detect AI-generated content
  auto_detect: true
  # AI model detection patterns
  model_patterns:
    - "claude-[0-9.-]+"
    - "gpt-[0-9.-]+"
    - "anthropic"
    - "openai"
  # Content markers that indicate AI generation
  content_markers:
    - "AI-Generated"
    - "Generated by"
    - "AI-assisted"
    - "LLM"

# Date validation settings
date_validation:
  # Check for future dates (likely hallucinated)
  check_future_dates: true
  # Check for suspicious date patterns
  check_suspicious_patterns: true
  # Exempt marker for intentional historical dates
  exempt_marker: "DATE:EXEMPT"

# Audit report settings
audit_reports:
  # Generate reports automatically
  auto_generate: true
  # Report format
  format: "json"
  # Retention period for reports (days)
  retention_days: 30
  # Include detailed file analysis
  detailed_analysis: true
EOF
    
    log "INFO" "Documentation audit configuration created"
}

# Create CI/CD integration
create_ci_integration() {
    log "INFO" "Creating CI/CD integration..."
    
    # Create GitHub Actions workflow for documentation audit
    mkdir -p "$PROJECT_ROOT/.github/workflows"
    
    cat > "$PROJECT_ROOT/.github/workflows/doc-audit.yml" << 'EOF'
name: Documentation Audit and AI Provenance

on:
  push:
    branches: [ main, develop ]
    paths:
      - 'docs/**'
      - '*.md'
      - 'scripts/git-doc-provenance.sh'
  pull_request:
    branches: [ main, develop ]
    paths:
      - 'docs/**'
      - '*.md'

jobs:
  audit-documentation:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout repository
      uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Setup dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y jq uuid-runtime
    
    - name: Initialize provenance tracking
      run: |
        chmod +x scripts/git-doc-provenance.sh
        ./scripts/git-doc-provenance.sh init
    
    - name: Validate documentation links
      run: |
        ./scripts/git-doc-provenance.sh validate
    
    - name: Generate audit report
      run: |
        ./scripts/git-doc-provenance.sh audit
    
    - name: Search for AI-generated content
      run: |
        echo "=== AI-Generated Content Report ==="
        ./scripts/git-doc-provenance.sh search all || true
    
    - name: Upload audit artifacts
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: documentation-audit
        path: |
          .git/provenance/audit-report-*.json
          .git/provenance/doc-linkage.json
        retention-days: 30
EOF
    
    log "INFO" "CI/CD integration created"
}

# Display usage instructions
show_usage() {
    echo -e "${BLUE}=== Git Documentation Provenance System Setup Complete ===${NC}"
    echo ""
    echo -e "${GREEN}The system has been configured with the following features:${NC}"
    echo "• AI-generated code tracking with commit trailers"
    echo "• Documentation link validation"
    echo "• Date accuracy enforcement (prevents LLM hallucinations)"
    echo "• Automated audit trail generation"
    echo "• Git hooks for pre/post-commit validation"
    echo ""
    echo -e "${YELLOW}Usage Examples:${NC}"
    echo "• Track AI-generated file:"
    echo "  ./scripts/git-doc-provenance.sh track src/main.rs claude-3.5-sonnet code high true"
    echo ""
    echo "• Validate documentation links:"
    echo "  ./scripts/git-doc-provenance.sh validate"
    echo ""
    echo "• Generate audit report:"
    echo "  ./scripts/git-doc-provenance.sh audit"
    echo ""
    echo "• Commit with AI provenance:"
    echo "  ./scripts/git-doc-provenance.sh commit 'feat: add new feature' claude-3.5-sonnet src/main.rs"
    echo ""
    echo "• Search AI-generated code:"
    echo "  ./scripts/git-doc-provenance.sh search model claude-3.5-sonnet"
    echo ""
    echo -e "${GREEN}Git hooks are now active and will automatically:${NC}"
    echo "• Detect and track AI-generated content"
    echo "• Validate documentation links on commit"
    echo "• Check for date accuracy issues"
    echo "• Generate provenance records"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Review the commit message template in .gitmessage"
    echo "2. Customize .doc-audit.yaml for your needs"
    echo "3. Test the system with a commit containing documentation"
    echo "4. Review generated audit reports in .git/provenance/"
}

# Main setup function
main() {
    echo -e "${BLUE}=== Setting up Git Documentation Provenance System ===${NC}"
    
    check_prerequisites
    install_git_hooks
    configure_git
    initialize_provenance
    create_audit_config
    create_ci_integration
    show_usage
    
    log "INFO" "Setup completed successfully!"
}

# Run main function
main "$@" 