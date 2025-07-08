# Date Enforcement System

This document describes the comprehensive date enforcement system that ensures deterministic, accurate dating through schema contracts and prevents date hallucination.

## Overview

The date enforcement system provides:
- **Deterministic dating** through canonical sources (system clock, git commits)
- **Schema contract enforcement** to prevent date hallucination
- **Tool registry** for secure tool management
- **Comprehensive validation** across the entire codebase
- **Automatic correction** of placeholder dates

## Architecture

### 1. Date Contract Schema

The `date-contract-v1.json` schema defines the contract for date enforcement:

```json
{
  "dateContract": {
    "version": "1.0.0",
    "enforcement": {
      "strict": true,
      "canonicalSource": "system_clock",
      "timezone": "UTC",
      "format": "%Y-%m-%d",
      "datetimeFormat": "%Y-%m-%dT%H:%M:%SZ"
    },
    "validation": {
      "allowFutureDates": false,
      "maxAgeDays": 365,
      "placeholderPattern": "{{TODAY}}",
      "exemptionPattern": "DATE:EXEMPT",
      "hardcodedPatterns": ["2024-01-01", "2024-01-01T00:00:00Z"]
    }
  }
}
```

### 2. Date Enforcer Tool

The `date-enforcer.py` tool provides:

- **Canonical date sources**: System clock, git commit dates
- **Pattern detection**: Finds hardcoded dates, placeholders, exemptions
- **Validation**: Checks for future dates, old dates, invalid formats
- **Auto-correction**: Replaces placeholders with current dates
- **Database logging**: Tracks all date operations and issues

### 3. Tool Registry

The `tool-registry.py` tool manages:

- **Secure tool registration** in SQLite database
- **Schema validation** against universal envelope
- **Date validation** for all registered tools
- **Audit trail** of tool operations
- **Version tracking** and validation history

### 4. Comprehensive Validator

The `validate-dates.py` script integrates:

- **Contract updates** with current dates
- **Full codebase validation**
- **Tool registration** and validation
- **Audit reporting** for both date and tool systems

## Usage

### Basic Date Validation

```bash
# Validate dates in a file
python scripts/date-enforcer.py validate --file my-file.json

# Validate dates in a directory
python scripts/date-enforcer.py validate --directory .

# Auto-correct date issues
python scripts/date-enforcer.py correct --file my-file.json
```

### Tool Registration

```bash
# Register a tool
python scripts/tool-registry.py register --tool-path tools/my-tool.json

# List registered tools
python scripts/tool-registry.py list

# Validate a specific tool
python scripts/tool-registry.py validate --tool-name my-tool
```

### Comprehensive Validation

```bash
# Run full validation
python scripts/validate-dates.py --command comprehensive

# Validate specific directory
python scripts/validate-dates.py --command validate --directory .

# Register all tools
python scripts/validate-dates.py --command register --tools-dir tools
```

## Date Patterns

### 1. Placeholder Pattern

Use `{{TODAY}}` for dates that should be replaced with the current date:

```json
{
  "metadata": {
    "created": "{{TODAY}}T00:00:00Z",
    "lastModified": "{{TODAY}}T00:00:00Z"
  }
}
```

### 2. Exemption Pattern

Use `DATE:EXEMPT` for intentional historical dates:

```json
{
  "metadata": {
    "created": "2023-01-01T00:00:00Z"  // DATE:EXEMPT source="Historical reference"
  }
}
```

### 3. Canonical Date Format

All dates use ISO 8601 format:
- Date: `YYYY-MM-DD`
- DateTime: `YYYY-MM-DDTHH:MM:SSZ`

## Validation Rules

### 1. No Future Dates

Dates cannot be in the future unless explicitly allowed:

```json
{
  "validation": {
    "allowFutureDates": false
  }
}
```

### 2. Age Limits

Dates cannot be older than specified limit:

```json
{
  "validation": {
    "maxAgeDays": 365
  }
}
```

### 3. No Hardcoded Dates

Prevents common hardcoded patterns:

```json
{
  "validation": {
    "hardcodedPatterns": [
      "2024-01-01",
      "2024-01-01T00:00:00Z"
    ]
  }
}
```

## Database Schema

### Date Audit Table

```sql
CREATE TABLE date_audit (
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
);
```

### Tool Registry Table

```sql
CREATE TABLE tools (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_name TEXT UNIQUE NOT NULL,
    tool_path TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    registration_date TEXT NOT NULL,
    last_validated TEXT,
    validation_status TEXT DEFAULT 'pending',
    schema_hash TEXT,
    metadata_hash TEXT,
    capabilities_hash TEXT,
    exemption_source TEXT
);
```

## Integration with Universal Extension

### 1. Schema Integration

All resources use the universal envelope schema with date enforcement:

```json
{
  "$schema": "https://github.com/toka-ai/toka/.llm/schemas/resource-envelope-v1.json",
  "metadata": {
    "created": "{{TODAY}}T00:00:00Z",
    "lastModified": "{{TODAY}}T00:00:00Z"
  }
}
```

### 2. Tool Registration

Tools are automatically registered and validated:

```bash
# Register all tools in the extension
python scripts/validate-dates.py --command register --tools-dir tools
```

### 3. Continuous Validation

Date validation is integrated into the development workflow:

```bash
# Pre-commit validation
python scripts/validate-dates.py --command validate --directory .

# Post-commit audit
python scripts/date-enforcer.py audit
```

## Best Practices

### 1. Use Placeholders

Always use `{{TODAY}}` for current dates:

```json
{
  "metadata": {
    "created": "{{TODAY}}T00:00:00Z",
    "lastModified": "{{TODAY}}T00:00:00Z"
  }
}
```

### 2. Document Historical Dates

Use exemptions for intentional historical references:

```json
{
  "metadata": {
    "created": "2023-01-01T00:00:00Z"  // DATE:EXEMPT source="RFC 2119, published 1997-03-01"
  }
}
```

### 3. Validate Before Committing

Run validation before committing changes:

```bash
python scripts/validate-dates.py --command comprehensive
```

### 4. Register New Tools

Register new tools when created:

```bash
python scripts/tool-registry.py register --tool-path tools/new-tool.json
```

## Error Handling

### 1. Future Date Detection

```bash
# Error: Date 2025-12-31 is in the future
# Suggestion: Use current date: 2025-07-07
```

### 2. Hardcoded Date Detection

```bash
# Error: Contains hardcoded date: 2024-01-01
# Suggestion: Use {{TODAY}} placeholder
```

### 3. Placeholder Detection

```bash
# Error: Contains placeholder date: {{TODAY}}
# Suggestion: Replace with current date
```

## Configuration

### Environment Variables

```bash
export DATE_ENFORCEMENT_STRICT=true
export DATE_ENFORCEMENT_TIMEZONE=UTC
export DATE_ENFORCEMENT_FORMAT=%Y-%m-%d
```

### Contract Configuration

```json
{
  "dateContract": {
    "enforcement": {
      "strict": true,
      "canonicalSource": "system_clock",
      "timezone": "UTC"
    }
  }
}
```

## Troubleshooting

### Common Issues

1. **Future dates detected**: Use current date or add exemption
2. **Hardcoded dates found**: Replace with `{{TODAY}}` placeholder
3. **Placeholders not replaced**: Run auto-correction
4. **Tool registration failed**: Check schema validation

### Debug Commands

```bash
# Check date contract
python scripts/validate-dates.py --command validate --file date-contract.json

# Audit database
python scripts/date-enforcer.py audit

# List registered tools
python scripts/tool-registry.py list
```

## Future Enhancements

### 1. Git Integration

- Automatic date extraction from git commits
- Commit date validation
- Historical date tracking

### 2. CI/CD Integration

- Pre-commit hooks for date validation
- Automated date correction
- Date audit reporting

### 3. Rust Implementation

- Port tools to Rust for performance
- Native database integration
- Cross-platform compatibility

## Conclusion

The date enforcement system ensures deterministic, accurate dating through:

1. **Schema contracts** that prevent date hallucination
2. **Canonical sources** for current dates
3. **Comprehensive validation** across the codebase
4. **Secure tool registry** with audit trails
5. **Automatic correction** of placeholder dates

This system maintains date integrity while providing flexibility for historical references and exemptions. 