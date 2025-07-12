# Environment File Organization Guide

**Date:** 2025-01-27  
**Status:** Comprehensive Environment Configuration Strategy  
**Scope:** Toka OS Environment File Organization and Security

## Executive Summary

This guide establishes a clear, secure, and maintainable environment file organization for the Toka OS project. It consolidates scattered environment files and provides clear guidelines for different contexts.

## 🎯 Organization Strategy

### **Core Principle: Scope-Based Organization**
Environment files should be organized by their **scope and responsibility**:

- **Global/Project-level**: Root directory
- **Context-specific**: `.cursor/`, `.devcontainer/`, `.github/`
- **Component-specific**: Individual crate directories
- **Environment-specific**: `config/environments/`

## 📁 Recommended File Structure

```
Toka/
├── .env.local                      # Personal/local overrides (gitignored)
├── .env.cursor                     # Cursor-specific config (gitignored)
├── config/
│   └── environments/
│       ├── toka.env.example        # Main Toka environment template
│       ├── dev.env.example         # Development environment
│       ├── prod.env.example        # Production environment
│       └── cursor.env.example      # Cursor environment template
├── .cursor/
│   └── cursor.env.example          # Cursor IDE specific template
├── .devcontainer/
│   └── devcontainer.env.example    # Dev container template
├── crates/
│   └── toka-collaborative-auth/
│       └── auth.env.example        # Auth-specific template
└── .github/
    └── workflows/
        └── ci.env.example          # CI/CD specific template
```

## 🔒 Security Guidelines

### **1. Never Commit Secrets**
- All `.env*` files (except `.env.example`) are gitignored
- Use templates (`.env.example`) for documentation
- Store actual secrets in local files only

### **2. Scope-Based Access**
- **Global secrets**: `.env.local` (API keys, tokens)
- **Context secrets**: `.env.cursor` (cursor-specific)
- **Component secrets**: crate-specific env files

### **3. Environment Separation**
- **Development**: `.env.local` or `dev.env`
- **Production**: `prod.env` (never in git)
- **Testing**: `test.env` (safe defaults)

## 📋 File Purpose and Usage

### **Root Level Files**

#### `.env.local` (Personal Development)
```bash
# Personal development environment
# Copy from config/environments/toka.env.example
# Fill in your actual API keys and tokens

# LLM Configuration
ANTHROPIC_API_KEY=your_anthropic_key_here
OPENAI_API_KEY=your_openai_key_here
LLM_PROVIDER=anthropic

# GitHub Integration
GITHUB_TOKEN=your_github_token_here
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret

# Development Settings
RUST_LOG=debug
TOKA_ENV=development
```

#### `.env.cursor` (Cursor IDE)
```bash
# Cursor Background Agent Environment
# Minimal configuration for Cursor IDE background agents

# Cursor-specific configuration
CURSOR_AGENT_MODE=true
RUST_LOG=warn
RUST_BACKTRACE=0

# Minimal resource usage
AGENT_POOL_SIZE=1
AGENT_SPAWN_TIMEOUT=10
AGENT_WORKSTREAM_TIMEOUT=300

# LLM Configuration (minimal)
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=your_api_key_here
LLM_MODEL=claude-3-5-sonnet-20241022
LLM_RATE_LIMIT=5
LLM_TIMEOUT=30
LLM_DEBUG=false
```

### **Template Files (Safe to Commit)**

#### `config/environments/toka.env.example`
```bash
# Toka OS Main Environment Template
# Copy this file to .env.local and customize

# =============================================================================
# LLM Provider Configuration
# =============================================================================
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=your_anthropic_api_key_here
LLM_MODEL=claude-3-5-sonnet-20241022
LLM_RATE_LIMIT=50
LLM_TIMEOUT=30

# =============================================================================
# Database Configuration
# =============================================================================
DATABASE_URL=sqlite:///app/data/toka.db
STORAGE_TYPE=sqlite

# =============================================================================
# Agent Configuration
# =============================================================================
AGENT_POOL_SIZE=10
MAX_CONCURRENT_AGENTS=8
AGENT_SPAWN_TIMEOUT=30
WORKSTREAM_TIMEOUT=3600

# =============================================================================
# Security Settings
# =============================================================================
JWT_SECRET=your-super-secret-jwt-key-change-in-production
AGENT_SANDBOX_ENABLED=true
CAPABILITY_VALIDATION=strict

# =============================================================================
# Performance Settings
# =============================================================================
RUST_LOG=info
RUST_BACKTRACE=1
TOKIO_WORKER_THREADS=4
```

## 🚀 Migration Steps

### **Step 1: Create Template Structure**
```bash
# Create organized template structure
mkdir -p config/environments/.templates
mkdir -p .cursor/.templates
mkdir -p .devcontainer/.templates
mkdir -p .github/.templates
```

### **Step 2: Consolidate Templates**
- Move all `.env.example` files to appropriate locations
- Create comprehensive templates for each context
- Remove duplicate or incomplete templates

### **Step 3: Update Documentation**
- Update all README files to reference new structure
- Update Docker configurations
- Update CI/CD workflows

### **Step 4: Secure Existing Files**
- Backup current environment files
- Move secrets to appropriate locations
- Update gitignore if needed

## 🔧 Usage Instructions

### **For Development**
```bash
# 1. Copy main template
cp config/environments/toka.env.example .env.local

# 2. Add your API keys
nano .env.local

# 3. Load environment
source .env.local
# or
export $(cat .env.local | grep -v '^#' | xargs)
```

### **For Cursor IDE**
```bash
# 1. Copy cursor template
cp config/environments/cursor.env.example .env.cursor

# 2. Customize for cursor
nano .env.cursor

# 3. Cursor will automatically load .env.cursor
```

### **For Production**
```bash
# 1. Copy production template
cp config/environments/prod.env.example .env.prod

# 2. Set production values
nano .env.prod

# 3. Use with Docker
docker-compose --env-file .env.prod up -d
```

## 📚 Documentation Updates Needed

### **Files to Update**
1. `README.md` - Main setup instructions
2. `SETUP_GUIDE.md` - Environment setup
3. `QUICKSTART.md` - Quick start commands
4. `docs/operations/README-Docker.md` - Docker instructions
5. `docs/development/README_TOKA_TESTING.md` - Testing setup
6. All crate-specific README files

### **Documentation Standards**
- Always reference the organized structure
- Provide clear copy-paste commands
- Include security warnings
- Show context-specific examples

## 🔍 Validation

### **Environment Validation Script**
```bash
#!/bin/bash
# scripts/validate-env.sh

validate_env_file() {
    local file=$1
    local context=$2
    
    echo "Validating $file for $context..."
    
    # Check required variables
    if [ "$context" = "development" ]; then
        check_required_vars "$file" "ANTHROPIC_API_KEY LLM_PROVIDER"
    elif [ "$context" = "cursor" ]; then
        check_required_vars "$file" "CURSOR_AGENT_MODE LLM_PROVIDER"
    fi
    
    # Check for secrets in templates
    if [[ "$file" == *.example ]]; then
        check_no_secrets "$file"
    fi
}

check_required_vars() {
    local file=$1
    local vars=$2
    
    for var in $vars; do
        if ! grep -q "^$var=" "$file"; then
            echo "ERROR: Missing required variable $var in $file"
            exit 1
        fi
    done
}

check_no_secrets() {
    local file=$1
    
    if grep -q "sk-" "$file" || grep -q "ghp_" "$file"; then
        echo "ERROR: Found actual secrets in template $file"
        exit 1
    fi
}
```

## 📊 Benefits of This Organization

1. **Clear Responsibility**: Each file has a clear purpose and scope
2. **Security**: Secrets are properly isolated and never committed
3. **Maintainability**: Easy to find and update configuration
4. **Consistency**: Standardized naming and structure
5. **Documentation**: Clear examples and instructions
6. **Flexibility**: Easy to customize for different contexts

## 🎯 Next Steps

1. **Implement Migration**: Follow the migration steps
2. **Update Documentation**: Update all references
3. **Test Configurations**: Validate all environments work
4. **Team Training**: Ensure team understands new structure
5. **Monitoring**: Set up validation in CI/CD

This organization will provide a solid foundation for environment management as the Toka OS project grows and evolves. 