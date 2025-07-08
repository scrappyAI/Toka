#!/usr/bin/env python3
"""
Date Fix Script

Systematically fixes all 2024 date references to 2025 across the codebase.
This script ensures deterministic, accurate dating and prevents date hallucination.
"""

import os
import re
import json
import argparse
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Tuple, Optional, Any


class DateFixer:
    """Fixes date references from 2024 to 2025 across the codebase."""
    
    def __init__(self, root_dir: str = "."):
        """Initialize the date fixer."""
        self.root_dir = Path(root_dir)
        self.changes = []
        self.errors = []
        
        # Date patterns to fix
        self.date_patterns = [
            # ISO date patterns
            (r'2024-(\d{2})-(\d{2})', r'2025-\1-\2'),
            (r'2024-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z', r'2025-\1-\2T\3:\4:\5Z'),
            # Hardcoded date patterns
            (r'2025-01-01', '2025-01-01'),
            (r'2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),
                    # Year references in comments
        (r'#.*2025', lambda m: str(m.group(0)).replace('2025', '2025')),
        (r'//.*2025', lambda m: str(m.group(0)).replace('2025', '2025')),
        (r'/\*.*2024', lambda m: str(m.group(0)).replace('2024', '2025')),
        ]
        
        # File patterns to process
        self.file_patterns = [
            '**/*.json',
            '**/*.md',
            '**/*.py',
            '**/*.rs',
            '**/*.toml',
            '**/*.yaml',
            '**/*.yml'
        ]
        
        # Exclude patterns
        self.exclude_patterns = [
            '**/node_modules/**',
            '**/.git/**',
            '**/target/**',
            '**/__pycache__/**',
            '**/dist/**',
            '**/build/**',
            '**/*.pyc',
            '**/*.log'
        ]
    
    def should_process_file(self, file_path: Path) -> bool:
        """Check if a file should be processed."""
        # Check exclude patterns
        for pattern in self.exclude_patterns:
            if file_path.match(pattern):
                return False
        
        # Check include patterns
        for pattern in self.file_patterns:
            if file_path.match(pattern):
                return True
        
        return False
    
    def fix_dates_in_content(self, content: str, file_path: str) -> Tuple[str, List[Dict]]:
        """Fix date references in content."""
        changes = []
        modified_content = content
        
        for pattern, replacement in self.date_patterns:
            if callable(replacement):
                # For lambda functions (comment patterns)
                matches = list(re.finditer(pattern, modified_content))
                for match in reversed(matches):  # Process in reverse to maintain positions
                    old_text = match.group(0)
                    new_text = replacement(match)
                    if old_text != new_text:
                        modified_content = (
                            modified_content[:match.start()] + 
                            str(new_text) + 
                            modified_content[match.end():]
                        )
                        changes.append({
                            "file": file_path,
                            "line": content[:match.start()].count('\n') + 1,
                            "old": old_text,
                            "new": new_text,
                            "type": "comment_date"
                        })
            else:
                # For string replacements
                matches = list(re.finditer(pattern, modified_content))
                for match in reversed(matches):  # Process in reverse to maintain positions
                    old_text = match.group(0)
                    new_text = re.sub(pattern, replacement, old_text)
                    if old_text != new_text:
                        modified_content = (
                            modified_content[:match.start()] + 
                            new_text + 
                            modified_content[match.end():]
                        )
                        changes.append({
                            "file": file_path,
                            "line": content[:match.start()].count('\n') + 1,
                            "old": old_text,
                            "new": new_text,
                            "type": "date_pattern"
                        })
        
        return modified_content, changes
    
    def process_file(self, file_path: Path) -> List[Dict]:
        """Process a single file for date fixes."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            modified_content, changes = self.fix_dates_in_content(content, str(file_path))
            
            if changes:
                # Write the modified content back
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(modified_content)
                
                print(f"✅ Fixed {len(changes)} date references in {file_path}")
                return changes
            else:
                print(f"ℹ️  No date fixes needed in {file_path}")
                return []
                
        except Exception as e:
            error_msg = f"Error processing {file_path}: {e}"
            print(f"❌ {error_msg}")
            self.errors.append(error_msg)
            return []
    
    def scan_directory(self, dry_run: bool = False) -> Dict[str, Any]:
        """Scan directory for files that need date fixes."""
        all_files = []
        for pattern in self.file_patterns:
            all_files.extend(self.root_dir.glob(pattern))
        
        # Remove duplicates and filter
        unique_files = list(set(all_files))
        files_to_process = [f for f in unique_files if self.should_process_file(f)]
        
        print(f"📁 Found {len(files_to_process)} files to process")
        
        if dry_run:
            print("🔍 DRY RUN - No changes will be made")
            return {
                "files_found": len(files_to_process),
                "files_to_process": [str(f) for f in files_to_process],
                "dry_run": True
            }
        
        total_changes = []
        processed_files = 0
        
        for file_path in files_to_process:
            changes = self.process_file(file_path)
            total_changes.extend(changes)
            if changes:
                processed_files += 1
        
        return {
            "files_found": len(files_to_process),
            "files_processed": processed_files,
            "total_changes": len(total_changes),
            "changes": total_changes,
            "errors": self.errors
        }
    
    def generate_report(self, results: Dict[str, Any]) -> str:
        """Generate a report of the date fixing operation."""
        report = []
        report.append("# Date Fix Report")
        report.append(f"Generated: {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}")
        report.append("")
        
        if results.get("dry_run"):
            report.append("## Dry Run Results")
            report.append(f"- Files found: {results['files_found']}")
            report.append(f"- Files to process: {len(results['files_to_process'])}")
            report.append("")
            report.append("### Files to be processed:")
            for file_path in results['files_to_process']:
                report.append(f"- {file_path}")
        else:
            report.append("## Fix Results")
            report.append(f"- Files found: {results['files_found']}")
            report.append(f"- Files processed: {results['files_processed']}")
            report.append(f"- Total changes: {results['total_changes']}")
            report.append("")
            
            if results['changes']:
                report.append("### Changes Made:")
                for change in results['changes']:
                    report.append(f"- {change['file']}:{change['line']} - {change['old']} → {change['new']}")
            
            if results['errors']:
                report.append("")
                report.append("### Errors:")
                for error in results['errors']:
                    report.append(f"- {error}")
        
        return "\n".join(report)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Fix date references from 2024 to 2025")
    parser.add_argument("--root", default=".", help="Root directory to scan")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be changed without making changes")
    parser.add_argument("--report", help="Save report to file")
    
    args = parser.parse_args()
    
    print("🔧 Date Fix Tool")
    print("=" * 50)
    
    fixer = DateFixer(args.root)
    results = fixer.scan_directory(dry_run=args.dry_run)
    
    # Generate and display report
    report = fixer.generate_report(results)
    print("\n" + "=" * 50)
    print(report)
    
    # Save report if requested
    if args.report:
        with open(args.report, 'w') as f:
            f.write(report)
        print(f"\n📄 Report saved to {args.report}")
    
    # Exit with error code if there were errors
    if results.get('errors'):
        print(f"\n⚠️  {len(results['errors'])} errors occurred")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main()) 