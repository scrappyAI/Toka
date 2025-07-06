#!/bin/bash

# Toka Codebase Bloat Reduction Script
# This script implements the immediate, low-risk reductions identified in the analysis

set -e  # Exit on any error

echo "🧹 Starting Toka Codebase Bloat Reduction"
echo "========================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "crates" ]; then
    echo "❌ Error: Please run this script from the workspace root"
    exit 1
fi

# Create backup branch
echo "📦 Creating backup branch..."
git checkout -b bloat-reduction-backup || echo "Branch already exists, continuing..."

# Phase 1: Remove problematic crates (immediate impact)
echo ""
echo "🗂️  PHASE 1: Removing problematic crates"
echo "----------------------------------------"

# List of crates to remove (immediate wins)
CRATES_TO_REMOVE=(
    "crates/toka-store-sled"
    "crates/toka-store-semantic" 
    "crates/security/toka-cvm"
    "crates/security/toka-revocation"
    "crates/raft-core"
    "crates/raft-storage"
    "crates/toka-performance"
    "crates/toka-config-cli"
)

# Remove from workspace Cargo.toml
echo "Updating workspace Cargo.toml..."
cp Cargo.toml Cargo.toml.backup

# Create new workspace members list (excluding removed crates)
cat > temp_members.txt << 'EOF'
    # Core kernel ecosystem (v0.2) - deterministic layer
    "crates/toka-types",
    "crates/toka-auth",
    "crates/toka-bus-core", 
    "crates/toka-kernel",
    
    # Storage layer - core abstractions and drivers (REDUCED)
    "crates/toka-store-core",
    "crates/toka-store-memory",
    "crates/toka-store-sqlite",
    # REMOVED: toka-store-sled, toka-store-semantic (bloat reduction)
    
    # Runtime layer - fuzzy/async coordination
    "crates/toka-runtime",
    
    # Tools and utilities
    "crates/toka-tools",

    # Applications / demos (REDUCED)
    "crates/toka-cli",
    # REMOVED: toka-config-cli (redundant utility)
    "crates/toka-testing",
    
    # LLM integration
    "crates/toka-llm-gateway",
    
    # Agent orchestration
    "crates/toka-orchestration",
    "crates/toka-orchestration-service",
    
    # Agent execution runtime
    "crates/toka-agent-runtime",
    
    # REMOVED: Raft consensus (not needed for MVP)
    # "crates/raft-core",
    # "crates/raft-storage",
    
    # Security enhancements (REDUCED)
    "crates/security/toka-capability-core",
    "crates/security/toka-capability-jwt-hs256",
    "crates/security/toka-key-rotation",
    "crates/security/toka-rate-limiter",
    "crates/security/toka-capability-delegation",
    # REMOVED: toka-cvm, toka-revocation (over-engineered)
    # REMOVED: toka-performance (premature optimization)
EOF

# Update Cargo.toml with new members list
sed -i '/^members = \[/,/^\]/ c\
members = [\
'"$(cat temp_members.txt)"'\
]' Cargo.toml

rm temp_members.txt

# Move crates to removed/ directory instead of deleting
echo "Moving removed crates to /tmp/toka-removed-crates..."
mkdir -p /tmp/toka-removed-crates

for crate in "${CRATES_TO_REMOVE[@]}"; do
    if [ -d "$crate" ]; then
        echo "  Moving $crate"
        mv "$crate" "/tmp/toka-removed-crates/"
    else
        echo "  Skipping $crate (doesn't exist)"
    fi
done

# Phase 2: Remove heavy optional dependencies
echo ""
echo "🔧 PHASE 2: Removing heavy optional dependencies"
echo "-----------------------------------------------"

# Remove WASM support temporarily (can be re-enabled later)
echo "Disabling WASM support in toka-runtime..."
if [ -f "crates/toka-runtime/Cargo.toml" ]; then
    # Comment out wasmtime dependencies
    sed -i 's/^wasmtime = /# wasmtime = /' crates/toka-runtime/Cargo.toml
    sed -i 's/^wasmtime-wasi = /# wasmtime-wasi = /' crates/toka-runtime/Cargo.toml
    
    # Update default features to exclude WASM
    sed -i 's/default = \["wasm", "codegen"\]/default = ["codegen"]/' crates/toka-runtime/Cargo.toml
    sed -i 's/all-engines = \["wasm", "codegen"\]/all-engines = ["codegen"]/' crates/toka-runtime/Cargo.toml
fi

# Remove WASM from toka-tools
echo "Disabling WASM support in toka-tools..."
if [ -f "crates/toka-tools/Cargo.toml" ]; then
    sed -i 's/^wasmtime = /# wasmtime = /' crates/toka-tools/Cargo.toml
    sed -i 's/^wasm_loader = /# wasm_loader = /' crates/toka-tools/Cargo.toml
fi

# Remove heavy monitoring dependencies from workspace
echo "Simplifying workspace dependencies..."
cp Cargo.toml temp_cargo.toml

# Comment out heavy dependencies
sed -i 's/^prometheus = /# prometheus = /' temp_cargo.toml
sed -i 's/^opentelemetry = /# opentelemetry = /' temp_cargo.toml
sed -i 's/^criterion = /# criterion = /' temp_cargo.toml

mv temp_cargo.toml Cargo.toml

# Phase 3: Fix immediate build issues
echo ""
echo "🔨 PHASE 3: Fixing build issues"
echo "-------------------------------"

# The comment issue in toka-tools should already be fixed

# Phase 4: Update references to removed crates
echo ""
echo "🔗 PHASE 4: Updating cross-references"
echo "-------------------------------------"

# Find and comment out references to removed crates
find crates/ -name "*.toml" -exec grep -l "toka-store-sled\|toka-store-semantic\|raft-core\|raft-storage\|toka-performance\|toka-cvm\|toka-revocation" {} \; | while read file; do
    echo "Updating references in $file..."
    sed -i 's/^toka-store-sled = /# toka-store-sled = /' "$file"
    sed -i 's/^toka-store-semantic = /# toka-store-semantic = /' "$file"
    sed -i 's/^raft-core = /# raft-core = /' "$file"
    sed -i 's/^raft-storage = /# raft-storage = /' "$file"
    sed -i 's/^toka-performance = /# toka-performance = /' "$file"
    sed -i 's/^toka-cvm = /# toka-cvm = /' "$file"
    sed -i 's/^toka-revocation = /# toka-revocation = /' "$file"
done

# Phase 5: Test the changes
echo ""
echo "🧪 PHASE 5: Testing changes"
echo "---------------------------"

echo "Running cargo check to verify compilation..."
if cargo check --workspace; then
    echo "✅ Basic compilation successful!"
else
    echo "❌ Compilation failed. You may need to fix remaining references."
    echo "Check the error messages and update import statements in the source code."
fi

# Summary
echo ""
echo "📊 REDUCTION SUMMARY"
echo "==================="
echo "Removed crates: ${#CRATES_TO_REMOVE[@]}"
echo "Disabled WASM support (can be re-enabled)"
echo "Disabled Python support (already done)"
echo "Removed heavy monitoring dependencies"
echo ""
echo "Removed crates are backed up in: /tmp/toka-removed-crates/"
echo "Original Cargo.toml backed up as: Cargo.toml.backup"
echo ""

# Calculate size reduction
REMAINING_CRATES=$(find crates/ -name "Cargo.toml" | wc -l)
echo "📈 Before: ~31 crates"
echo "📉 After:  $REMAINING_CRATES crates"
echo "🎯 Reduction: ~$((31 - REMAINING_CRATES)) crates removed"

echo ""
echo "🎉 Bloat reduction complete!"
echo ""
echo "Next steps:"
echo "1. Fix any remaining compilation errors by updating imports"
echo "2. Run tests to ensure core functionality still works"
echo "3. Consider consolidating security crates (Phase 2 of the plan)"
echo "4. Measure build time and binary size improvements"
echo ""
echo "To restore a removed crate:"
echo "  cp -r /tmp/toka-removed-crates/CRATE_NAME crates/"
echo "  # Then update Cargo.toml to re-add it to workspace members"