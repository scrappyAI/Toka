#!/bin/bash
# Date insertion script for Toka workspace
# Replaces {{today}} and {{commit_date}} placeholders with actual dates

set -euo pipefail

# Get current UTC date
TODAY=$(date -u +%Y-%m-%d)

# Get commit date
COMMIT_DATE=$(git log -1 --format=%cd --date=format:%Y-%m-%d 2>/dev/null || echo "$TODAY")

# Function to process a file
process_file() {
    local file="$1"
    
    if [[ ! -f "$file" ]]; then
        echo "Error: File '$file' not found" >&2
        return 1
    fi
    
    # Create backup
    cp "$file" "${file}.bak"
    
    # Replace placeholders
    sed -i "s/{{today}}/${TODAY}/g" "$file"
    sed -i "s/{{commit_date}}/${COMMIT_DATE}/g" "$file"
    
    echo "Processed: $file (today: $TODAY, commit: $COMMIT_DATE)"
}

# Main function
main() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: $0 <file1> [file2] [file3] ..."
        echo "       $0 --all  # Process all template files"
        echo ""
        echo "Replaces {{today}} with current UTC date and {{commit_date}} with commit date"
        exit 1
    fi
    
    if [[ "$1" == "--all" ]]; then
        # Find all files with date placeholders
        echo "Searching for files with date placeholders..."
        
        # Use find to locate files with placeholders
        while IFS= read -r file; do
            if [[ -n "$file" ]]; then
                process_file "$file"
            fi
        done < <(grep -r -l "{{today}}\|{{commit_date}}" . --include="*.md" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.yaml" 2>/dev/null || true)
        
        echo "Date placeholder replacement complete."
    else
        # Process specific files
        for file in "$@"; do
            process_file "$file"
        done
    fi
}

main "$@"