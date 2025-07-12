#!/bin/bash
# Date Insertion Script for Toka Workspace
# Replaces {{today}} and {{commit_date}} placeholders with canonical dates

set -euo pipefail

# Get canonical current date
TODAY=$(date -u +%Y-%m-%d)
COMMIT_DATE=$(git log -1 --format=%cd --date=format:%Y-%m-%d 2>/dev/null || echo "$TODAY")

echo "Date Insertion Script - $TODAY"
echo "Current date: $TODAY"
echo "Commit date: $COMMIT_DATE"

# Function to process a single file
process_file() {
    local file="$1"
    local changes=0
    
    if [[ ! -f "$file" ]]; then
        echo "⚠️  File not found: $file"
        return
    fi
    
    echo "Processing: $file"
    
    # Create backup
    cp "$file" "${file}.bak"
    
    # Replace {{today}} with current date
    if grep -q "{{today}}" "$file"; then
        sed -i "s/{{today}}/$TODAY/g" "$file"
        changes=$((changes + 1))
        echo "  ✅ Replaced {{today}} with $TODAY"
    fi
    
    # Replace {{commit_date}} with commit date
    if grep -q "{{commit_date}}" "$file"; then
        sed -i "s/{{commit_date}}/$COMMIT_DATE/g" "$file"
        changes=$((changes + 1))
        echo "  ✅ Replaced {{commit_date}} with $COMMIT_DATE"
    fi
    
    # If no changes, remove backup
    if [[ $changes -eq 0 ]]; then
        rm "${file}.bak"
        echo "  ℹ️  No date placeholders found"
    else
        echo "  📝 Applied $changes date replacements"
    fi
}

# Function to process all files in a directory
process_directory() {
    local dir="$1"
    local pattern="$2"
    
    if [[ ! -d "$dir" ]]; then
        echo "⚠️  Directory not found: $dir"
        return
    fi
    
    echo "=== Processing directory: $dir ==="
    
    find "$dir" -name "$pattern" -type f | while read -r file; do
        process_file "$file"
    done
}

# Handle command line arguments
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <file|directory> [--all]"
    echo ""
    echo "Options:"
    echo "  file                Process a single file"
    echo "  directory           Process all markdown files in directory"
    echo "  --all               Process all common locations"
    echo ""
    echo "Examples:"
    echo "  $0 docs/proposals/2025-01-27_example.md"
    echo "  $0 docs/proposals/"
    echo "  $0 --all"
    exit 1
fi

if [[ "$1" == "--all" ]]; then
    echo "=== Processing All Common Locations ==="
    
    # Process documentation
    process_directory "docs" "*.md"
    
    # Process proposals
    process_directory "docs/proposals" "*.md"
    
    # Process agent specifications
    process_directory "agents-specs" "*.yaml"
    
    # Process reports
    process_directory "docs/reports" "*.md"
    
    # Process research
    process_directory "docs/research" "*.md"
    
    # Process crate documentation
    find crates -name "*.md" -type f | while read -r file; do
        process_file "$file"
    done
    
    # Process root-level files
    for file in README.md CHANGELOG.md QUICKSTART.md; do
        if [[ -f "$file" ]]; then
            process_file "$file"
        fi
    done
    
elif [[ -f "$1" ]]; then
    # Process single file
    process_file "$1"
    
elif [[ -d "$1" ]]; then
    # Process directory
    process_directory "$1" "*.md"
    
else
    echo "❌ Error: '$1' is not a valid file or directory"
    exit 1
fi

echo ""
echo "=== Date Insertion Summary ==="
echo "✅ Processed date placeholders"
echo "✅ Current date: $TODAY"
echo "✅ Commit date: $COMMIT_DATE"
echo ""
echo "Placeholders replaced:"
echo "  - {{today}} → $TODAY"
echo "  - {{commit_date}} → $COMMIT_DATE"
echo ""
echo "Note: Backup files (.bak) created for modified files"
echo "Review changes and remove backups when satisfied" 