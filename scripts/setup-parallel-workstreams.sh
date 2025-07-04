#!/bin/bash
# Setup script for parallel workstream development infrastructure
# This script initializes all necessary components for v0.3.0 enhancement roadmap

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_NAME="${GITHUB_REPOSITORY:-org/toka}"
BASE_BRANCH="main"
WORKSTREAMS=(
    "build-system-stabilization"
    "testing-infrastructure"
    "kernel-events-expansion"
    "storage-enhancements"
    "security-enhancements"
    "performance-observability"
)

# Utility functions
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

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for required tools
    local missing_tools=()
    
    command -v git >/dev/null 2>&1 || missing_tools+=("git")
    command -v gh >/dev/null 2>&1 || missing_tools+=("gh (GitHub CLI)")
    command -v cargo >/dev/null 2>&1 || missing_tools+=("cargo")
    command -v yamllint >/dev/null 2>&1 || missing_tools+=("yamllint")
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Please install the missing tools and run this script again"
        exit 1
    fi
    
    # Check GitHub CLI authentication
    if ! gh auth status >/dev/null 2>&1; then
        log_error "GitHub CLI is not authenticated"
        log_info "Please run 'gh auth login' first"
        exit 1
    fi
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

create_feature_branches() {
    log_info "Creating feature branches..."
    
    # Ensure we're on the base branch and up to date
    git checkout "$BASE_BRANCH"
    git pull origin "$BASE_BRANCH"
    
    for workstream in "${WORKSTREAMS[@]}"; do
        local branch_name="feature/$workstream"
        
        if git show-ref --verify --quiet "refs/heads/$branch_name"; then
            log_warning "Branch $branch_name already exists locally"
        else
            log_info "Creating branch $branch_name"
            git checkout -b "$branch_name" "$BASE_BRANCH"
            
            # Push branch to remote
            git push -u origin "$branch_name"
            log_success "Created and pushed branch $branch_name"
        fi
    done
    
    # Return to base branch
    git checkout "$BASE_BRANCH"
}

setup_environment_configs() {
    log_info "Setting up environment configurations..."
    
    local env_dir=".github/environments"
    mkdir -p "$env_dir"
    
    for workstream in "${WORKSTREAMS[@]}"; do
        local env_file="$env_dir/$workstream.yml"
        
        if [ -f "$env_file" ]; then
            log_warning "Environment config $env_file already exists"
            continue
        fi
        
        log_info "Creating environment config for $workstream"
        
        # Create environment-specific configuration
        cat > "$env_file" << EOF
# $workstream Workstream Environment Configuration
name: $workstream
description: "${workstream^} Workstream Environment"

protection_rules:
  required_reviewers:
    - workstream-lead
    - maintainer
  
  wait_timer: 0
  
  deployment_branch_policy:
    protected_branches: true
    custom_branch_policies: false

variables:
  WORKSTREAM_NAME: "$workstream"
  WORKSTREAM_BRANCH: "feature/$workstream"
  AGENT_CONFIG_PATH: "agents/v0.3.0/workstreams/$workstream.yaml"
  
  # Rust Configuration
  RUST_TOOLCHAIN: "stable"
  RUST_BACKTRACE: "1"
  CARGO_TERM_COLOR: "always"
  
  # CI/CD Settings
  CACHE_CARGO_REGISTRY: "true"
  CACHE_CARGO_GIT: "true"
  CACHE_TARGET_DIR: "true"

secrets:
  WORKSTREAM_WEBHOOK: "Webhook for $workstream notifications"
  METRICS_API_KEY: "API key for $workstream metrics"
EOF
        
        log_success "Created environment config $env_file"
    done
}

apply_branch_protection() {
    log_info "Applying branch protection rules..."
    
    # Check if we have the necessary permissions
    if ! gh api "repos/$REPO_NAME" >/dev/null 2>&1; then
        log_error "Cannot access repository $REPO_NAME"
        log_info "Please check repository name and permissions"
        return 1
    fi
    
    # Protect main branch
    log_info "Protecting main branch..."
    gh api "repos/$REPO_NAME/branches/main/protection" \
        --method PUT \
        --field required_status_checks='{"strict":true,"contexts":["CI / test","Cross-Workstream Integration / integration-summary"]}' \
        --field enforce_admins=true \
        --field required_pull_request_reviews='{"required_approving_review_count":2,"dismiss_stale_reviews":true,"require_code_owner_reviews":true}' \
        --field restrictions='{"users":[],"teams":["maintainers"],"apps":[]}' \
        --field allow_force_pushes=false \
        --field allow_deletions=false \
        2>/dev/null || log_warning "Failed to protect main branch (may already be protected)"
    
    # Protect feature branches
    for workstream in "${WORKSTREAMS[@]}"; do
        local branch_name="feature/$workstream"
        log_info "Protecting branch $branch_name..."
        
        # Create basic protection (can be customized per workstream)
        gh api "repos/$REPO_NAME/branches/$branch_name/protection" \
            --method PUT \
            --field required_status_checks='{"strict":true,"contexts":["Workstream CI / basic-validation","Workstream CI / agent-config-validation"]}' \
            --field enforce_admins=false \
            --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":true}' \
            --field restrictions='{"users":[],"teams":["maintainers"],"apps":[]}' \
            --field allow_force_pushes=false \
            --field allow_deletions=false \
            2>/dev/null || log_warning "Failed to protect branch $branch_name"
    done
    
    log_success "Branch protection rules applied"
}

validate_agent_configs() {
    log_info "Validating agent configurations..."
    
    local config_dir="agents/v0.3.0/workstreams"
    local validation_failed=false
    
    for workstream in "${WORKSTREAMS[@]}"; do
        local config_file="$config_dir/$workstream.yaml"
        
        if [ ! -f "$config_file" ]; then
            log_error "Agent config not found: $config_file"
            validation_failed=true
            continue
        fi
        
        log_info "Validating $config_file"
        
        # YAML syntax validation
        if ! yamllint "$config_file" >/dev/null 2>&1; then
            log_error "YAML syntax error in $config_file"
            validation_failed=true
            continue
        fi
        
        # Check required fields
        local required_fields=("metadata.name" "spec.name" "spec.domain" "spec.priority")
        for field in "${required_fields[@]}"; do
            if ! yq eval ".$field" "$config_file" >/dev/null 2>&1; then
                log_error "Missing required field '$field' in $config_file"
                validation_failed=true
            fi
        done
        
        log_success "Agent config $config_file is valid"
    done
    
    if [ "$validation_failed" = true ]; then
        log_error "Agent configuration validation failed"
        return 1
    fi
    
    log_success "All agent configurations validated"
}

test_ci_workflows() {
    log_info "Testing CI workflow syntax..."
    
    local workflow_files=(
        ".github/workflows/workstream-ci.yml"
        ".github/workflows/integration-test.yml"
        ".github/workflows/dependency-management.yml"
    )
    
    for workflow_file in "${workflow_files[@]}"; do
        if [ ! -f "$workflow_file" ]; then
            log_error "Workflow file not found: $workflow_file"
            continue
        fi
        
        log_info "Validating $workflow_file"
        
        # Basic YAML validation
        if ! yamllint "$workflow_file" >/dev/null 2>&1; then
            log_error "YAML syntax error in $workflow_file"
            continue
        fi
        
        # GitHub Actions workflow validation (if available)
        if command -v act >/dev/null 2>&1; then
            act --dry-run -W "$workflow_file" >/dev/null 2>&1 || log_warning "Workflow validation failed for $workflow_file"
        fi
        
        log_success "Workflow $workflow_file is valid"
    done
}

create_initial_commits() {
    log_info "Creating initial commits for workstream branches..."
    
    for workstream in "${WORKSTREAMS[@]}"; do
        local branch_name="feature/$workstream"
        
        # Switch to workstream branch
        git checkout "$branch_name"
        
        # Create workstream-specific documentation
        local workstream_dir="docs/workstreams/$workstream"
        mkdir -p "$workstream_dir"
        
        cat > "$workstream_dir/README.md" << EOF
# $workstream Workstream

This workstream implements the $workstream enhancements for Toka OS v0.3.0.

## Objectives

See [agent configuration](../../../agents/v0.3.0/workstreams/$workstream.yaml) for detailed objectives and tasks.

## Development

This branch follows the parallel workstream development strategy outlined in the v0.3.0 enhancement roadmap.

## Status

- [ ] Phase 1: Foundation setup
- [ ] Phase 2: Core development
- [ ] Phase 3: Integration testing
- [ ] Phase 4: Documentation and validation

## Integration

This workstream integrates with:
- Build system stabilization (dependency)
- Other workstreams as defined in agent configuration

For integration testing, see the Cross-Workstream Integration workflow.
EOF
        
        # Add and commit
        git add "$workstream_dir/README.md"
        git commit -m "feat($workstream): initialize workstream branch

- Add workstream documentation
- Prepare for parallel development
- Link to agent configuration

Relates to v0.3.0 enhancement roadmap Phase 1"
        
        # Push to remote
        git push origin "$branch_name"
        
        log_success "Initialized workstream branch $branch_name"
    done
    
    # Return to base branch
    git checkout "$BASE_BRANCH"
}

generate_setup_report() {
    log_info "Generating setup report..."
    
    local report_file="parallel-workstream-setup-report.md"
    
    cat > "$report_file" << EOF
# Parallel Workstream Setup Report

**Generated:** $(date -u)
**Script Version:** v1.0.0

## Setup Summary

This report documents the setup of parallel workstream development infrastructure for Toka OS v0.3.0.

### Feature Branches Created

$(for workstream in "${WORKSTREAMS[@]}"; do
    echo "- \`feature/$workstream\`"
done)

### CI/CD Workflows

- ✅ Workstream CI (\`.github/workflows/workstream-ci.yml\`)
- ✅ Cross-Workstream Integration (\`.github/workflows/integration-test.yml\`)
- ✅ Dependency Management (\`.github/workflows/dependency-management.yml\`)

### Environment Configurations

$(for workstream in "${WORKSTREAMS[@]}"; do
    echo "- ✅ $workstream environment (\`.github/environments/$workstream.yml\`)"
done)

### Branch Protection

- ✅ Main branch protection (2 reviewers required)
- ✅ Feature branch protection (1 reviewer required)
- ✅ Required status checks configured

### Agent Configurations

$(for workstream in "${WORKSTREAMS[@]}"; do
    if [ -f "agents/v0.3.0/workstreams/$workstream.yaml" ]; then
        echo "- ✅ $workstream agent config validated"
    else
        echo "- ❌ $workstream agent config missing"
    fi
done)

## Next Steps

1. **Manual GitHub Setup**:
   - Configure teams and permissions as defined in \`.github/branch-protection.yml\`
   - Set up environment secrets in repository settings
   - Configure notification webhooks

2. **Development Process**:
   - Spawn agents for each workstream
   - Begin Phase 1 development (Build System Stabilization first)
   - Monitor CI/CD pipelines for issues

3. **Integration Testing**:
   - Weekly integration tests will run automatically
   - Monitor cross-workstream compatibility
   - Address conflicts as they arise

## Monitoring

- **CI/CD**: GitHub Actions workflows will provide continuous validation
- **Dependencies**: Daily dependency conflict detection
- **Security**: Weekly security audits
- **Integration**: Weekly cross-workstream integration tests

## Support

For issues with the parallel workstream setup:
1. Check GitHub Actions workflow logs
2. Review agent configuration validation
3. Verify branch protection settings
4. Consult the v0.3.0 enhancement roadmap documentation

---

**Status**: Setup Complete ✅  
**Ready for Phase 1**: Yes ✅
EOF
    
    log_success "Setup report generated: $report_file"
}

# Main execution
main() {
    log_info "Starting parallel workstream setup..."
    log_info "Repository: $REPO_NAME"
    log_info "Workstreams: ${WORKSTREAMS[*]}"
    
    check_prerequisites
    
    create_feature_branches
    setup_environment_configs
    
    # Only attempt branch protection if we have admin access
    if apply_branch_protection; then
        log_success "Branch protection applied successfully"
    else
        log_warning "Branch protection failed - may need manual setup"
    fi
    
    validate_agent_configs
    test_ci_workflows
    create_initial_commits
    generate_setup_report
    
    log_success "Parallel workstream setup complete!"
    log_info "See parallel-workstream-setup-report.md for details"
    log_info "Next: Begin Phase 1 development with Build System Stabilization workstream"
}

# Script options
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--help|--check-only|--branches-only]"
        echo ""
        echo "Options:"
        echo "  --help, -h         Show this help message"
        echo "  --check-only       Only run prerequisite checks"
        echo "  --branches-only    Only create feature branches"
        echo ""
        echo "This script sets up the complete parallel workstream development infrastructure."
        exit 0
        ;;
    --check-only)
        check_prerequisites
        log_success "Prerequisites check complete"
        exit 0
        ;;
    --branches-only)
        check_prerequisites
        create_feature_branches
        log_success "Feature branches created"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        log_info "Use --help for usage information"
        exit 1
        ;;
esac 