#!/usr/bin/env python3
"""
Date Validation Script for Toka Workspace
Validates all dates in the repository against canonical sources
"""

import os
import re
import sys
import subprocess
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Tuple, Optional
import argparse


class DateValidator:
    """Validates dates in files against canonical sources"""
    
    def __init__(self, max_future_days: int = 0):
        self.max_future_days = max_future_days
        self.today = datetime.now().date()
        self.commit_date = self._get_commit_date()
        self.date_pattern = re.compile(r'\b(\d{4})-(\d{2})-(\d{2})\b')
        self.exempt_pattern = re.compile(r'<!-- DATE:EXEMPT.*?-->', re.IGNORECASE)
        self.violations = []
        
    def _get_commit_date(self) -> datetime.date:
        """Get the date of the last commit"""
        try:
            result = subprocess.run(
                ['git', 'log', '-1', '--format=%cd', '--date=format:%Y-%m-%d'],
                capture_output=True,
                text=True,
                check=True
            )
            return datetime.strptime(result.stdout.strip(), '%Y-%m-%d').date()
        except (subprocess.CalledProcessError, ValueError):
            return self.today
    
    def _is_date_exempt(self, content: str, match_start: int) -> bool:
        """Check if a date is exempt from validation"""
        # Look for DATE:EXEMPT comment in the preceding lines
        lines_before = content[:match_start].split('\n')
        
        # Check the last few lines for exemption comment
        for line in lines_before[-3:]:
            if self.exempt_pattern.search(line):
                return True
        
        return False
    
    def _parse_date(self, date_str: str) -> Optional[datetime.date]:
        """Parse a date string"""
        try:
            return datetime.strptime(date_str, '%Y-%m-%d').date()
        except ValueError:
            return None
    
    def _get_line_number(self, content: str, position: int) -> int:
        """Get line number for a position in the content"""
        return content[:position].count('\n') + 1
    
    def validate_file(self, file_path: Path) -> List[dict]:
        """Validate dates in a single file"""
        violations = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except (UnicodeDecodeError, IOError) as e:
            print(f"⚠️  Could not read {file_path}: {e}")
            return violations
        
        for match in self.date_pattern.finditer(content):
            date_str = match.group(0)
            date_obj = self._parse_date(date_str)
            
            if not date_obj:
                continue
            
            # Check if date is exempt
            if self._is_date_exempt(content, match.start()):
                continue
            
            # Check if date is in the future
            days_in_future = (date_obj - self.today).days
            
            if days_in_future > self.max_future_days:
                violations.append({
                    'file': file_path,
                    'line': self._get_line_number(content, match.start()),
                    'date': date_str,
                    'parsed_date': date_obj,
                    'days_in_future': days_in_future,
                    'violation_type': 'future_date'
                })
            
            # Check for obvious hallucinated dates (common LLM patterns)
            hallucinated_dates = [
                '2025-01-27', '2025-01-28', '2025-01-04', '2025-01-01',
                '2024-12-31', '2025-01-08', '2025-01-10', '2025-01-15',
                '2025-02-01', '2025-03-01'
            ]
            
            if date_str in hallucinated_dates and date_obj != self.today:
                violations.append({
                    'file': file_path,
                    'line': self._get_line_number(content, match.start()),
                    'date': date_str,
                    'parsed_date': date_obj,
                    'days_in_future': days_in_future,
                    'violation_type': 'hallucinated_date'
                })
        
        return violations
    
    def validate_directory(self, directory: Path, patterns: List[str]) -> List[dict]:
        """Validate dates in all files matching patterns in a directory"""
        violations = []
        
        for pattern in patterns:
            for file_path in directory.glob(pattern):
                if file_path.is_file():
                    file_violations = self.validate_file(file_path)
                    violations.extend(file_violations)
        
        return violations
    
    def validate_workspace(self) -> List[dict]:
        """Validate dates in the entire workspace"""
        violations = []
        
        # Define file patterns to check
        patterns = [
            ('docs', ['**/*.md']),
            ('agents-specs', ['**/*.yaml']),
            ('crates', ['**/*.md', '**/*.rs']),
            ('.', ['*.md', '*.yaml', '*.toml'])
        ]
        
        for directory, file_patterns in patterns:
            dir_path = Path(directory)
            if dir_path.exists():
                print(f"Validating {directory}/...")
                dir_violations = self.validate_directory(dir_path, file_patterns)
                violations.extend(dir_violations)
        
        return violations
    
    def fix_violations(self, violations: List[dict], dry_run: bool = True) -> int:
        """Fix date violations by replacing with current date"""
        fixes_applied = 0
        
        # Group violations by file
        files_to_fix = {}
        for violation in violations:
            file_path = violation['file']
            if file_path not in files_to_fix:
                files_to_fix[file_path] = []
            files_to_fix[file_path].append(violation)
        
        for file_path, file_violations in files_to_fix.items():
            if dry_run:
                print(f"Would fix {len(file_violations)} violations in {file_path}")
                continue
            
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Replace dates (in reverse order to maintain positions)
                for violation in sorted(file_violations, key=lambda x: x['line'], reverse=True):
                    old_date = violation['date']
                    new_date = self.today.strftime('%Y-%m-%d')
                    content = content.replace(old_date, new_date, 1)
                
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                
                fixes_applied += len(file_violations)
                print(f"✅ Fixed {len(file_violations)} violations in {file_path}")
                
            except (IOError, UnicodeDecodeError) as e:
                print(f"❌ Could not fix {file_path}: {e}")
        
        return fixes_applied
    
    def print_report(self, violations: List[dict]):
        """Print a detailed validation report"""
        if not violations:
            print("✅ No date violations found!")
            return
        
        print(f"❌ Found {len(violations)} date violations:")
        print()
        
        # Group by violation type
        by_type = {}
        for violation in violations:
            vtype = violation['violation_type']
            if vtype not in by_type:
                by_type[vtype] = []
            by_type[vtype].append(violation)
        
        for vtype, type_violations in by_type.items():
            print(f"=== {vtype.replace('_', ' ').title()} ({len(type_violations)} violations) ===")
            
            for violation in type_violations:
                file_path = violation['file']
                line = violation['line']
                date = violation['date']
                days_future = violation['days_in_future']
                
                print(f"  {file_path}:{line} - {date}", end="")
                if days_future > 0:
                    print(f" ({days_future} days in future)")
                else:
                    print()
            
            print()
        
        print(f"Current date: {self.today}")
        print(f"Commit date: {self.commit_date}")
        print()
        print("To fix violations automatically, run with --fix")
        print("To exempt historical dates, add: <!-- DATE:EXEMPT source=\"reference\" -->")


def main():
    parser = argparse.ArgumentParser(description='Validate dates in Toka workspace')
    parser.add_argument('--fix', action='store_true', help='Fix violations automatically')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be fixed')
    parser.add_argument('--max-future-days', type=int, default=0, 
                       help='Maximum allowed days in future (default: 0)')
    parser.add_argument('file', nargs='?', help='Validate specific file')
    
    args = parser.parse_args()
    
    validator = DateValidator(max_future_days=args.max_future_days)
    
    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ File not found: {file_path}")
            sys.exit(1)
        
        violations = validator.validate_file(file_path)
    else:
        violations = validator.validate_workspace()
    
    validator.print_report(violations)
    
    if violations:
        if args.fix:
            fixes = validator.fix_violations(violations, dry_run=args.dry_run)
            if not args.dry_run:
                print(f"✅ Applied {fixes} fixes")
        
        sys.exit(1)  # Exit with error code if violations found
    
    sys.exit(0)


if __name__ == '__main__':
    main() 