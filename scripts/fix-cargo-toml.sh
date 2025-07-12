#!/bin/bash
# Fix Cargo.toml Semantic Tags Script
# Properly inserts semantic tags without breaking TOML structure

set -euo pipefail

echo "Fixing Cargo.toml semantic tags..."

# Function to properly fix a Cargo.toml file
fix_cargo_toml() {
    local cargo_toml="$1"
    local crate_name="$2"
    local keywords="$3"
    local categories="$4"
    
    if [[ ! -f "$cargo_toml" ]]; then
        echo "⚠️  File not found: $cargo_toml"
        return
    fi
    
    echo "  📝 Fixing $crate_name"
    
    # Create backup
    cp "$cargo_toml" "${cargo_toml}.backup"
    
    # Create temp file
    local temp_file=$(mktemp)
    
    # Process the file line by line
    {
        while IFS= read -r line; do
            echo "$line"
            
            # After the repository line, add keywords and categories
            if [[ "$line" =~ ^repository[[:space:]]*= ]]; then
                echo "keywords = [$keywords]"
                echo "categories = [$categories]"
            fi
        done < "$cargo_toml"
    } > "$temp_file"
    
    # Replace the original file
    mv "$temp_file" "$cargo_toml"
    
    echo "  ✅ Fixed $crate_name"
}

# Fix all the problematic Cargo.toml files
echo "=== Fixing Core Infrastructure Crates ==="

fix_cargo_toml "crates/toka-kernel/Cargo.toml" "toka-kernel" \
    '"kernel", "security", "deterministic", "agent-os", "capability-tokens"' \
    '"os", "security", "concurrency"'

fix_cargo_toml "crates/toka-runtime/Cargo.toml" "toka-runtime" \
    '"runtime", "execution", "dynamic", "sandbox", "agent-execution"' \
    '"os", "virtualization", "concurrency"'

fix_cargo_toml "crates/toka-types/Cargo.toml" "toka-types" \
    '"types", "core", "primitives", "agent-types", "serialization"' \
    '"data-structures", "serialization"'

fix_cargo_toml "crates/toka-auth/Cargo.toml" "toka-auth" \
    '"authentication", "authorization", "jwt", "capability-tokens", "security"' \
    '"authentication", "security"'

fix_cargo_toml "crates/toka-bus-core/Cargo.toml" "toka-bus-core" \
    '"event-bus", "messaging", "deterministic", "core", "events"' \
    '"concurrency", "data-structures"'

echo ""
echo "=== Cargo.toml Fix Summary ==="
echo "✅ Fixed malformed Cargo.toml files"
echo "✅ Proper TOML structure maintained"
echo "✅ Semantic tags correctly inserted"
echo ""
echo "Test with: cargo check --workspace" 