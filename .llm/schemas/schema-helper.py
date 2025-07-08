#!/usr/bin/env python3
"""
Schema Helper Utilities
Provides utilities for managing schema files, including injecting $id fields
and validating schema consistency across the project.
"""

import os
import json
import re
from pathlib import Path
from typing import Dict, Any, Optional
import subprocess
import argparse

class SchemaHelper:
    """Helper utilities for schema management"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.schemas_dir = workspace_root / '.cursor' / 'schemas'
        self.github_repo_url = self._detect_github_repo()
    
    def _detect_github_repo(self) -> str:
        """Detect GitHub repository URL from git remote"""
        try:
            # Get remote origin URL
            result = subprocess.run(
                ['git', 'remote', 'get-url', 'origin'],
                cwd=self.workspace_root,
                capture_output=True,
                text=True,
                check=True
            )
            
            remote_url = result.stdout.strip()
            
            # Convert SSH URL to HTTPS if needed
            if remote_url.startswith('git@github.com:'):
                # git@github.com:owner/repo.git -> https://github.com/owner/repo
                repo_path = remote_url.replace('git@github.com:', '').replace('.git', '')
                return f"https://github.com/{repo_path}"
            elif remote_url.startswith('https://github.com/'):
                # Remove .git suffix if present
                return remote_url.replace('.git', '')
            else:
                print(f"Warning: Unrecognized remote URL format: {remote_url}")
                return "https://github.com/ScrappyAI/Toka"  # Default fallback
                
        except subprocess.CalledProcessError:
            print("Warning: Could not detect git remote, using default")
            return "https://github.com/ScrappyAI/Toka"  # Default fallback
    
    def inject_schema_id(self, schema_path: Path, schema_name: str) -> bool:
        """Inject $id field pointing to GitHub schema location"""
        try:
            with open(schema_path, 'r') as f:
                schema = json.load(f)
            
            # Generate $id URL
            schema_id = f"{self.github_repo_url}/blob/main/.cursor/schemas/{schema_name}"
            
            # Inject $id as second field (after $schema if present)
            new_schema = {}
            
            # Preserve $schema if it exists
            if '$schema' in schema:
                new_schema['$schema'] = schema['$schema']
            
            # Add $id
            new_schema['$id'] = schema_id
            
            # Add remaining fields
            for key, value in schema.items():
                if key not in ['$schema', '$id']:
                    new_schema[key] = value
            
            # Write back to file
            with open(schema_path, 'w') as f:
                json.dump(new_schema, f, indent=2, sort_keys=False)
            
            print(f"Injected $id into {schema_path}: {schema_id}")
            return True
            
        except Exception as e:
            print(f"Error injecting $id into {schema_path}: {e}")
            return False
    
    def inject_all_schema_ids(self) -> bool:
        """Inject $id fields into all schema files"""
        success = True
        
        schema_files = [
            ('resource-envelope-v1.json', 'resource-envelope-v1.json'),
            ('agent-spec-schema.json', 'agent-spec-schema.json'),
            ('cursor-rule-schema.json', 'cursor-rule-schema.json'),
            ('plan-schema.json', 'plan-schema.json'),
            ('event-schema.json', 'event-schema.json'),
        ]
        
        for schema_file, schema_name in schema_files:
            schema_path = self.schemas_dir / schema_file
            if schema_path.exists():
                if not self.inject_schema_id(schema_path, schema_name):
                    success = False
            else:
                print(f"Warning: Schema file not found: {schema_path}")
        
        return success
    
    def validate_schema_consistency(self) -> bool:
        """Validate that all schemas use consistent $defs and patterns"""
        # TODO: Implement schema consistency validation
        # This would check that all schemas reference the same $defs
        # and use consistent patterns for common fields
        return True
    
    def create_schema_index(self) -> Dict[str, Any]:
        """Create an index of all available schemas"""
        index = {
            "schemas": {},
            "kinds": [],
            "version": "1.0.0",
            "generated": "auto"
        }
        
        # Scan for schema files
        for schema_file in self.schemas_dir.glob('*.json'):
            if schema_file.name == 'schema-index.json':
                continue
                
            try:
                with open(schema_file, 'r') as f:
                    schema = json.load(f)
                
                schema_info = {
                    "file": schema_file.name,
                    "title": schema.get("title", ""),
                    "version": schema.get("version", ""),
                    "description": schema.get("description", ""),
                    "$id": schema.get("$id", "")
                }
                
                # Extract supported kinds from schema
                if "envelope" in schema_file.name:
                    # This is the envelope schema - extract kinds from enum
                    kind_enum = schema.get("properties", {}).get("kind", {}).get("enum", [])
                    index["kinds"] = kind_enum
                    schema_info["kinds"] = kind_enum
                
                index["schemas"][schema_file.stem] = schema_info
                
            except Exception as e:
                print(f"Error reading schema {schema_file}: {e}")
        
        return index
    
    def save_schema_index(self) -> bool:
        """Save schema index to file"""
        try:
            index = self.create_schema_index()
            index_path = self.schemas_dir / 'schema-index.json'
            
            with open(index_path, 'w') as f:
                json.dump(index, f, indent=2)
            
            print(f"Created schema index: {index_path}")
            return True
            
        except Exception as e:
            print(f"Error creating schema index: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(description='Schema helper utilities')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(), help='Workspace root directory')
    parser.add_argument('--inject-ids', action='store_true', help='Inject $id fields into all schemas')
    parser.add_argument('--create-index', action='store_true', help='Create schema index file')
    parser.add_argument('--validate', action='store_true', help='Validate schema consistency')
    parser.add_argument('--all', action='store_true', help='Run all operations')
    
    args = parser.parse_args()
    
    helper = SchemaHelper(args.workspace)
    success = True
    
    if args.inject_ids or args.all:
        print("Injecting $id fields...")
        if not helper.inject_all_schema_ids():
            success = False
    
    if args.create_index or args.all:
        print("Creating schema index...")
        if not helper.save_schema_index():
            success = False
    
    if args.validate or args.all:
        print("Validating schema consistency...")
        if not helper.validate_schema_consistency():
            success = False
    
    if not any([args.inject_ids, args.create_index, args.validate, args.all]):
        print("No action specified. Use --help for options.")
        success = False
    
    if success:
        print("Schema helper operations completed successfully!")
    else:
        print("Some operations failed!")
        return 1

if __name__ == '__main__':
    exit(main()) 