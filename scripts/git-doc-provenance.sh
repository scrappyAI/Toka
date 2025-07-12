#!/bin/bash
# Git-based Documentation Provenance and AI Code Tracking System
# Tracks AI-generated code, validates documentation links, and maintains audit trails

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROVENANCE_DIR="$PROJECT_ROOT/.git/provenance"
AI_METADATA_FILE="$PROVENANCE_DIR/ai-generations.jsonl"
DOC_LINKAGE_FILE="$PROVENANCE_DIR/doc-linkage.json"
AUDIT_LOG="$PROVENANCE_DIR/audit.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date -u '+%Y-%m-%d %H:%M:%S UTC')
    
    case $level in
        "INFO")  echo -e "${GREEN}[INFO]${NC} $message" ;;
        "WARN")  echo -e "${YELLOW}[WARN]${NC} $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $message" ;;
        "DEBUG") echo -e "${BLUE}[DEBUG]${NC} $message" ;;
    esac
    
    echo "[$timestamp] [$level] $message" >> "$AUDIT_LOG"
}

# Initialize provenance tracking
init_provenance() {
    log "INFO" "Initializing Git-based provenance tracking..."
    
    mkdir -p "$PROVENANCE_DIR"
    
    # Initialize AI generations log
    if [[ ! -f "$AI_METADATA_FILE" ]]; then
        echo '{"initialized": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'", "version": "1.0.0"}' > "$AI_METADATA_FILE"
    fi
    
    # Initialize documentation linkage
    if [[ ! -f "$DOC_LINKAGE_FILE" ]]; then
        echo '{"links": {}, "last_updated": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > "$DOC_LINKAGE_FILE"
    fi
    
    log "INFO" "Provenance tracking initialized"
}

# Track AI-generated code with metadata
track_ai_generation() {
    local file_path="$1"
    local ai_model="${2:-unknown}"
    local generation_type="${3:-code}"
    local confidence="${4:-unknown}"
    local human_review="${5:-false}"
    
    log "INFO" "Tracking AI generation: $file_path"
    
    local commit_hash=$(git rev-parse HEAD 2>/dev/null || echo "uncommitted")
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local file_hash=$(sha256sum "$file_path" 2>/dev/null | cut -d' ' -f1 || echo "unknown")
    
    # Create AI generation metadata
    local ai_metadata=$(cat <<EOF
{
    "timestamp": "$timestamp",
    "commit_hash": "$commit_hash",
    "file_path": "$file_path",
    "file_hash": "$file_hash",
    "ai_model": "$ai_model",
    "generation_type": "$generation_type",
    "confidence": "$confidence",
    "human_review": $human_review,
    "git_author": "$(git config user.name 2>/dev/null || echo 'unknown')",
    "git_email": "$(git config user.email 2>/dev/null || echo 'unknown')",
    "provenance_id": "$(uuidgen 2>/dev/null || echo "$(date +%s)-$$")"
}
EOF
    )
    
    # Append to AI generations log
    echo "$ai_metadata" >> "$AI_METADATA_FILE"
    
    # Add special commit trailer for AI-generated content
    git config --local trailer.AI-Generated.key "AI-Generated"
    git config --local trailer.AI-Generated.command "echo 'true'"
    git config --local trailer.AI-Model.key "AI-Model"
    git config --local trailer.AI-Model.command "echo '$ai_model'"
    git config --local trailer.AI-Confidence.key "AI-Confidence"
    git config --local trailer.AI-Confidence.command "echo '$confidence'"
    
    log "INFO" "AI generation tracked with provenance ID: $(echo "$ai_metadata" | jq -r '.provenance_id')"
}

# Validate documentation cross-references
validate_doc_links() {
    log "INFO" "Validating documentation cross-references..."
    
    local broken_links=()
    local valid_links=()
    
    # Find all markdown files
    while IFS= read -r -d '' file; do
        log "DEBUG" "Checking links in: $file"
        
        # Extract markdown links [text](path) and [text](path#anchor)
        while IFS= read -r link; do
            if [[ -n "$link" ]]; then
                local link_path=$(echo "$link" | sed 's/.*](\([^)]*\)).*/\1/' | sed 's/#.*//')
                local full_path=""
                
                # Handle relative paths
                if [[ "$link_path" =~ ^\.\.?/ ]]; then
                    full_path="$(dirname "$file")/$link_path"
                    full_path="$(cd "$(dirname "$full_path")" && pwd)/$(basename "$full_path")" 2>/dev/null || echo "$full_path"
                elif [[ "$link_path" =~ ^/ ]]; then
                    full_path="$PROJECT_ROOT$link_path"
                elif [[ ! "$link_path" =~ ^https?:// ]]; then
                    full_path="$(dirname "$file")/$link_path"
                fi
                
                # Check if file exists
                if [[ -n "$full_path" && ! -f "$full_path" && ! "$link_path" =~ ^https?:// ]]; then
                    broken_links+=("$file: $link_path")
                    log "WARN" "Broken link found: $file -> $link_path"
                else
                    valid_links+=("$file: $link_path")
                fi
            fi
        done < <(grep -oE '\[([^\]]+)\]\(([^)]+)\)' "$file" 2>/dev/null || true)
        
    done < <(find "$PROJECT_ROOT/docs" -name "*.md" -print0 2>/dev/null)
    
    # Update documentation linkage file
    local linkage_data=$(cat <<EOF
{
    "last_updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "validation_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'uncommitted')",
    "total_links": $((${#valid_links[@]} + ${#broken_links[@]})),
    "valid_links": ${#valid_links[@]},
    "broken_links": ${#broken_links[@]},
    "broken_link_details": $(printf '%s\n' "${broken_links[@]}" | jq -R . | jq -s .)
}
EOF
    )
    
    echo "$linkage_data" > "$DOC_LINKAGE_FILE"
    
    if [[ ${#broken_links[@]} -gt 0 ]]; then
        log "ERROR" "Found ${#broken_links[@]} broken documentation links"
        return 1
    else
        log "INFO" "All ${#valid_links[@]} documentation links are valid"
        return 0
    fi
}

# Generate documentation audit report
generate_audit_report() {
    log "INFO" "Generating documentation audit report..."
    
    local report_file="$PROVENANCE_DIR/audit-report-$(date +%Y%m%d-%H%M%S).json"
    local commit_hash=$(git rev-parse HEAD 2>/dev/null || echo "uncommitted")
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    # Count AI-generated files
    local ai_count=$(grep -c "^{" "$AI_METADATA_FILE" 2>/dev/null || echo "0")
    
    # Get documentation statistics
    local total_docs=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f | wc -l)
    local total_size=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f -exec wc -c {} + | tail -1 | awk '{print $1}')
    
    # Get recent commits affecting documentation
    local recent_doc_commits=$(git log --since="1 week ago" --oneline --name-only -- docs/ | grep -E "^[a-f0-9]{7,}" | wc -l)
    
    # Create comprehensive audit report
    local audit_report=$(cat <<EOF
{
    "audit_timestamp": "$timestamp",
    "commit_hash": "$commit_hash",
    "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')",
    "documentation_stats": {
        "total_markdown_files": $total_docs,
        "total_size_bytes": $total_size,
        "recent_commits_1week": $recent_doc_commits
    },
    "ai_generation_stats": {
        "total_ai_generations": $ai_count,
        "ai_metadata_file": "$AI_METADATA_FILE"
    },
    "link_validation": $(cat "$DOC_LINKAGE_FILE" 2>/dev/null || echo '{}'),
    "git_stats": {
        "total_commits": $(git rev-list --count HEAD 2>/dev/null || echo "0"),
        "contributors": $(git shortlog -sn | wc -l),
        "last_commit_date": "$(git log -1 --format=%cd --date=iso 2>/dev/null || echo 'unknown')"
    }
}
EOF
    )
    
    echo "$audit_report" > "$report_file"
    
    log "INFO" "Audit report generated: $report_file"
    
    # Display summary
    echo -e "\n${BLUE}=== Documentation Audit Summary ===${NC}"
    echo "$audit_report" | jq -r '
        "Timestamp: " + .audit_timestamp,
        "Commit: " + .commit_hash[0:8],
        "Total Docs: " + (.documentation_stats.total_markdown_files | tostring),
        "AI Generations: " + (.ai_generation_stats.total_ai_generations | tostring),
        "Valid Links: " + (.link_validation.valid_links | tostring),
        "Broken Links: " + (.link_validation.broken_links | tostring)
    '
}

# Create Git commit with AI provenance
commit_with_provenance() {
    local commit_message="$1"
    local ai_model="${2:-}"
    local files_to_commit=("${@:3}")
    
    log "INFO" "Creating commit with AI provenance tracking..."
    
    # Add files to staging
    for file in "${files_to_commit[@]}"; do
        if [[ -f "$file" ]]; then
            git add "$file"
            
            # Track AI generation if model specified
            if [[ -n "$ai_model" ]]; then
                track_ai_generation "$file" "$ai_model" "code" "medium" "true"
            fi
        fi
    done
    
    # Create commit with AI metadata trailers
    if [[ -n "$ai_model" ]]; then
        git commit -m "$commit_message" \
            --trailer "AI-Generated: true" \
            --trailer "AI-Model: $ai_model" \
            --trailer "AI-Confidence: medium" \
            --trailer "Human-Review: true" \
            --trailer "Provenance-Tracked: true"
    else
        git commit -m "$commit_message"
    fi
    
    log "INFO" "Commit created with provenance tracking"
}

# Search AI-generated code by criteria
search_ai_code() {
    local search_type="${1:-all}"
    local search_value="${2:-}"
    
    log "INFO" "Searching AI-generated code: $search_type"
    
    case "$search_type" in
        "model")
            jq -r "select(.ai_model == \"$search_value\") | .file_path" "$AI_METADATA_FILE"
            ;;
        "date")
            jq -r "select(.timestamp | startswith(\"$search_value\")) | .file_path" "$AI_METADATA_FILE"
            ;;
        "commit")
            jq -r "select(.commit_hash | startswith(\"$search_value\")) | .file_path" "$AI_METADATA_FILE"
            ;;
        "type")
            jq -r "select(.generation_type == \"$search_value\") | .file_path" "$AI_METADATA_FILE"
            ;;
        "all")
            jq -r '.file_path' "$AI_METADATA_FILE"
            ;;
        *)
            log "ERROR" "Unknown search type: $search_type"
            return 1
            ;;
    esac
}

# Main command dispatcher
main() {
    case "${1:-help}" in
        "init")
            init_provenance
            ;;
        "track")
            if [[ $# -lt 2 ]]; then
                log "ERROR" "Usage: $0 track <file> [ai_model] [type] [confidence] [human_review]"
                exit 1
            fi
            track_ai_generation "$2" "${3:-unknown}" "${4:-code}" "${5:-unknown}" "${6:-false}"
            ;;
        "validate")
            validate_doc_links
            ;;
        "audit")
            generate_audit_report
            ;;
        "commit")
            if [[ $# -lt 2 ]]; then
                log "ERROR" "Usage: $0 commit <message> [ai_model] [files...]"
                exit 1
            fi
            commit_with_provenance "$2" "${3:-}" "${@:4}"
            ;;
        "search")
            search_ai_code "${2:-all}" "${3:-}"
            ;;
        "help")
            echo "Git Documentation Provenance and AI Code Tracking System"
            echo ""
            echo "Usage: $0 <command> [options]"
            echo ""
            echo "Commands:"
            echo "  init                           Initialize provenance tracking"
            echo "  track <file> [model] [type]    Track AI-generated file"
            echo "  validate                       Validate documentation links"
            echo "  audit                          Generate audit report"
            echo "  commit <msg> [model] [files]   Commit with AI provenance"
            echo "  search [type] [value]          Search AI-generated code"
            echo "  help                           Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 init"
            echo "  $0 track src/main.rs claude-3.5-sonnet code high true"
            echo "  $0 validate"
            echo "  $0 commit 'Add new feature' claude-3.5-sonnet src/main.rs"
            echo "  $0 search model claude-3.5-sonnet"
            ;;
        *)
            log "ERROR" "Unknown command: $1"
            main help
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 