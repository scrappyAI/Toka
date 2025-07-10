#!/bin/bash

# GitHub Authentication Setup for Toka Dev Container
# This script safely configures GitHub authentication for workspace access

set -e

echo "🔐 Setting up GitHub Workspace Authentication..."
echo "================================================="

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

# Function to check if GitHub CLI is available
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        print_error "GitHub CLI not found. This should have been installed by devcontainer features."
        return 1
    fi
    print_status "GitHub CLI is available: $(gh --version | head -n1)"
    return 0
}

# Function to check if already authenticated
check_existing_auth() {
    if gh auth status &> /dev/null; then
        print_status "Already authenticated with GitHub"
        gh auth status
        return 0
    fi
    return 1
}

# Function to setup GitHub authentication
setup_github_auth() {
    print_info "Setting up GitHub authentication..."
    
    # Check for environment variable token first
    if [[ -n "$GITHUB_TOKEN" ]]; then
        print_info "Found GITHUB_TOKEN environment variable"
        echo "$GITHUB_TOKEN" | gh auth login --with-token
        if gh auth status &> /dev/null; then
            print_status "Successfully authenticated using GITHUB_TOKEN"
            return 0
        else
            print_warning "GITHUB_TOKEN authentication failed, falling back to device flow"
        fi
    fi
    
    # Fall back to device flow for interactive setup
    print_info "Starting GitHub device flow authentication..."
    print_info "This will open a browser window for authentication"
    
    # Use device flow with web browser
    if gh auth login --web --scopes "repo,read:org,read:user,gist" --hostname github.com; then
        print_status "Successfully authenticated with GitHub via device flow"
        return 0
    else
        print_error "GitHub authentication failed"
        return 1
    fi
}

# Function to configure git with GitHub user info
setup_git_config() {
    print_info "Configuring Git with GitHub user information..."
    
    # Get GitHub user info
    local github_user
    local github_email
    
    github_user=$(gh api user --jq '.login' 2>/dev/null || echo "")
    github_email=$(gh api user --jq '.email' 2>/dev/null || echo "")
    
    # If email is null from API, try to get primary email
    if [[ "$github_email" == "null" || -z "$github_email" ]]; then
        github_email=$(gh api user/emails --jq '.[] | select(.primary == true) | .email' 2>/dev/null || echo "")
    fi
    
    # Configure git if we have the info
    if [[ -n "$github_user" ]]; then
        git config --global user.name "$github_user"
        print_status "Set git user.name to: $github_user"
    fi
    
    if [[ -n "$github_email" && "$github_email" != "null" ]]; then
        git config --global user.email "$github_email"
        print_status "Set git user.email to: $github_email"
    else
        print_warning "Could not retrieve email from GitHub. Please set manually: git config --global user.email your-email@example.com"
    fi
    
    # Set up other useful git configurations
    git config --global init.defaultBranch main
    git config --global pull.rebase false
    git config --global push.default simple
    
    print_status "Git configuration completed"
}

# Function to setup GitHub CLI aliases and helpful commands
setup_gh_aliases() {
    print_info "Setting up helpful GitHub CLI aliases..."
    
    # Set up useful aliases
    gh alias set pv 'pr view'
    gh alias set pc 'pr create'
    gh alias set pm 'pr merge'
    gh alias set iv 'issue view'
    gh alias set ic 'issue create'
    gh alias set repo-info 'repo view'
    gh alias set workflows 'workflow list'
    
    print_status "GitHub CLI aliases configured"
}

# Function to test GitHub access
test_github_access() {
    print_info "Testing GitHub access..."
    
    # Test basic API access
    if gh api user --jq '.login' &> /dev/null; then
        local username
        username=$(gh api user --jq '.login')
        print_status "GitHub API access confirmed for user: $username"
    else
        print_error "GitHub API access test failed"
        return 1
    fi
    
    # Test repository access if we're in a git repo
    if git rev-parse --git-dir &> /dev/null; then
        local repo_url
        repo_url=$(git config --get remote.origin.url 2>/dev/null || echo "")
        
        if [[ "$repo_url" == *"github.com"* ]]; then
            local repo_name
            repo_name=$(echo "$repo_url" | sed -n 's/.*github\.com[:/]\([^/]*\/[^/]*\)\.git/\1/p' | sed 's/\.git$//')
            
            if [[ -n "$repo_name" ]]; then
                if gh repo view "$repo_name" &> /dev/null; then
                    print_status "Repository access confirmed for: $repo_name"
                else
                    print_warning "Repository access limited for: $repo_name"
                fi
            fi
        fi
    fi
    
    return 0
}

# Function to create safe workspace environment
setup_workspace_env() {
    print_info "Setting up workspace environment..."
    
    # Create workspace-specific environment file
    cat > ~/.toka-workspace-env << 'EOF'
# Toka Workspace Environment
# This file contains workspace-specific configurations

export TOKA_WORKSPACE_MODE="github-authenticated"
export TOKA_GH_INTEGRATION="enabled"
export TOKA_AUTH_METHOD="github"
EOF

    # Add to bashrc for persistence
    if ! grep -q "toka-workspace-env" ~/.bashrc; then
        echo "" >> ~/.bashrc
        echo "# Toka Workspace Environment" >> ~/.bashrc
        echo "source ~/.toka-workspace-env 2>/dev/null || true" >> ~/.bashrc
    fi
    
    print_status "Workspace environment configured"
}

# Function to show helpful information
show_usage_info() {
    echo ""
    echo "🚀 GitHub Integration Ready!"
    echo "==========================="
    echo ""
    echo "Quick Commands:"
    echo "  gh auth status          - Check authentication status"
    echo "  gh repo view            - View current repository"
    echo "  gh pr list              - List pull requests"
    echo "  gh issue list           - List issues"
    echo "  gh workflow list        - List GitHub Actions workflows"
    echo ""
    echo "Aliases available:"
    echo "  pv                      - pr view"
    echo "  pc                      - pr create"
    echo "  pm                      - pr merge"
    echo "  iv                      - issue view"
    echo "  ic                      - issue create"
    echo ""
    echo "🔒 Security Notes:"
    echo "  - Your GitHub token is securely stored by GitHub CLI"
    echo "  - Token permissions: repo, read:org, read:user, gist"
    echo "  - Use 'gh auth logout' to revoke access if needed"
    echo ""
}

# Main execution function
main() {
    echo ""
    
    # Check prerequisites
    if ! check_gh_cli; then
        exit 1
    fi
    
    # Check if already authenticated
    if check_existing_auth; then
        print_info "GitHub authentication already configured"
    else
        # Setup authentication
        if ! setup_github_auth; then
            print_error "Failed to setup GitHub authentication"
            exit 1
        fi
    fi
    
    # Configure git
    setup_git_config
    
    # Setup aliases
    setup_gh_aliases
    
    # Test access
    if ! test_github_access; then
        print_warning "GitHub access test had issues, but continuing..."
    fi
    
    # Setup workspace environment
    setup_workspace_env
    
    # Show usage information
    show_usage_info
    
    print_status "GitHub workspace authentication setup complete!"
}

# Run main function
main "$@" 