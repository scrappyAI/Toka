#!/usr/bin/env python3
"""
Date validation script for Toka workspace
Enforces date accuracy according to the date enforcement rule.
"""

import os
import re
import sys
from datetime import datetime, timedelta
import subprocess
from pathlib import Path
import argparse

def get_current_date():
    """Get current UTC date"""
    return datetime.now().strftime('%Y-%m-%d')

def get_commit_date():
    """Get the current commit date"""
    try:
        result = subprocess.run(
            ['git', 'log', '-1', '--format=%cd', '--date=format:%Y-%m-%d'],
            capture_output=True, text=True, check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return get_current_date()

def find_date_patterns(file_path):
    """Find all date patterns in a file"""
    date_pattern = r'\b(\d{4}-\d{2}-\d{2})\b'
    exempt_pattern = r'<!--\s*DATE:EXEMPT\s+source="[^"]*"\s*-->'
    
    dates_found = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
            
        for i, line in enumerate(lines, 1):
            # Check if previous line has exemption
            is_exempt = False
            if i > 1:
                prev_line = lines[i-2]
                if re.search(exempt_pattern, prev_line):
                    is_exempt = True
            
            # Find all dates in current line
            for match in re.finditer(date_pattern, line):
                date_str = match.group(1)
                dates_found.append({
                    'date': date_str,
                    'line': i,
                    'line_content': line.strip(),
                    'is_exempt': is_exempt,
                    'file': file_path
                })
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    
    return dates_found

def validate_date(date_str, current_date, commit_date):
    """Validate a date string against current date and commit date"""
    try:
        date_obj = datetime.strptime(date_str, '%Y-%m-%d')
        current_obj = datetime.strptime(current_date, '%Y-%m-%d')
        
        # Allow dates up to 30 days in the future for planning
        future_threshold = current_obj + timedelta(days=30)
        
        # Check if date is too far in the future
        if date_obj > future_threshold:
            return False, f"Date too far in the future (>{future_threshold.strftime('%Y-%m-%d')})"
        
        # Check if date matches current date (acceptable)
        if date_str == current_date:
            return True, "Current date"
        
        # Check if date matches commit date (acceptable)
        if date_str == commit_date:
            return True, "Commit date"
        
        # Check if date is recent (within 7 days)
        if abs((date_obj - current_obj).days) <= 7:
            return True, "Recent date"
        
        # Date is old but not necessarily wrong
        return True, f"Old date ({date_str})"
        
    except ValueError:
        return False, "Invalid date format"

def main():
    parser = argparse.ArgumentParser(description='Validate dates in workspace files')
    parser.add_argument('--fix', action='store_true', help='Fix obvious date issues')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('files', nargs='*', help='Specific files to check')
    
    args = parser.parse_args()
    
    current_date = get_current_date()
    commit_date = get_commit_date()
    
    print(f"Current date: {current_date}")
    print(f"Commit date: {commit_date}")
    print()
    
    # File patterns to check
    if args.files:
        files_to_check = args.files
    else:
        files_to_check = []
        patterns = ['**/*.md', '**/*.rs', '**/*.py', '**/*.toml', '**/*.yml', '**/*.yaml']
        
        for pattern in patterns:
            for file_path in Path('.').glob(pattern):
                if file_path.is_file() and not any(skip in str(file_path) for skip in ['.git', 'target', 'node_modules']):
                    files_to_check.append(str(file_path))
    
    errors = []
    warnings = []
    
    for file_path in files_to_check:
        dates = find_date_patterns(file_path)
        
        for date_info in dates:
            if date_info['is_exempt']:
                if args.verbose:
                    print(f"EXEMPT: {date_info['file']}:{date_info['line']} - {date_info['date']}")
                continue
            
            is_valid, reason = validate_date(date_info['date'], current_date, commit_date)
            
            if not is_valid:
                errors.append(f"ERROR: {date_info['file']}:{date_info['line']} - {date_info['date']} - {reason}")
            elif args.verbose and "Old date" in reason:
                warnings.append(f"WARNING: {date_info['file']}:{date_info['line']} - {date_info['date']} - {reason}")
    
    # Print results
    if errors:
        print("ERRORS FOUND:")
        for error in errors:
            print(f"  {error}")
        print()
    
    if warnings and args.verbose:
        print("WARNINGS:")
        for warning in warnings:
            print(f"  {warning}")
        print()
    
    if not errors and not warnings:
        print("All dates are valid!")
    elif not errors:
        print("No critical date errors found.")
    
    # Exit with error code if there are errors
    return 1 if errors else 0

if __name__ == '__main__':
    sys.exit(main())