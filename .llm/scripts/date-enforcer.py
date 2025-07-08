#!/usr/bin/env python3
"""
Date Enforcement Tool

Enforces deterministic, accurate dating through schema contracts and prevents
date hallucination. Provides canonical sources for dates and validates all
date references in the codebase.
"""

import json
import os
import sys
import argparse
import sqlite3
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
import click
from dateutil import parser as date_parser
from git import Repo


class DateEnforcer:
    """Enforces deterministic, accurate dating through schema contracts."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize the date enforcer with configuration."""
        self.config = config
        self.db_path = Path(config.get('databaseFile', 'date_audit.db'))
        self.date_format = config.get('dateFormat', '%Y-%m-%d')
        self.time_format = config.get('timeFormat', '%Y-%m-%dT%H:%M:%SZ')
        self.placeholder_pattern = config.get('placeholderPattern', '2025-07-07')
        self.exemption_pattern = config.get('exemptionPattern', 'DATE:EXEMPT')
        self.strict_mode = config.get('strictMode', True)
        self.auto_correct = config.get('autoCorrect', False)
        
        # Initialize database
        self._init_database()
        
    def _init_database(self):
        """Initialize the date audit database."""
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS date_audit (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    file_path TEXT NOT NULL,
                    line_number INTEGER NOT NULL,
                    original_date TEXT,
                    corrected_date TEXT,
                    issue_type TEXT NOT NULL,
                    severity TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    git_commit TEXT,
                    exemption_source TEXT
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS registered_tools (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tool_name TEXT UNIQUE NOT NULL,
                    tool_path TEXT NOT NULL,
                    schema_version TEXT NOT NULL,
                    registration_date TEXT NOT NULL,
                    last_validated TEXT,
                    validation_status TEXT DEFAULT 'pending'
                )
            """)
            
            conn.commit()
    
    def get_canonical_date(self) -> str:
        """Get the canonical current date in UTC."""
        return datetime.now(timezone.utc).strftime(self.date_format)
    
    def get_canonical_datetime(self) -> str:
        """Get the canonical current datetime in UTC."""
        return datetime.now(timezone.utc).strftime(self.time_format)
    
    def get_git_commit_date(self, file_path: str) -> Optional[str]:
        """Get the git commit date for a file."""
        try:
            repo = Repo(search_parent_directories=True)
            file_path = Path(file_path).resolve()
            repo_path = Path(repo.working_dir).resolve()
            
            if file_path.is_relative_to(repo_path):
                rel_path = file_path.relative_to(repo_path)
                commits = list(repo.iter_commits(paths=str(rel_path), max_count=1))
                if commits:
                    return commits[0].committed_datetime.strftime(self.date_format)
        except Exception as e:
            print(f"Warning: Could not get git commit date for {file_path}: {e}")
        
        return None
    
    def validate_date(self, date_str: str, context: str = "") -> Dict[str, Any]:
        """Validate a date string against canonical sources."""
        try:
            parsed_date = date_parser.parse(date_str)
            canonical_date = datetime.now(timezone.utc)
            
            # Check if date is in the future
            if parsed_date > canonical_date:
                return {
                    "valid": False,
                    "issue": "future_date",
                    "severity": "error",
                    "suggestion": f"Date {date_str} is in the future. Use current date: {self.get_canonical_date()}"
                }
            
            # Check if date is too old (more than 1 year)
            if (canonical_date - parsed_date).days > 365:
                return {
                    "valid": False,
                    "issue": "old_date",
                    "severity": "warning",
                    "suggestion": f"Date {date_str} is more than 1 year old. Verify if this is intentional."
                }
            
            return {"valid": True, "issue": None, "severity": None, "suggestion": None}
            
        except Exception as e:
            return {
                "valid": False,
                "issue": "invalid_format",
                "severity": "error",
                "suggestion": f"Invalid date format: {date_str}. Use format: {self.date_format}"
            }
    
    def find_date_patterns(self, content: str) -> List[Tuple[int, str, str]]:
        """Find all date patterns in content."""
        patterns = [
            # ISO date patterns
            (r'\d{4}-\d{2}-\d{2}', 'iso_date'),
            (r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z', 'iso_datetime'),
            # Placeholder patterns
            (re.escape(self.placeholder_pattern), 'placeholder'),
            # Hardcoded date patterns
            (r'2025-01-01', 'hardcoded_date'),
            (r'2025-01-01T00:00:00Z', 'hardcoded_datetime'),
        ]
        
        results = []
        for pattern, pattern_type in patterns:
            for match in re.finditer(pattern, content):
                line_num = content[:match.start()].count('\n') + 1
                results.append((line_num, match.group(), pattern_type))
        
        return results
    
    def check_exemption(self, content: str, line_num: int) -> Optional[str]:
        """Check if a line has a date exemption."""
        lines = content.split('\n')
        if line_num <= len(lines):
            line = lines[line_num - 1]
            if self.exemption_pattern in line:
                # Extract exemption source
                match = re.search(rf'{self.exemption_pattern}\s+source="([^"]+)"', line)
                if match:
                    return match.group(1)
                return "manual_exemption"
        return None
    
    def validate_file(self, file_path: str) -> Dict[str, Any]:
        """Validate dates in a single file."""
        issues = []
        corrected = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            date_patterns = self.find_date_patterns(content)
            
            for line_num, date_str, pattern_type in date_patterns:
                # Check for exemption
                exemption_source = self.check_exemption(content, line_num)
                if exemption_source:
                    continue
                
                if pattern_type == 'placeholder':
                    # Replace placeholder with canonical date
                    new_date = self.get_canonical_date()
                    if self.auto_correct:
                        content = content.replace(date_str, new_date)
                        corrected.append({
                            "line": line_num,
                            "old": date_str,
                            "new": new_date
                        })
                    else:
                        issues.append({
                            "file": file_path,
                            "line": line_num,
                            "issue": "placeholder_not_replaced",
                            "severity": "error",
                            "suggestion": f"Replace {date_str} with {new_date}"
                        })
                
                elif pattern_type in ['hardcoded_date', 'hardcoded_datetime']:
                    # Check if hardcoded date is valid
                    validation = self.validate_date(date_str)
                    if not validation["valid"]:
                        issues.append({
                            "file": file_path,
                            "line": line_num,
                            "issue": validation["issue"],
                            "severity": validation["severity"],
                            "suggestion": validation["suggestion"]
                        })
                
                elif pattern_type in ['iso_date', 'iso_datetime']:
                    # Validate actual dates
                    validation = self.validate_date(date_str)
                    if not validation["valid"]:
                        issues.append({
                            "file": file_path,
                            "line": line_num,
                            "issue": validation["issue"],
                            "severity": validation["severity"],
                            "suggestion": validation["suggestion"]
                        })
            
            # Write corrected content if auto-correct is enabled
            if self.auto_correct and corrected:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
            
            # Log to database
            self._log_validation(file_path, issues, corrected)
            
            return {
                "valid": len(issues) == 0,
                "issues": issues,
                "corrected": corrected
            }
            
        except Exception as e:
            return {
                "valid": False,
                "issues": [{
                    "file": file_path,
                    "line": 0,
                    "issue": "file_error",
                    "severity": "error",
                    "suggestion": f"Could not process file: {e}"
                }],
                "corrected": []
            }
    
    def validate_directory(self, directory: str) -> Dict[str, Any]:
        """Validate dates in all files in a directory."""
        all_issues = []
        all_corrected = []
        valid_files = 0
        total_files = 0
        
        directory_path = Path(directory)
        if not directory_path.exists():
            return {
                "valid": False,
                "issues": [{
                    "file": directory,
                    "line": 0,
                    "issue": "directory_not_found",
                    "severity": "error",
                    "suggestion": f"Directory {directory} does not exist"
                }],
                "corrected": []
            }
        
        # File patterns to check
        patterns = ['*.json', '*.md', '*.py', '*.rs', '*.yml', '*.yaml']
        
        for pattern in patterns:
            for file_path in directory_path.rglob(pattern):
                total_files += 1
                result = self.validate_file(str(file_path))
                
                if result["valid"]:
                    valid_files += 1
                
                all_issues.extend(result["issues"])
                all_corrected.extend(result["corrected"])
        
        return {
            "valid": len(all_issues) == 0,
            "issues": all_issues,
            "corrected": all_corrected,
            "summary": {
                "total_files": total_files,
                "valid_files": valid_files,
                "files_with_issues": total_files - valid_files
            }
        }
    
    def _log_validation(self, file_path: str, issues: List[Dict], corrected: List[Dict]):
        """Log validation results to database."""
        with sqlite3.connect(self.db_path) as conn:
            for issue in issues:
                conn.execute("""
                    INSERT INTO date_audit 
                    (file_path, line_number, original_date, issue_type, severity, timestamp, git_commit)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                """, (
                    file_path,
                    issue.get('line', 0),
                    None,
                    issue.get('issue', 'unknown'),
                    issue.get('severity', 'info'),
                    self.get_canonical_datetime(),
                    self.get_git_commit_date(file_path)
                ))
            
            for correction in corrected:
                conn.execute("""
                    INSERT INTO date_audit 
                    (file_path, line_number, original_date, corrected_date, issue_type, severity, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                """, (
                    file_path,
                    correction.get('line', 0),
                    correction.get('old'),
                    correction.get('new'),
                    'auto_corrected',
                    'info',
                    self.get_canonical_datetime()
                ))
            
            conn.commit()
    
    def register_tool(self, tool_path: str) -> Dict[str, Any]:
        """Register a tool in the database."""
        try:
            with open(tool_path, 'r') as f:
                tool_data = json.load(f)
            
            tool_name = tool_data.get('metadata', {}).get('name', Path(tool_path).stem)
            schema_version = tool_data.get('metadata', {}).get('version', 'unknown')
            
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("""
                    INSERT OR REPLACE INTO registered_tools 
                    (tool_name, tool_path, schema_version, registration_date, last_validated, validation_status)
                    VALUES (?, ?, ?, ?, ?, ?)
                """, (
                    tool_name,
                    tool_path,
                    schema_version,
                    self.get_canonical_datetime(),
                    self.get_canonical_datetime(),
                    'registered'
                ))
                conn.commit()
            
            return {
                "success": True,
                "message": f"Tool {tool_name} registered successfully",
                "data": {
                    "tool_name": tool_name,
                    "tool_path": tool_path,
                    "schema_version": schema_version,
                    "registration_date": self.get_canonical_datetime()
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to register tool: {e}",
                "data": {}
            }
    
    def audit_database(self) -> Dict[str, Any]:
        """Audit the date validation database."""
        with sqlite3.connect(self.db_path) as conn:
            # Get audit statistics
            audit_stats = conn.execute("""
                SELECT 
                    COUNT(*) as total_entries,
                    COUNT(DISTINCT file_path) as unique_files,
                    COUNT(CASE WHEN severity = 'error' THEN 1 END) as errors,
                    COUNT(CASE WHEN severity = 'warning' THEN 1 END) as warnings,
                    COUNT(CASE WHEN severity = 'info' THEN 1 END) as info
                FROM date_audit
            """).fetchone()
            
            # Get registered tools
            tools = conn.execute("""
                SELECT tool_name, tool_path, schema_version, registration_date, validation_status
                FROM registered_tools
                ORDER BY registration_date DESC
            """).fetchall()
            
            return {
                "audit_stats": {
                    "total_entries": audit_stats[0],
                    "unique_files": audit_stats[1],
                    "errors": audit_stats[2],
                    "warnings": audit_stats[3],
                    "info": audit_stats[4]
                },
                "registered_tools": [
                    {
                        "tool_name": tool[0],
                        "tool_path": tool[1],
                        "schema_version": tool[2],
                        "registration_date": tool[3],
                        "validation_status": tool[4]
                    }
                    for tool in tools
                ]
            }


def main():
    """Main CLI interface for the date enforcer."""
    parser = argparse.ArgumentParser(description="Date Enforcement Tool")
    parser.add_argument('command', choices=['validate', 'enforce', 'correct', 'audit', 'register'],
                       help='Command to execute')
    parser.add_argument('--file', '-f', help='File to process')
    parser.add_argument('--directory', '-d', help='Directory to scan')
    parser.add_argument('--strict', action='store_true', default=True,
                       help='Enable strict validation mode')
    parser.add_argument('--auto-correct', action='store_true',
                       help='Automatically correct date issues')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    # Load configuration
    config = {
        'databaseFile': 'date_audit.db',
        'dateFormat': '%Y-%m-%d',
        'timeFormat': '%Y-%m-%dT%H:%M:%SZ',
        'placeholderPattern': '{{TODAY}}',
        'exemptionPattern': 'DATE:EXEMPT',
        'strictMode': args.strict,
        'autoCorrect': args.auto_correct
    }
    
    enforcer = DateEnforcer(config)
    
    try:
        if args.command == 'validate':
            if args.file:
                result = enforcer.validate_file(args.file)
            elif args.directory:
                result = enforcer.validate_directory(args.directory)
            else:
                print("Error: --file or --directory is required for validate command")
                sys.exit(1)
            
            if args.output == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["valid"]:
                    print("✅ All dates are valid")
                else:
                    print("❌ Date validation issues found:")
                    for issue in result["issues"]:
                        print(f"  {issue['file']}:{issue['line']} - {issue['issue']} ({issue['severity']})")
                        if issue.get('suggestion'):
                            print(f"    Suggestion: {issue['suggestion']}")
        
        elif args.command == 'enforce':
            if not args.directory:
                print("Error: --directory is required for enforce command")
                sys.exit(1)
            
            result = enforcer.validate_directory(args.directory)
            if not result["valid"]:
                print("❌ Date enforcement failed. Issues found:")
                for issue in result["issues"]:
                    print(f"  {issue['file']}:{issue['line']} - {issue['issue']}")
                sys.exit(1)
            else:
                print("✅ Date enforcement passed")
        
        elif args.command == 'correct':
            if not args.file and not args.directory:
                print("Error: --file or --directory is required for correct command")
                sys.exit(1)
            
            # Enable auto-correct for this command
            enforcer.auto_correct = True
            enforcer.config['autoCorrect'] = True
            
            if args.file:
                result = enforcer.validate_file(args.file)
            else:
                result = enforcer.validate_directory(args.directory)
            
            if args.output == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["corrected"]:
                    print("✅ Date corrections applied:")
                    for correction in result["corrected"]:
                        print(f"  {correction['file']}:{correction['line']} - {correction['old']} → {correction['new']}")
                else:
                    print("ℹ️ No corrections needed")
        
        elif args.command == 'audit':
            result = enforcer.audit_database()
            if args.output == 'json':
                print(json.dumps(result, indent=2))
            else:
                stats = result["audit_stats"]
                print(f"📊 Date Audit Statistics:")
                print(f"  Total entries: {stats['total_entries']}")
                print(f"  Unique files: {stats['unique_files']}")
                print(f"  Errors: {stats['errors']}")
                print(f"  Warnings: {stats['warnings']}")
                print(f"  Info: {stats['info']}")
                
                tools = result["registered_tools"]
                print(f"\n🔧 Registered Tools ({len(tools)}):")
                for tool in tools:
                    print(f"  {tool['tool_name']} v{tool['schema_version']} ({tool['validation_status']})")
        
        elif args.command == 'register':
            if not args.file:
                print("Error: --file is required for register command")
                sys.exit(1)
            
            result = enforcer.register_tool(args.file)
            if args.output == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["success"]:
                    print(f"✅ {result['message']}")
                else:
                    print(f"❌ {result['message']}")
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 