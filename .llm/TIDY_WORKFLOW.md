# Folder Tidy-Up Workflow

This document describes the folder tidy-up workflow used to clean up the `.cursor` directory and provides a general tool for future refactoring tasks.

## Workflow Summary

### 1. Cursor Directory Cleanup

The `.cursor` directory was cleaned up to serve as a lightweight pointer to the universal extension:

**Before:**
```
.cursor/
├── schemas/                  # Duplicated schemas
├── rules/                    # Duplicated rules
├── scripts/                  # Duplicated scripts
├── version-manager.py        # Duplicated tool
├── version_db.json           # Duplicated database
├── environment.json          # Legacy config
├── README-Cursor-Integration.md
└── TROUBLESHOOTING-Ownership.md
```

**After:**
```
.cursor/
├── cursor-extension.json     # Cursor extension schema
├── cursor-config.json        # Cursor-specific configuration
└── README.md                # Updated documentation
```

### 2. Universal Extension Structure

All functionality moved to the universal extension:

```
.llm/
├── extension.json            # Universal extension schema
├── extension-config.json     # Universal extension configuration
├── schemas/                  # All schema definitions
├── rules/                    # Rule definitions
├── tools/                    # Tool specifications
├── agents/                   # Agent specifications
├── scripts/                  # Utility scripts
├── date-contract.json        # Date enforcement contract
└── DATE_ENFORCEMENT.md      # Date enforcement documentation
```

## Folder Tidy Tool

### Overview

The `folder-tidy.py` tool provides comprehensive cleanup and organization capabilities:

- **Analysis**: Detect duplicates, stale files, orphaned files
- **Cleaning**: Remove duplicates and unnecessary files
- **Organization**: Structure files into logical categories
- **Backup**: Create backups before making changes
- **Audit**: Track all operations in database

### Usage

#### 1. Analyze Directory

```bash
# Analyze directory for cleanup opportunities
python .llm/scripts/folder-tidy.py analyze <directory> --output text

# Example
python .llm/scripts/folder-tidy.py analyze .cursor --output text
```

#### 2. Clean Directory

```bash
# Show what would be cleaned (dry run)
python .llm/scripts/folder-tidy.py clean <directory> --dry-run

# Actually clean the directory
python .llm/scripts/folder-tidy.py clean <directory> --no-dry-run
```

#### 3. Organize Directory

```bash
# Organize files into categories
python .llm/scripts/folder-tidy.py organize <directory> --dry-run
```

#### 4. Create Backup

```bash
# Create backup of directory
python .llm/scripts/folder-tidy.py backup <directory>
```

### Configuration

The tool uses configurable patterns and rules:

```json
{
  "excludePatterns": [
    ".git",
    "node_modules", 
    "__pycache__",
    "*.tmp",
    "*.bak"
  ],
  "duplicateDetection": true,
  "staleFileDetection": true,
  "orphanedFileDetection": true,
  "maxBackupAge": 30
}
```

### Organization Rules

Files are organized into categories:

- **images**: `*.jpg`, `*.jpeg`, `*.png`, `*.gif`, `*.svg`, `*.ico`
- **documents**: `*.pdf`, `*.doc`, `*.docx`, `*.txt`, `*.md`
- **archives**: `*.zip`, `*.tar`, `*.gz`, `*.rar`, `*.7z`
- **temp**: `*.tmp`, `*.temp`, `*.cache`
- **backups**: `*.bak`, `*.backup`, `*.old`

## Database Schema

### Tidy Operations Table

```sql
CREATE TABLE tidy_operations (
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
);
```

### File Analysis Table

```sql
CREATE TABLE file_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    file_hash TEXT,
    file_size INTEGER,
    last_modified TEXT,
    analysis_type TEXT NOT NULL,
    issue_description TEXT,
    severity TEXT NOT NULL,
    timestamp TEXT NOT NULL
);
```

### Backups Table

```sql
CREATE TABLE backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backup_path TEXT NOT NULL,
    original_directory TEXT NOT NULL,
    backup_size INTEGER,
    created_date TEXT NOT NULL,
    expires_date TEXT,
    status TEXT DEFAULT 'active'
);
```

## Best Practices

### 1. Always Use Dry Run First

```bash
# Always test with dry run
python .llm/scripts/folder-tidy.py clean <directory> --dry-run
```

### 2. Create Backups

```bash
# Enable backups (default)
python .llm/scripts/folder-tidy.py clean <directory>

# Skip backups (not recommended)
python .llm/scripts/folder-tidy.py clean <directory> --no-backup
```

### 3. Review Analysis Results

```bash
# Analyze before cleaning
python .llm/scripts/folder-tidy.py analyze <directory> --output json
```

### 4. Clean Up Old Backups

```bash
# The tool automatically cleans up old backups
# Based on maxBackupAge configuration
```

## Future Refactoring Workflows

### 1. Extension Migration

When migrating to a new extension structure:

```bash
# 1. Analyze current structure
python .llm/scripts/folder-tidy.py analyze <old-extension> --output json

# 2. Create backup
python .llm/scripts/folder-tidy.py backup <old-extension>

# 3. Clean up old files
python .llm/scripts/folder-tidy.py clean <old-extension> --no-dry-run

# 4. Organize new structure
python .llm/scripts/folder-tidy.py organize <new-extension> --no-dry-run
```

### 2. Schema Migration

When updating schemas:

```bash
# 1. Backup old schemas
python .llm/scripts/folder-tidy.py backup schemas/

# 2. Clean up old schema files
python .llm/scripts/folder-tidy.py clean schemas/ --no-dry-run

# 3. Validate new schemas
python .llm/scripts/validate-dates.py --command validate --directory schemas/
```

### 3. Tool Migration

When migrating tools:

```bash
# 1. Register new tools
python .llm/scripts/tool-registry.py register --tool-path tools/new-tool.json

# 2. Clean up old tools
python .llm/scripts/folder-tidy.py clean tools/ --no-dry-run

# 3. Validate tool registry
python .llm/scripts/tool-registry.py audit
```

## Integration with Date Enforcement

The folder tidy tool integrates with the date enforcement system:

```bash
# 1. Tidy up directory
python .llm/scripts/folder-tidy.py clean <directory> --no-dry-run

# 2. Validate dates in cleaned files
python .llm/scripts/validate-dates.py --command validate --directory <directory>

# 3. Register any new tools
python .llm/scripts/tool-registry.py register --tool-path <new-tool>
```

## Error Handling

### Common Issues

1. **Permission denied**: Check file permissions
2. **Directory not found**: Verify path exists
3. **Backup failed**: Check disk space
4. **Database locked**: Close other processes using database

### Recovery

```bash
# Restore from backup
# (Backup paths are logged in tidy_audit.db)

# Check backup status
sqlite3 tidy_audit.db "SELECT backup_path, created_date FROM backups WHERE original_directory = '<directory>' ORDER BY created_date DESC LIMIT 1;"
```

## Monitoring and Auditing

### View Operation History

```bash
# Check tidy operations
sqlite3 tidy_audit.db "SELECT operation_type, directory, files_processed, files_deleted, timestamp FROM tidy_operations ORDER BY timestamp DESC LIMIT 10;"
```

### View File Analysis

```bash
# Check file analysis
sqlite3 tidy_audit.db "SELECT file_path, analysis_type, issue_description, severity FROM file_analysis WHERE severity != 'info' ORDER BY timestamp DESC LIMIT 10;"
```

## Conclusion

The folder tidy-up workflow provides:

1. **Systematic cleanup** of directories and files
2. **Safe operations** with dry-run and backup capabilities
3. **Audit trail** for all operations
4. **Integration** with date enforcement and tool registry
5. **Reusable tool** for future refactoring tasks

This workflow ensures clean, organized, and maintainable codebases while preserving history and providing rollback capabilities. 