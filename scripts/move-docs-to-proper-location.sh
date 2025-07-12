#!/bin/bash

# Documentation Organization Script
# Generated: 2025-07-12 (UTC)
# Purpose: Move documentation files from root to proper docs directory structure

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_NAME="$(basename "$0")"
WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKUP_DIR="${WORKSPACE_ROOT}/backup/docs-move-$(date +%Y%m%d_%H%M%S)"
DRY_RUN=false

# Logging
print_status() {
    echo -e "${1}${2}${NC}"
}

print_section() {
    echo
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to create backup
create_backup() {
    local file_path="$1"
    local backup_path="${BACKUP_DIR}/${file_path#${WORKSPACE_ROOT}/}"
    
    mkdir -p "$(dirname "$backup_path")"
    cp "$file_path" "$backup_path"
    print_status $YELLOW "  Backed up: $file_path"
}

# Function to move file with backup
move_file_with_backup() {
    local source="$1"
    local destination="$2"
    
    if [[ ! -f "$source" ]]; then
        print_status $YELLOW "  File not found: $source"
        return 1
    fi
    
    # Create backup
    create_backup "$source"
    
    # Create destination directory
    mkdir -p "$(dirname "$destination")"
    
    # Move file
    if [[ "$DRY_RUN" == "true" ]]; then
        print_status $YELLOW "  [DRY RUN] Would move: $source -> $destination"
    else
        mv "$source" "$destination"
        print_status $GREEN "  Moved: $source -> $destination"
    fi
}

# Function to move documentation files
move_docs() {
    local moved_count=0
    
    # Documentation audit and analysis files -> docs/reports/
    move_file_with_backup "DOCUMENTATION_AUDIT_AND_VERSIONING_ANALYSIS.md" "docs/reports/DOCUMENTATION_AUDIT_AND_VERSIONING_ANALYSIS.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "DOCUMENTATION_LINKAGE_IMPROVEMENTS.md" "docs/reports/DOCUMENTATION_LINKAGE_IMPROVEMENTS.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "DOCUMENTATION_MAINTENANCE_SYSTEM_PROPOSAL.md" "docs/reports/DOCUMENTATION_MAINTENANCE_SYSTEM_PROPOSAL.md" && moved_count=$((moved_count + 1))
    
    # Implementation summaries -> docs/reports/
    move_file_with_backup "PHASE_1_IMPLEMENTATION_SUMMARY.md" "docs/reports/PHASE_1_IMPLEMENTATION_SUMMARY.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "PHASE_2_IMPLEMENTATION_SUMMARY.md" "docs/reports/PHASE_2_IMPLEMENTATION_SUMMARY.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "PHASE_2_SURGICAL_REDUCTION_PLAN.md" "docs/reports/PHASE_2_SURGICAL_REDUCTION_PLAN.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "PHASE_3_MERGE_AND_NEXT_STEPS_SUMMARY.md" "docs/reports/PHASE_3_MERGE_AND_NEXT_STEPS_SUMMARY.md" && moved_count=$((moved_count + 1))
    
    # Architecture analysis -> docs/architecture/
    move_file_with_backup "TOKA_ARCHITECTURE_ANALYSIS_REPORT.md" "docs/architecture/TOKA_ARCHITECTURE_ANALYSIS_REPORT.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "TOKA_ARCHITECTURE_CLEANUP_ROADMAP.md" "docs/architecture/TOKA_ARCHITECTURE_CLEANUP_ROADMAP.md" && moved_count=$((moved_count + 1))
    
    # Executive and implementation guides -> docs/guides/
    move_file_with_backup "EXECUTIVE_SUMMARY.md" "docs/guides/EXECUTIVE_SUMMARY.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "IMPLEMENTATION_GUIDE.md" "docs/guides/IMPLEMENTATION_GUIDE.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "SETUP_GUIDE.md" "docs/guides/SETUP_GUIDE.md" && moved_count=$((moved_count + 1))
    
    # Tool and metadata files -> docs/tools/
    move_file_with_backup "TOOL_METADATA_CATALOGUE.md" "docs/tools/TOOL_METADATA_CATALOGUE.md" && moved_count=$((moved_count + 1))
    move_file_with_backup "TOOLS_CONSOLIDATION.md" "docs/tools/TOOLS_CONSOLIDATION.md" && moved_count=$((moved_count + 1))
    
    # Research and analysis -> docs/research/
    move_file_with_backup "merge-analysis-collaborative-ecosystem.md" "docs/research/merge-analysis-collaborative-ecosystem.md" && moved_count=$((moved_count + 1))
    
    echo $moved_count
}

# Main execution
main() {
    print_section "Documentation Organization"
    echo "Workspace: $WORKSPACE_ROOT"
    echo "Mode: $([ "$DRY_RUN" == "true" ] && echo "DRY RUN" || echo "LIVE EXECUTION")"
    echo "Created backup directory: $BACKUP_DIR"
    
    # Create backup directory
    mkdir -p "$BACKUP_DIR"
    
    print_section "Moving Documentation Files"
    
    local moved_count=$(move_docs)
    
    print_section "Summary"
    print_status $GREEN "✅ Moved $moved_count documentation files to proper locations"
    print_status $GREEN "✅ All files backed up to: $BACKUP_DIR"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo
        echo "To execute the moves, run without --dry-run"
    else
        echo
        echo "Next steps:"
        echo "1. Review the moved files in docs/ directory"
        echo "2. Update any internal links that reference the old locations"
        echo "3. Commit the reorganization"
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--dry-run]"
            echo "  --dry-run    Show what would be moved without actually moving"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main "$@" 