#!/usr/bin/env python3
"""
Folder Tidy-Up Tool

General folder tidy-up tool for cleanup and refactoring tasks.
Provides analysis, cleaning, organization, and backup capabilities.
"""

import json
import os
import sys
import argparse
import sqlite3
import shutil
import hashlib
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
import fnmatch


class FolderTidy:
    """General folder tidy-up tool for cleanup and refactoring."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize the folder tidy tool with configuration."""
        self.config = config
        self.backup_dir = Path(config.get('backupDirectory', 'backups'))
        self.log_file = config.get('logFile', 'tidy.log')
        self.max_backup_age = int(config.get('maxBackupAge', 30))
        self.exclude_patterns = config.get('excludePatterns', [])
        self.duplicate_detection = config.get('duplicateDetection', True)
        self.stale_file_detection = config.get('staleFileDetection', True)
        self.orphaned_file_detection = config.get('orphanedFileDetection', True)
        
        # Initialize database
        self._init_database()
        
    def _init_database(self):
        """Initialize the tidy-up database."""
        self.db_path = Path('tidy_audit.db')
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS tidy_operations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    operation_type TEXT NOT NULL,
                    directory TEXT NOT NULL,
                    files_processed INTEGER,
                    files_cleaned INTEGER,
                    files_moved INTEGER,
                    files_deleted INTEGER,
                    backup_created TEXT,
                    timestamp TEXT NOT NULL,
                    dry_run BOOLEAN DEFAULT FALSE
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS file_analysis (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    file_path TEXT NOT NULL,
                    file_hash TEXT,
                    file_size INTEGER,
                    last_modified TEXT,
                    analysis_type TEXT NOT NULL,
                    issue_description TEXT,
                    severity TEXT NOT NULL,
                    timestamp TEXT NOT NULL
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS backups (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    backup_path TEXT NOT NULL,
                    original_directory TEXT NOT NULL,
                    backup_size INTEGER,
                    created_date TEXT NOT NULL,
                    expires_date TEXT,
                    status TEXT DEFAULT 'active'
                )
            """)
            
            conn.commit()
    
    def should_exclude(self, file_path: Path) -> bool:
        """Check if file should be excluded based on patterns."""
        file_str = str(file_path)
        for pattern in self.exclude_patterns:
            if fnmatch.fnmatch(file_str, pattern):
                return True
        return False
    
    def calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA-256 hash of file."""
        try:
            with open(file_path, 'rb') as f:
                return hashlib.sha256(f.read()).hexdigest()
        except Exception:
            return ""
    
    def analyze_directory(self, directory: str) -> Dict[str, Any]:
        """Analyze directory for cleanup opportunities."""
        directory_path = Path(directory)
        if not directory_path.exists():
            return {
                "success": False,
                "message": f"Directory not found: {directory}",
                "data": {}
            }
        
        files_processed = 0
        duplicates = []
        stale_files = []
        orphaned_files = []
        issues = []
        
        # Collect all files
        all_files = []
        for file_path in directory_path.rglob("*"):
            if file_path.is_file() and not self.should_exclude(file_path):
                all_files.append(file_path)
                files_processed += 1
        
        # Detect duplicates
        if self.duplicate_detection:
            hash_map = {}
            for file_path in all_files:
                file_hash = self.calculate_file_hash(file_path)
                if file_hash:
                    if file_hash in hash_map:
                        duplicates.append({
                            "original": str(hash_map[file_hash]),
                            "duplicate": str(file_path),
                            "hash": file_hash
                        })
                    else:
                        hash_map[file_hash] = file_path
        
        # Detect stale files
        if self.stale_file_detection:
            current_time = datetime.now(timezone.utc)
            stale_threshold = current_time - timedelta(days=90)  # 90 days
            
            for file_path in all_files:
                try:
                    mtime = datetime.fromtimestamp(file_path.stat().st_mtime, tz=timezone.utc)
                    if mtime < stale_threshold:
                        stale_files.append({
                            "file": str(file_path),
                            "last_modified": mtime.isoformat(),
                            "age_days": (current_time - mtime).days
                        })
                except Exception as e:
                    issues.append({
                        "file": str(file_path),
                        "issue": f"Could not check modification time: {e}",
                        "severity": "warning"
                    })
        
        # Detect orphaned files
        if self.orphaned_file_detection:
            # Look for files that might be orphaned (e.g., .tmp, .bak, etc.)
            orphaned_patterns = ['*.tmp', '*.bak', '*.old', '*.backup']
            for file_path in all_files:
                for pattern in orphaned_patterns:
                    if fnmatch.fnmatch(file_path.name, pattern):
                        orphaned_files.append({
                            "file": str(file_path),
                            "pattern": pattern,
                            "size": file_path.stat().st_size
                        })
                        break
        
        # Log analysis to database
        self._log_analysis(directory, all_files, duplicates, stale_files, orphaned_files)
        
        return {
            "success": True,
            "message": f"Analysis completed for {directory}",
            "data": {
                "files_processed": files_processed,
                "duplicates": duplicates,
                "stale_files": stale_files,
                "orphaned_files": orphaned_files,
                "issues": issues,
                "summary": {
                    "total_files": len(all_files),
                    "duplicate_count": len(duplicates),
                    "stale_count": len(stale_files),
                    "orphaned_count": len(orphaned_files)
                }
            }
        }
    
    def clean_directory(self, directory: str, dry_run: bool = True) -> Dict[str, Any]:
        """Clean directory by removing duplicates, stale files, etc."""
        directory_path = Path(directory)
        if not directory_path.exists():
            return {
                "success": False,
                "message": f"Directory not found: {directory}",
                "data": {}
            }
        
        # Create backup if not dry run
        backup_path = None
        if not dry_run:
            backup_path = self._create_backup(directory)
        
        files_cleaned = 0
        files_deleted = 0
        files_moved = 0
        issues = []
        
        # Analyze first
        analysis = self.analyze_directory(directory)
        if not analysis["success"]:
            return analysis
        
        data = analysis["data"]
        
        # Remove duplicates (keep the first one)
        for duplicate in data["duplicates"]:
            duplicate_path = Path(duplicate["duplicate"])
            if not dry_run:
                try:
                    duplicate_path.unlink()
                    files_deleted += 1
                except Exception as e:
                    issues.append({
                        "file": str(duplicate_path),
                        "issue": f"Failed to delete duplicate: {e}",
                        "severity": "error"
                    })
            else:
                files_deleted += 1
        
        # Remove stale files
        for stale_file in data["stale_files"]:
            file_path = Path(stale_file["file"])
            if not dry_run:
                try:
                    file_path.unlink()
                    files_deleted += 1
                except Exception as e:
                    issues.append({
                        "file": str(file_path),
                        "issue": f"Failed to delete stale file: {e}",
                        "severity": "error"
                    })
            else:
                files_deleted += 1
        
        # Remove orphaned files
        for orphaned_file in data["orphaned_files"]:
            file_path = Path(orphaned_file["file"])
            if not dry_run:
                try:
                    file_path.unlink()
                    files_deleted += 1
                except Exception as e:
                    issues.append({
                        "file": str(file_path),
                        "issue": f"Failed to delete orphaned file: {e}",
                        "severity": "error"
                    })
            else:
                files_deleted += 1
        
        # Log operation
        self._log_operation("clean", directory, data["files_processed"], 
                           files_cleaned, files_moved, files_deleted, 
                           backup_path, dry_run)
        
        return {
            "success": True,
            "message": f"Cleanup completed for {directory}",
            "data": {
                "files_processed": data["files_processed"],
                "files_cleaned": files_cleaned,
                "files_moved": files_moved,
                "files_deleted": files_deleted,
                "backup_created": str(backup_path) if backup_path else None,
                "issues": issues,
                "dry_run": dry_run
            }
        }
    
    def organize_directory(self, directory: str, dry_run: bool = True) -> Dict[str, Any]:
        """Organize directory structure."""
        directory_path = Path(directory)
        if not directory_path.exists():
            return {
                "success": False,
                "message": f"Directory not found: {directory}",
                "data": {}
            }
        
        # Create backup if not dry run
        backup_path = None
        if not dry_run:
            backup_path = self._create_backup(directory)
        
        files_moved = 0
        issues = []
        
        # Define organization rules
        organization_rules = {
            "images": ["*.jpg", "*.jpeg", "*.png", "*.gif", "*.svg", "*.ico"],
            "documents": ["*.pdf", "*.doc", "*.docx", "*.txt", "*.md"],
            "archives": ["*.zip", "*.tar", "*.gz", "*.rar", "*.7z"],
            "temp": ["*.tmp", "*.temp", "*.cache"],
            "backups": ["*.bak", "*.backup", "*.old"]
        }
        
        # Create directories and move files
        for category, patterns in organization_rules.items():
            category_dir = directory_path / category
            if not dry_run:
                category_dir.mkdir(exist_ok=True)
            
            for file_path in directory_path.glob("*"):
                if file_path.is_file() and not self.should_exclude(file_path):
                    for pattern in patterns:
                        if fnmatch.fnmatch(file_path.name, pattern):
                            target_path = category_dir / file_path.name
                            if not dry_run:
                                try:
                                    shutil.move(str(file_path), str(target_path))
                                    files_moved += 1
                                except Exception as e:
                                    issues.append({
                                        "file": str(file_path),
                                        "issue": f"Failed to move file: {e}",
                                        "severity": "error"
                                    })
                            else:
                                files_moved += 1
                            break
        
        # Log operation
        self._log_operation("organize", directory, 0, 0, files_moved, 0, 
                           backup_path, dry_run)
        
        return {
            "success": True,
            "message": f"Organization completed for {directory}",
            "data": {
                "files_processed": 0,
                "files_cleaned": 0,
                "files_moved": files_moved,
                "files_deleted": 0,
                "backup_created": str(backup_path) if backup_path else None,
                "issues": issues,
                "dry_run": dry_run
            }
        }
    
    def _create_backup(self, directory: str) -> Optional[Path]:
        """Create backup of directory."""
        try:
            timestamp = datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')
            backup_name = f"{Path(directory).name}_{timestamp}"
            backup_path = self.backup_dir / backup_name
            
            self.backup_dir.mkdir(parents=True, exist_ok=True)
            shutil.copytree(directory, backup_path)
            
            # Log backup
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("""
                    INSERT INTO backups (backup_path, original_directory, backup_size, created_date, expires_date)
                    VALUES (?, ?, ?, ?, ?)
                """, (
                    str(backup_path),
                    directory,
                    sum(f.stat().st_size for f in backup_path.rglob('*') if f.is_file()),
                    datetime.now(timezone.utc).isoformat(),
                    (datetime.now(timezone.utc) + timedelta(days=self.max_backup_age)).isoformat()
                ))
                conn.commit()
            
            return backup_path
        except Exception as e:
            print(f"Warning: Failed to create backup: {e}")
            return None
    
    def _log_analysis(self, directory: str, files: List[Path], duplicates: List[Dict], 
                     stale_files: List[Dict], orphaned_files: List[Dict]):
        """Log analysis results to database."""
        with sqlite3.connect(self.db_path) as conn:
            for file_path in files:
                file_hash = self.calculate_file_hash(file_path)
                try:
                    mtime = datetime.fromtimestamp(file_path.stat().st_mtime, tz=timezone.utc)
                    last_modified = mtime.isoformat()
                except Exception:
                    last_modified = None
                
                conn.execute("""
                    INSERT INTO file_analysis 
                    (file_path, file_hash, file_size, last_modified, analysis_type, issue_description, severity, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    str(file_path),
                    file_hash,
                    file_path.stat().st_size,
                    last_modified,
                    "general",
                    None,
                    "info",
                    datetime.now(timezone.utc).isoformat()
                ))
            
            # Log duplicates
            for duplicate in duplicates:
                conn.execute("""
                    INSERT INTO file_analysis 
                    (file_path, file_hash, analysis_type, issue_description, severity, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?)
                """, (
                    duplicate["duplicate"],
                    duplicate["hash"],
                    "duplicate",
                    f"Duplicate of {duplicate['original']}",
                    "warning",
                    datetime.now(timezone.utc).isoformat()
                ))
            
            # Log stale files
            for stale_file in stale_files:
                conn.execute("""
                    INSERT INTO file_analysis 
                    (file_path, last_modified, analysis_type, issue_description, severity, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?)
                """, (
                    stale_file["file"],
                    stale_file["last_modified"],
                    "stale",
                    f"File not modified for {stale_file['age_days']} days",
                    "warning",
                    datetime.now(timezone.utc).isoformat()
                ))
            
            # Log orphaned files
            for orphaned_file in orphaned_files:
                conn.execute("""
                    INSERT INTO file_analysis 
                    (file_path, file_size, analysis_type, issue_description, severity, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?)
                """, (
                    orphaned_file["file"],
                    orphaned_file["size"],
                    "orphaned",
                    f"Orphaned file matching pattern {orphaned_file['pattern']}",
                    "info",
                    datetime.now(timezone.utc).isoformat()
                ))
            
            conn.commit()
    
    def _log_operation(self, operation_type: str, directory: str, files_processed: int,
                       files_cleaned: int, files_moved: int, files_deleted: int,
                       backup_path: Optional[Path], dry_run: bool):
        """Log operation to database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                INSERT INTO tidy_operations 
                (operation_type, directory, files_processed, files_cleaned, files_moved, files_deleted, backup_created, timestamp, dry_run)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                operation_type,
                directory,
                files_processed,
                files_cleaned,
                files_moved,
                files_deleted,
                str(backup_path) if backup_path else None,
                datetime.now(timezone.utc).isoformat(),
                dry_run
            ))
            conn.commit()
    
    def cleanup_old_backups(self) -> Dict[str, Any]:
        """Clean up old backups."""
        if not self.backup_dir.exists():
            return {
                "success": True,
                "message": "No backup directory found",
                "data": {"backups_removed": 0}
            }
        
        backups_removed = 0
        current_time = datetime.now(timezone.utc)
        cutoff_date = current_time - timedelta(days=self.max_backup_age)
        
        for backup_path in self.backup_dir.iterdir():
            if backup_path.is_dir():
                try:
                    # Check if backup is old
                    backup_time = datetime.fromtimestamp(backup_path.stat().st_mtime, tz=timezone.utc)
                    if backup_time < cutoff_date:
                        shutil.rmtree(backup_path)
                        backups_removed += 1
                except Exception:
                    pass
        
        return {
            "success": True,
            "message": f"Cleaned up {backups_removed} old backups",
            "data": {"backups_removed": backups_removed}
        }


def main():
    """Main CLI interface for the folder tidy tool."""
    parser = argparse.ArgumentParser(description="Folder Tidy-Up Tool")
    parser.add_argument('command', choices=['analyze', 'clean', 'organize', 'backup', 'restore'],
                       help='Command to execute')
    parser.add_argument('directory', help='Directory to process')
    parser.add_argument('--dry-run', action='store_true', default=True,
                       help='Show what would be done without making changes')
    parser.add_argument('--no-backup', action='store_true',
                       help='Skip backup creation')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    # Load configuration
    config = {
        'backupDirectory': 'backups',
        'logFile': 'tidy.log',
        'maxBackupAge': 30,
        'excludePatterns': [
            '.git',
            'node_modules',
            '__pycache__',
            '*.tmp',
            '*.bak'
        ],
        'duplicateDetection': True,
        'staleFileDetection': True,
        'orphanedFileDetection': True
    }
    
    tidy = FolderTidy(config)
    
    try:
        if args.command == 'analyze':
            result = tidy.analyze_directory(args.directory)
            
        elif args.command == 'clean':
            result = tidy.clean_directory(args.directory, args.dry_run)
            
        elif args.command == 'organize':
            result = tidy.organize_directory(args.directory, args.dry_run)
            
        elif args.command == 'backup':
            backup_path = tidy._create_backup(args.directory)
            result = {
                "success": True,
                "message": f"Backup created: {backup_path}",
                "data": {"backup_path": str(backup_path) if backup_path else None}
            }
            
        elif args.command == 'restore':
            # This would implement restore functionality
            result = {
                "success": False,
                "message": "Restore functionality not implemented yet",
                "data": {}
            }
        
        if args.output == 'json':
            print(json.dumps(result, indent=2))
        else:
            if result["success"]:
                print(f"✅ {result['message']}")
                if 'data' in result:
                    data = result['data']
                    if 'files_processed' in data:
                        print(f"  Files processed: {data['files_processed']}")
                    if 'files_cleaned' in data:
                        print(f"  Files cleaned: {data['files_cleaned']}")
                    if 'files_moved' in data:
                        print(f"  Files moved: {data['files_moved']}")
                    if 'files_deleted' in data:
                        print(f"  Files deleted: {data['files_deleted']}")
                    if 'backup_created' in data and data['backup_created']:
                        print(f"  Backup created: {data['backup_created']}")
            else:
                print(f"❌ {result['message']}")
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 