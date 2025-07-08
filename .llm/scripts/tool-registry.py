#!/usr/bin/env python3
"""
Tool Registry

Manages registered tools securely in a database with validation and tracking.
Integrates with the date enforcer to ensure deterministic dating.
"""

import json
import os
import sys
import argparse
import sqlite3
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Any, List, Optional
import click
import jsonschema
from jsonschema import RefResolver


class ToolRegistry:
    """Manages registered tools securely in a database."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize the tool registry with configuration."""
        self.config = config
        self.db_path = Path(config.get('databaseFile', 'tool_registry.db'))
        self.schema_root = Path(config.get('schemaRoot', 'schemas'))
        self.validation_strict = config.get('validationStrict', True)
        self.auto_validate = config.get('autoValidate', True)
        self.backup_enabled = config.get('backupEnabled', True)
        
        # Initialize database
        self._init_database()
        
    def _init_database(self):
        """Initialize the tool registry database."""
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS tools (
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
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS tool_audit (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tool_name TEXT NOT NULL,
                    action TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    details TEXT,
                    user TEXT,
                    git_commit TEXT
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS validation_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tool_name TEXT NOT NULL,
                    validation_date TEXT NOT NULL,
                    validation_status TEXT NOT NULL,
                    issues TEXT,
                    schema_version TEXT,
                    validator_version TEXT
                )
            """)
            
            conn.commit()
    
    def get_canonical_datetime(self) -> str:
        """Get the canonical current datetime in UTC."""
        return datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
    
    def calculate_hash(self, data: str) -> str:
        """Calculate SHA-256 hash of data."""
        return hashlib.sha256(data.encode('utf-8')).hexdigest()
    
    def validate_tool_schema(self, tool_path: str) -> Dict[str, Any]:
        """Validate a tool against the universal envelope schema."""
        try:
            with open(tool_path, 'r') as f:
                tool_data = json.load(f)
            
            # Load schemas
            schemas = {}
            for schema_file in self.schema_root.glob("*.json"):
                with open(schema_file, 'r') as f:
                    schema_data = json.load(f)
                    schema_id = schema_data.get('$id', schema_file.name)
                    schemas[schema_id] = schema_data
            
            # Get the universal envelope schema
            envelope_schema = schemas.get('resource-envelope-v1.json')
            if not envelope_schema:
                return {
                    "valid": False,
                    "issues": ["Universal envelope schema not found"]
                }
            
            # Create resolver
            resolver = RefResolver.from_schema(envelope_schema, store=schemas)
            
            # Validate against envelope schema
            jsonschema.validate(tool_data, envelope_schema, resolver=resolver)
            
            # Additional tool-specific validation
            if tool_data.get('kind') != 'tool':
                return {
                    "valid": False,
                    "issues": ["Tool must have kind 'tool'"]
                }
            
            # Check required fields
            required_fields = ['metadata', 'spec']
            for field in required_fields:
                if field not in tool_data:
                    return {
                        "valid": False,
                        "issues": [f"Missing required field: {field}"]
                    }
            
            # Validate metadata
            metadata = tool_data.get('metadata', {})
            required_metadata = ['name', 'version', 'title', 'description']
            for field in required_metadata:
                if field not in metadata:
                    return {
                        "valid": False,
                        "issues": [f"Missing required metadata field: {field}"]
                    }
            
            return {"valid": True, "issues": []}
            
        except jsonschema.ValidationError as e:
            return {
                "valid": False,
                "issues": [f"Schema validation error: {e.message}"]
            }
        except Exception as e:
            return {
                "valid": False,
                "issues": [f"Validation error: {e}"]
            }
    
    def validate_tool_dates(self, tool_path: str) -> Dict[str, Any]:
        """Validate dates in a tool specification."""
        try:
            with open(tool_path, 'r') as f:
                content = f.read()
            
            issues = []
            
            # Check for placeholder dates
            placeholder_pattern = '2025-07-07'
            if placeholder_pattern in content:
                issues.append(f"Contains placeholder date: {placeholder_pattern}")
            
            # Check for hardcoded dates
            hardcoded_dates = ['2025-01-01', '2025-01-01T00:00:00Z']
            for date in hardcoded_dates:
                if date in content:
                    issues.append(f"Contains hardcoded date: {date}")
            
            # Check for future dates
            current_date = datetime.now(timezone.utc)
            date_pattern = r'\d{4}-\d{2}-\d{2}'
            import re
            for match in re.finditer(date_pattern, content):
                try:
                    date_str = match.group()
                    parsed_date = datetime.strptime(date_str, '%Y-%m-%d')
                    if parsed_date > current_date:
                        issues.append(f"Future date found: {date_str}")
                except ValueError:
                    pass
            
            return {
                "valid": len(issues) == 0,
                "issues": issues
            }
            
        except Exception as e:
            return {
                "valid": False,
                "issues": [f"Date validation error: {e}"]
            }
    
    def register_tool(self, tool_path: str) -> Dict[str, Any]:
        """Register a tool in the database."""
        try:
            # Validate tool path
            if not Path(tool_path).exists():
                return {
                    "success": False,
                    "message": f"Tool file not found: {tool_path}",
                    "data": {}
                }
            
            # Load tool data
            with open(tool_path, 'r') as f:
                tool_data = json.load(f)
            
            tool_name = tool_data.get('metadata', {}).get('name', Path(tool_path).stem)
            schema_version = tool_data.get('metadata', {}).get('version', 'unknown')
            
            # Validate schema
            schema_validation = self.validate_tool_schema(tool_path)
            if not schema_validation["valid"]:
                return {
                    "success": False,
                    "message": f"Schema validation failed: {', '.join(schema_validation['issues'])}",
                    "data": {
                        "tool_name": tool_name,
                        "issues": schema_validation["issues"]
                    }
                }
            
            # Validate dates
            date_validation = self.validate_tool_dates(tool_path)
            if not date_validation["valid"]:
                return {
                    "success": False,
                    "message": f"Date validation failed: {', '.join(date_validation['issues'])}",
                    "data": {
                        "tool_name": tool_name,
                        "issues": date_validation["issues"]
                    }
                }
            
            # Calculate hashes
            schema_hash = self.calculate_hash(json.dumps(tool_data, sort_keys=True))
            metadata_hash = self.calculate_hash(json.dumps(tool_data.get('metadata', {}), sort_keys=True))
            capabilities_hash = self.calculate_hash(json.dumps(tool_data.get('spec', {}).get('capabilities', {}), sort_keys=True))
            
            # Register in database
            with sqlite3.connect(self.db_path) as conn:
                # Check if tool already exists
                existing = conn.execute(
                    "SELECT tool_name FROM tools WHERE tool_name = ?",
                    (tool_name,)
                ).fetchone()
                
                if existing:
                    # Update existing tool
                    conn.execute("""
                        UPDATE tools SET
                            tool_path = ?,
                            schema_version = ?,
                            last_validated = ?,
                            validation_status = ?,
                            schema_hash = ?,
                            metadata_hash = ?,
                            capabilities_hash = ?
                        WHERE tool_name = ?
                    """, (
                        tool_path,
                        schema_version,
                        self.get_canonical_datetime(),
                        'validated',
                        schema_hash,
                        metadata_hash,
                        capabilities_hash,
                        tool_name
                    ))
                    action = "updated"
                else:
                    # Insert new tool
                    conn.execute("""
                        INSERT INTO tools 
                        (tool_name, tool_path, schema_version, registration_date, last_validated, validation_status, schema_hash, metadata_hash, capabilities_hash)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, (
                        tool_name,
                        tool_path,
                        schema_version,
                        self.get_canonical_datetime(),
                        self.get_canonical_datetime(),
                        'validated',
                        schema_hash,
                        metadata_hash,
                        capabilities_hash
                    ))
                    action = "registered"
                
                # Log audit
                conn.execute("""
                    INSERT INTO tool_audit (tool_name, action, timestamp, details)
                    VALUES (?, ?, ?, ?)
                """, (
                    tool_name,
                    action,
                    self.get_canonical_datetime(),
                    json.dumps({
                        "tool_path": tool_path,
                        "schema_version": schema_version,
                        "validation_status": "validated"
                    })
                ))
                
                conn.commit()
            
            return {
                "success": True,
                "message": f"Tool {tool_name} {action} successfully",
                "data": {
                    "tool_name": tool_name,
                    "tool_path": tool_path,
                    "schema_version": schema_version,
                    "registration_date": self.get_canonical_datetime(),
                    "validation_status": "validated"
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to register tool: {e}",
                "data": {}
            }
    
    def list_tools(self, output_format: str = "json") -> Dict[str, Any]:
        """List all registered tools."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                tools = conn.execute("""
                    SELECT tool_name, tool_path, schema_version, registration_date, validation_status, last_validated
                    FROM tools
                    ORDER BY registration_date DESC
                """).fetchall()
            
            tool_list = [
                {
                    "tool_name": tool[0],
                    "tool_path": tool[1],
                    "schema_version": tool[2],
                    "registration_date": tool[3],
                    "validation_status": tool[4],
                    "last_validated": tool[5]
                }
                for tool in tools
            ]
            
            return {
                "success": True,
                "message": f"Found {len(tool_list)} registered tools",
                "data": {
                    "tools": tool_list,
                    "total": len(tool_list)
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to list tools: {e}",
                "data": {}
            }
    
    def validate_tool(self, tool_name: str) -> Dict[str, Any]:
        """Validate a specific tool."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                tool = conn.execute("""
                    SELECT tool_path, schema_version FROM tools WHERE tool_name = ?
                """, (tool_name,)).fetchone()
            
            if not tool:
                return {
                    "success": False,
                    "message": f"Tool not found: {tool_name}",
                    "data": {}
                }
            
            tool_path = tool[0]
            schema_version = tool[1]
            
            # Validate schema
            schema_validation = self.validate_tool_schema(tool_path)
            date_validation = self.validate_tool_dates(tool_path)
            
            all_issues = schema_validation["issues"] + date_validation["issues"]
            is_valid = schema_validation["valid"] and date_validation["valid"]
            
            # Update validation status
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("""
                    UPDATE tools SET
                        last_validated = ?,
                        validation_status = ?
                    WHERE tool_name = ?
                """, (
                    self.get_canonical_datetime(),
                    'validated' if is_valid else 'invalid',
                    tool_name
                ))
                
                # Log validation history
                conn.execute("""
                    INSERT INTO validation_history 
                    (tool_name, validation_date, validation_status, issues, schema_version)
                    VALUES (?, ?, ?, ?, ?)
                """, (
                    tool_name,
                    self.get_canonical_datetime(),
                    'validated' if is_valid else 'invalid',
                    json.dumps(all_issues),
                    schema_version
                ))
                
                conn.commit()
            
            return {
                "success": True,
                "message": f"Tool {tool_name} validation completed",
                "data": {
                    "tool_name": tool_name,
                    "valid": is_valid,
                    "issues": all_issues,
                    "validation_date": self.get_canonical_datetime()
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to validate tool: {e}",
                "data": {}
            }
    
    def audit_database(self) -> Dict[str, Any]:
        """Audit the tool registry database."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                # Get tool statistics
                tool_stats = conn.execute("""
                    SELECT 
                        COUNT(*) as total_tools,
                        COUNT(CASE WHEN validation_status = 'validated' THEN 1 END) as valid_tools,
                        COUNT(CASE WHEN validation_status = 'invalid' THEN 1 END) as invalid_tools,
                        COUNT(CASE WHEN validation_status = 'pending' THEN 1 END) as pending_tools
                    FROM tools
                """).fetchone()
                
                # Get recent audit entries
                recent_audit = conn.execute("""
                    SELECT tool_name, action, timestamp, details
                    FROM tool_audit
                    ORDER BY timestamp DESC
                    LIMIT 10
                """).fetchall()
                
                # Get validation history
                validation_history = conn.execute("""
                    SELECT tool_name, validation_date, validation_status, issues
                    FROM validation_history
                    ORDER BY validation_date DESC
                    LIMIT 10
                """).fetchall()
            
            return {
                "success": True,
                "message": "Audit completed successfully",
                "data": {
                    "tool_stats": {
                        "total_tools": tool_stats[0],
                        "valid_tools": tool_stats[1],
                        "invalid_tools": tool_stats[2],
                        "pending_tools": tool_stats[3]
                    },
                    "recent_audit": [
                        {
                            "tool_name": entry[0],
                            "action": entry[1],
                            "timestamp": entry[2],
                            "details": json.loads(entry[3]) if entry[3] else {}
                        }
                        for entry in recent_audit
                    ],
                    "validation_history": [
                        {
                            "tool_name": entry[0],
                            "validation_date": entry[1],
                            "validation_status": entry[2],
                            "issues": json.loads(entry[3]) if entry[3] else []
                        }
                        for entry in validation_history
                    ]
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to audit database: {e}",
                "data": {}
            }


def main():
    """Main CLI interface for the tool registry."""
    parser = argparse.ArgumentParser(description="Tool Registry")
    parser.add_argument('command', choices=['register', 'list', 'validate', 'update', 'audit', 'export'],
                       help='Command to execute')
    parser.add_argument('--tool-path', '-p', help='Path to tool specification file')
    parser.add_argument('--tool-name', '-n', help='Tool name for operations')
    parser.add_argument('--output-format', '-o', choices=['json', 'text', 'csv'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    # Load configuration
    config = {
        'databaseFile': 'tool_registry.db',
        'schemaRoot': 'schemas',
        'validationStrict': True,
        'autoValidate': True,
        'backupEnabled': True
    }
    
    registry = ToolRegistry(config)
    
    try:
        if args.command == 'register':
            if not args.tool_path:
                print("Error: --tool-path is required for register command")
                sys.exit(1)
            
            result = registry.register_tool(args.tool_path)
            if args.output_format == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["success"]:
                    print(f"✅ {result['message']}")
                else:
                    print(f"❌ {result['message']}")
        
        elif args.command == 'list':
            result = registry.list_tools(args.output_format)
            if args.output_format == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["success"]:
                    tools = result["data"]["tools"]
                    print(f"📋 Registered Tools ({len(tools)}):")
                    for tool in tools:
                        status_icon = "✅" if tool["validation_status"] == "validated" else "❌"
                        print(f"  {status_icon} {tool['tool_name']} v{tool['schema_version']} ({tool['validation_status']})")
                else:
                    print(f"❌ {result['message']}")
        
        elif args.command == 'validate':
            if not args.tool_name:
                print("Error: --tool-name is required for validate command")
                sys.exit(1)
            
            result = registry.validate_tool(args.tool_name)
            if args.output_format == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["success"]:
                    data = result["data"]
                    if data["valid"]:
                        print(f"✅ {args.tool_name} is valid")
                    else:
                        print(f"❌ {args.tool_name} has issues:")
                        for issue in data["issues"]:
                            print(f"  - {issue}")
                else:
                    print(f"❌ {result['message']}")
        
        elif args.command == 'audit':
            result = registry.audit_database()
            if args.output_format == 'json':
                print(json.dumps(result, indent=2))
            else:
                if result["success"]:
                    stats = result["data"]["tool_stats"]
                    print(f"📊 Tool Registry Audit:")
                    print(f"  Total tools: {stats['total_tools']}")
                    print(f"  Valid tools: {stats['valid_tools']}")
                    print(f"  Invalid tools: {stats['invalid_tools']}")
                    print(f"  Pending tools: {stats['pending_tools']}")
                else:
                    print(f"❌ {result['message']}")
        
        else:
            print(f"Command '{args.command}' not implemented yet")
            sys.exit(1)
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 