#!/bin/bash
# Semantic Tagging Script for Toka Workspace Crates
# Adds GitHub-friendly keywords, topics, and categories to all Cargo.toml files

set -euo pipefail

# Get canonical current date
TODAY=$(date -u +%Y-%m-%d)
echo "Adding semantic tags to Toka workspace crates - $TODAY"

# Define the correct repository URL
REPO_URL="https://github.com/scrappyAI/Toka"

# Function to add semantic tags to a Cargo.toml file
add_semantic_tags() {
    local crate_path="$1"
    local crate_name="$2"
    local keywords="$3"
    local categories="$4"
    
    local cargo_toml="$crate_path/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        echo "⚠️  Cargo.toml not found: $cargo_toml"
        return
    fi
    
    echo "  📝 Adding tags to $crate_name"
    
    # Create a temporary file for the updated Cargo.toml
    local temp_file=$(mktemp)
    
    # Read the existing Cargo.toml and add semantic tags
    {
        # Copy everything up to dependencies section
        awk '/^\[dependencies\]/{print; exit} 1' "$cargo_toml"
        
        # Add semantic tags if not already present
        if ! grep -q "^keywords = " "$cargo_toml"; then
            echo "keywords = [$keywords]"
        fi
        
        if ! grep -q "^categories = " "$cargo_toml"; then
            echo "categories = [$categories]"
        fi
        
        # Add repository URL if not correct
        if ! grep -q "repository = \"$REPO_URL\"" "$cargo_toml"; then
            echo "repository = \"$REPO_URL\""
        fi
        
        echo ""
        
        # Copy the rest of the file
        awk '/^\[dependencies\]/{found=1; next} found' "$cargo_toml"
    } > "$temp_file"
    
    # Replace the original file
    mv "$temp_file" "$cargo_toml"
    
    echo "  ✅ Updated $crate_name"
}

# Core Infrastructure Crates
echo "=== Core Infrastructure Crates ==="

add_semantic_tags "crates/toka-kernel" "toka-kernel" \
    '"kernel", "security", "deterministic", "agent-os", "capability-tokens"' \
    '"os", "security", "concurrency"'

add_semantic_tags "crates/toka-runtime" "toka-runtime" \
    '"runtime", "execution", "dynamic", "sandbox", "agent-execution"' \
    '"os", "virtualization", "concurrency"'

add_semantic_tags "crates/toka-types" "toka-types" \
    '"types", "core", "primitives", "agent-types", "serialization"' \
    '"data-structures", "serialization"'

add_semantic_tags "crates/toka-auth" "toka-auth" \
    '"authentication", "authorization", "jwt", "capability-tokens", "security"' \
    '"authentication", "security"'

add_semantic_tags "crates/toka-bus-core" "toka-bus-core" \
    '"event-bus", "messaging", "deterministic", "core", "events"' \
    '"concurrency", "data-structures"'

# Storage Layer Crates
echo "=== Storage Layer Crates ==="

add_semantic_tags "crates/toka-store-core" "toka-store-core" \
    '"storage", "traits", "abstractions", "backend", "persistence"' \
    '"database", "data-structures"'

add_semantic_tags "crates/toka-store-memory" "toka-store-memory" \
    '"storage", "memory", "in-memory", "testing", "development"' \
    '"database", "development-tools"'

add_semantic_tags "crates/toka-store-sled" "toka-store-sled" \
    '"storage", "sled", "persistent", "database", "key-value"' \
    '"database"'

add_semantic_tags "crates/toka-store-sqlite" "toka-store-sqlite" \
    '"storage", "sqlite", "sql", "database", "persistent"' \
    '"database"'

add_semantic_tags "crates/toka-store-semantic" "toka-store-semantic" \
    '"storage", "semantic", "vector", "embeddings", "search"' \
    '"database", "algorithms"'

# Agent & Orchestration Crates
echo "=== Agent & Orchestration Crates ==="

add_semantic_tags "crates/toka-agent-runtime" "toka-agent-runtime" \
    '"agent", "runtime", "execution", "capabilities", "lifecycle"' \
    '"concurrency", "os"'

add_semantic_tags "crates/toka-orchestration" "toka-orchestration" \
    '"orchestration", "coordination", "agents", "workflow", "dependency-resolution"' \
    '"concurrency", "algorithms"'

add_semantic_tags "crates/toka-orchestration-service" "toka-orchestration-service" \
    '"orchestration", "service", "coordination", "agent-spawning", "lifecycle"' \
    '"web-programming", "concurrency"'

# Tools & Integration Crates
echo "=== Tools & Integration Crates ==="

add_semantic_tags "crates/toka-tools" "toka-tools" \
    '"tools", "execution", "python", "wasm", "security", "discovery"' \
    '"development-tools", "virtualization"'

add_semantic_tags "crates/toka-llm-gateway" "toka-llm-gateway" \
    '"llm", "gateway", "api", "rate-limiting", "providers"' \
    '"web-programming", "api-bindings"'

add_semantic_tags "crates/toka-collaborative-auth" "toka-collaborative-auth" \
    '"collaboration", "oauth", "github", "authentication", "permissions"' \
    '"authentication", "web-programming"'

# CLI & Configuration Crates
echo "=== CLI & Configuration Crates ==="

add_semantic_tags "crates/toka-cli" "toka-cli" \
    '"cli", "command-line", "interface", "developer-tools", "ergonomics"' \
    '"command-line-interface", "development-tools"'

add_semantic_tags "crates/toka-config-cli" "toka-config-cli" \
    '"config", "cli", "configuration", "validation", "management"' \
    '"command-line-interface", "config"'

# Performance & Monitoring Crates
echo "=== Performance & Monitoring Crates ==="

add_semantic_tags "crates/toka-performance" "toka-performance" \
    '"performance", "monitoring", "observability", "metrics", "tracing"' \
    '"development-tools", "profiling"'

add_semantic_tags "crates/toka-testing" "toka-testing" \
    '"testing", "integration", "e2e", "test-framework", "quality"' \
    '"development-tools", "testing"'

# Security Crates
echo "=== Security Crates ==="

add_semantic_tags "crates/security/toka-capability-core" "toka-capability-core" \
    '"capabilities", "security", "tokens", "no-std", "core"' \
    '"security", "no-std"'

add_semantic_tags "crates/security/toka-capability-delegation" "toka-capability-delegation" \
    '"capabilities", "delegation", "hierarchy", "security", "tokens"' \
    '"security", "data-structures"'

add_semantic_tags "crates/security/toka-capability-jwt-hs256" "toka-capability-jwt-hs256" \
    '"jwt", "hs256", "security", "tokens", "validation"' \
    '"security", "cryptography"'

add_semantic_tags "crates/security/toka-key-rotation" "toka-key-rotation" \
    '"key-rotation", "security", "cryptography", "management", "events"' \
    '"security", "cryptography"'

add_semantic_tags "crates/security/toka-rate-limiter" "toka-rate-limiter" \
    '"rate-limiting", "security", "middleware", "algorithms", "policies"' \
    '"security", "algorithms"'

add_semantic_tags "crates/security/toka-revocation" "toka-revocation" \
    '"revocation", "security", "tokens", "rfc7009", "validation"' \
    '"security"'

add_semantic_tags "crates/security/toka-cvm" "toka-cvm" \
    '"cvm", "capability-validation", "wasm", "security", "module"' \
    '"security", "wasm"'

# Raft Consensus Crates
echo "=== Raft Consensus Crates ==="

add_semantic_tags "crates/raft-core" "raft-core" \
    '"raft", "consensus", "distributed", "algorithm", "replication"' \
    '"algorithms", "concurrency"'

add_semantic_tags "crates/raft-storage" "raft-storage" \
    '"raft", "storage", "persistence", "snapshots", "log"' \
    '"database", "algorithms"'

# Utility & Metadata Crates
echo "=== Utility & Metadata Crates ==="

add_semantic_tags "crates/toka-rule-metadata" "toka-rule-metadata" \
    '"rules", "metadata", "catalogue", "validation", "management"' \
    '"development-tools", "config"'

add_semantic_tags "crates/toka-demo-environment" "toka-demo-environment" \
    '"demo", "environment", "testing", "multi-agent", "showcase"' \
    '"development-tools", "testing"'

echo ""
echo "=== Semantic Tagging Summary ==="
echo "✅ Added semantic tags to all core workspace crates"
echo "✅ Standardized repository URLs to: $REPO_URL"
echo "✅ Added GitHub-friendly keywords and categories"
echo "✅ Enhanced discoverability for:"
echo "   - Core infrastructure (kernel, runtime, types, auth)"
echo "   - Storage layer (core, memory, sled, sqlite, semantic)"
echo "   - Agent orchestration (agent-runtime, orchestration)"
echo "   - Tools & integration (tools, llm-gateway, collaborative-auth)"
echo "   - CLI & configuration (cli, config-cli)"
echo "   - Performance & monitoring (performance, testing)"
echo "   - Security (capability-*, key-rotation, rate-limiter, revocation, cvm)"
echo "   - Raft consensus (raft-core, raft-storage)"
echo "   - Utilities (rule-metadata, demo-environment)"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Test builds: cargo check --workspace"
echo "3. Commit semantic improvements" 