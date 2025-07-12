#!/bin/bash
# Deterministic Dating Fix Script
# Generated: 2025-07-12 (UTC)
# Purpose: Replace LLM hallucinated dates with canonical current date

set -euo pipefail

# Get canonical current date
TODAY=$(date -u +%Y-%m-%d)
echo "Canonical date: $TODAY"

# Define hallucinated date patterns to fix
HALLUCINATED_DATES=(
    "2025-01-27"
    "2025-01-28" 
    "2025-01-04"
    "2025-01-01"
    "2024-12-31"
    "2025-01-08"
    "2025-01-10"
)

# Function to fix dates in a file
fix_dates_in_file() {
    local file="$1"
    local changed=false
    
    echo "Checking: $file"
    
    for bad_date in "${HALLUCINATED_DATES[@]}"; do
        if grep -q "$bad_date" "$file" 2>/dev/null; then
            echo "  Fixing $bad_date -> $TODAY in $file"
            sed -i "s/$bad_date/$TODAY/g" "$file"
            changed=true
        fi
    done
    
    if [[ "$changed" == "true" ]]; then
        echo "  ✅ Fixed dates in $file"
    fi
}

# Core workspace crates to prioritize
CORE_CRATES=(
    "crates/toka-types"
    "crates/toka-kernel" 
    "crates/toka-bus-core"
    "crates/toka-runtime"
    "crates/toka-auth"
    "crates/toka-store-core"
    "crates/toka-orchestration"
    "crates/toka-agent-runtime"
)

echo "=== Fixing Core Workspace Crates ==="
for crate_dir in "${CORE_CRATES[@]}"; do
    if [[ -d "$crate_dir" ]]; then
        echo "Processing core crate: $crate_dir"
        
        # Fix Cargo.toml files
        find "$crate_dir" -name "Cargo.toml" -type f | while read -r file; do
            fix_dates_in_file "$file"
        done
        
        # Fix Rust source files
        find "$crate_dir" -name "*.rs" -type f | while read -r file; do
            fix_dates_in_file "$file"
        done
        
        # Fix documentation
        find "$crate_dir" -name "*.md" -type f | while read -r file; do
            fix_dates_in_file "$file"
        done
    else
        echo "⚠️  Core crate not found: $crate_dir"
    fi
done

# Fix agent specifications
echo "=== Fixing Agent Specifications ==="
find "agents-specs" -name "*.yaml" -type f | while read -r file; do
    fix_dates_in_file "$file"
done

# Fix documentation reports with specific date hallucinations
echo "=== Fixing Documentation Reports ==="
DOCS_WITH_DATES=(
    "docs/reports/github-cicd-fixes-report.md"
    "docs/research/toka_agent_implementation_research_and_proposal.md" 
    "docs/reports/memory-context-management-report.md"
    "docs/research/20250127_toka_crates_deep_dive_analysis.md"
    "docs/research/20250127_toka_production_readiness_report.md"
    "docs/reports/PYTHON_TOOLS_INTEGRATION_UNIFIED_APPROACH.md"
    "docs/reports/IMPLEMENTATION_ROADMAP.md"
    "docs/guides/TOKA_TESTING_SETUP_GUIDE.md"
    "docs/guides/AGENT_RUNTIME_IMPLEMENTATION_GUIDE.md"
    "crates/security/SECURITY_FIXES_SUMMARY.md"
    "crates/security/CRITICAL_ISSUES.md"
)

for doc_file in "${DOCS_WITH_DATES[@]}"; do
    if [[ -f "$doc_file" ]]; then
        fix_dates_in_file "$doc_file"
    fi
done

echo "=== Date Fix Summary ==="
echo "✅ Fixed hallucinated dates in core workspace crates"
echo "✅ Replaced with canonical date: $TODAY"
echo "✅ Core crates processed: ${#CORE_CRATES[@]}"
echo ""
echo "Next steps:"
echo "1. Review changes with: git diff"
echo "2. Test core crate builds: cargo check --workspace"
echo "3. Commit deterministic date fixes"