#!/usr/bin/env python3
"""
Automatic Versioning System for Cursor Rules and Agent Specifications
Handles automatic version management, schema validation, and metadata updates
"""

import os
import sys
import json
import yaml
import hashlib
import datetime
import re
import argparse
import semantic_version
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum

class ChangeType(Enum):
    """Types of changes for version calculation"""
    MAJOR = "major"
    MINOR = "minor"
    PATCH = "patch"

@dataclass
class VersionInfo:
    """Version information structure"""
    version: str
    created: str
    modified: str
    schema_version: str
    checksum: str
    change_type: Optional[str] = None
    changelog: Optional[List[str]] = None

class SchemaValidator:
    """Schema validation for rules and agent specs"""
    
    def __init__(self, schema_dir: Path):
        self.schema_dir = schema_dir
        self.schemas = {}
        self._load_schemas()
    
    def _load_schemas(self):
        """Load all schema files"""
        try:
            import jsonschema
            self.jsonschema = jsonschema
        except ImportError:
            print("Warning: jsonschema not installed. Schema validation disabled.")
            self.jsonschema = None
            return
        
        schema_files = {
            'cursor-rule': self.schema_dir / 'cursor-rule-schema.yaml',
            'agent-spec': self.schema_dir / 'agent-spec-schema.yaml'
        }
        
        for schema_name, schema_path in schema_files.items():
            if schema_path.exists():
                with open(schema_path, 'r') as f:
                    self.schemas[schema_name] = yaml.safe_load(f)
    
    def validate_cursor_rule(self, rule_data: Dict) -> Tuple[bool, List[str]]:
        """Validate cursor rule against schema"""
        return self._validate_against_schema(rule_data, 'cursor-rule')
    
    def validate_agent_spec(self, spec_data: Dict) -> Tuple[bool, List[str]]:
        """Validate agent spec against schema"""
        return self._validate_against_schema(spec_data, 'agent-spec')
    
    def _validate_against_schema(self, data: Dict, schema_name: str) -> Tuple[bool, List[str]]:
        """Validate data against specified schema"""
        if not self.jsonschema or schema_name not in self.schemas:
            return True, []
        
        try:
            self.jsonschema.validate(data, self.schemas[schema_name])
            return True, []
        except self.jsonschema.ValidationError as e:
            return False, [str(e)]
        except Exception as e:
            return False, [f"Schema validation error: {str(e)}"]

class AutoVersionManager:
    """Automatic version management system"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.cursor_rules_dir = workspace_root / '.cursor' / 'rules'
        self.agent_specs_dir = workspace_root / 'agents'
        self.schema_dir = workspace_root / '.cursor' / 'schemas'
        self.version_db_path = workspace_root / '.cursor' / 'version_db.json'
        
        self.validator = SchemaValidator(self.schema_dir)
        self.version_db = self._load_version_db()
    
    def _load_version_db(self) -> Dict:
        """Load version database"""
        if self.version_db_path.exists():
            with open(self.version_db_path, 'r') as f:
                return json.load(f)
        return {}
    
    def _save_version_db(self):
        """Save version database"""
        self.version_db_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.version_db_path, 'w') as f:
            json.dump(self.version_db, f, indent=2)
    
    def _calculate_checksum(self, content: str) -> str:
        """Calculate SHA256 checksum of content"""
        return hashlib.sha256(content.encode()).hexdigest()[:16]
    
    def _determine_version_bump(self, file_path: Path, old_data: Dict, new_data: Dict) -> ChangeType:
        """Determine version bump type based on changes"""
        # Remove metadata for comparison
        old_compare = {k: v for k, v in old_data.items() if k != 'metadata'}
        new_compare = {k: v for k, v in new_data.items() if k != 'metadata'}
        
        # Check for breaking changes
        if self._has_breaking_changes(old_compare, new_compare):
            return ChangeType.MAJOR
        
        # Check for new features
        if self._has_new_features(old_compare, new_compare):
            return ChangeType.MINOR
        
        # Default to patch
        return ChangeType.PATCH
    
    def _has_breaking_changes(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if changes are breaking"""
        breaking_changes = [
            # Schema changes
            'required' in str(old_data) and 'required' not in str(new_data),
            # Priority changes in agent specs
            old_data.get('spec', {}).get('priority') != new_data.get('spec', {}).get('priority'),
            # Capability removals
            self._capabilities_removed(old_data, new_data),
            # Objective removals
            self._objectives_removed(old_data, new_data),
        ]
        return any(breaking_changes)
    
    def _has_new_features(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if changes add new features"""
        new_features = [
            # New capabilities
            self._capabilities_added(old_data, new_data),
            # New objectives
            self._objectives_added(old_data, new_data),
            # New guidelines
            self._guidelines_added(old_data, new_data),
        ]
        return any(new_features)
    
    def _capabilities_removed(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if capabilities were removed"""
        old_caps = set(old_data.get('capabilities', {}).get('primary', []))
        new_caps = set(new_data.get('capabilities', {}).get('primary', []))
        return len(old_caps - new_caps) > 0
    
    def _objectives_removed(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if objectives were removed"""
        old_objs = len(old_data.get('objectives', []))
        new_objs = len(new_data.get('objectives', []))
        return new_objs < old_objs
    
    def _capabilities_added(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if capabilities were added"""
        old_caps = set(old_data.get('capabilities', {}).get('primary', []))
        new_caps = set(new_data.get('capabilities', {}).get('primary', []))
        return len(new_caps - old_caps) > 0
    
    def _objectives_added(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if objectives were added"""
        old_objs = len(old_data.get('objectives', []))
        new_objs = len(new_data.get('objectives', []))
        return new_objs > old_objs
    
    def _guidelines_added(self, old_data: Dict, new_data: Dict) -> bool:
        """Check if guidelines were added"""
        old_guidelines = len(str(old_data.get('guidelines', {})))
        new_guidelines = len(str(new_data.get('guidelines', {})))
        return new_guidelines > old_guidelines
    
    def _bump_version(self, current_version: str, change_type: ChangeType) -> str:
        """Bump version based on change type"""
        if not current_version or current_version == "0.0.0":
            return "1.0.0"
        
        try:
            # Handle both vX.Y.Z and X.Y.Z formats
            version_str = current_version.lstrip('v')
            version = semantic_version.Version(version_str)
            
            if change_type == ChangeType.MAJOR:
                new_version = version.next_major()
            elif change_type == ChangeType.MINOR:
                new_version = version.next_minor()
            else:  # PATCH
                new_version = version.next_patch()
            
            return str(new_version)
        except Exception as e:
            print(f"Error bumping version {current_version}: {e}")
            return "1.0.0"
    
    def update_cursor_rule(self, rule_path: Path, auto_version: bool = True) -> bool:
        """Update cursor rule with automatic versioning"""
        try:
            with open(rule_path, 'r') as f:
                data = yaml.safe_load(f)
            
            if not data:
                print(f"Error: Empty or invalid YAML file: {rule_path}")
                return False
            
            # Validate against schema
            is_valid, errors = self.validator.validate_cursor_rule(data)
            if not is_valid:
                print(f"Schema validation failed for {rule_path}:")
                for error in errors:
                    print(f"  - {error}")
                return False
            
            # Calculate checksum
            content = yaml.dump(data, default_flow_style=False)
            checksum = self._calculate_checksum(content)
            
            # Get current version info
            file_key = str(rule_path.relative_to(self.workspace_root))
            current_info = self.version_db.get(file_key, {})
            
            # Check if file changed
            if current_info.get('checksum') == checksum:
                print(f"No changes detected in {rule_path}")
                return True
            
            # Determine version bump
            current_version = current_info.get('version', '0.0.0')
            if auto_version and current_info:
                # Load old data for comparison
                old_data = current_info.get('data', {})
                change_type = self._determine_version_bump(rule_path, old_data, data)
                new_version = self._bump_version(current_version, change_type)
            else:
                new_version = self._bump_version(current_version, ChangeType.PATCH)
            
            # Update metadata
            now = datetime.datetime.now(datetime.timezone.utc)
            if 'metadata' not in data:
                data['metadata'] = {}
            
            data['version'] = new_version
            data['metadata'].update({
                'created': current_info.get('created', now.date().isoformat()),
                'modified': now.isoformat(),
                'schema_version': '1.0.0',
                'checksum': checksum
            })
            
            # Save updated file
            with open(rule_path, 'w') as f:
                yaml.dump(data, f, default_flow_style=False, sort_keys=False)
            
            # Update version database
            self.version_db[file_key] = {
                'version': new_version,
                'created': data['metadata']['created'],
                'modified': data['metadata']['modified'],
                'schema_version': '1.0.0',
                'checksum': checksum,
                'data': data
            }
            
            print(f"Updated {rule_path}: v{current_version} -> v{new_version}")
            return True
            
        except Exception as e:
            print(f"Error updating cursor rule {rule_path}: {e}")
            return False
    
    def update_agent_spec(self, spec_path: Path, auto_version: bool = True) -> bool:
        """Update agent spec with automatic versioning"""
        try:
            with open(spec_path, 'r') as f:
                data = yaml.safe_load(f)
            
            if not data:
                print(f"Error: Empty or invalid YAML file: {spec_path}")
                return False
            
            # Validate against schema
            is_valid, errors = self.validator.validate_agent_spec(data)
            if not is_valid:
                print(f"Schema validation failed for {spec_path}:")
                for error in errors:
                    print(f"  - {error}")
                return False
            
            # Calculate checksum
            content = yaml.dump(data, default_flow_style=False)
            checksum = self._calculate_checksum(content)
            
            # Get current version info
            file_key = str(spec_path.relative_to(self.workspace_root))
            current_info = self.version_db.get(file_key, {})
            
            # Check if file changed
            if current_info.get('checksum') == checksum:
                print(f"No changes detected in {spec_path}")
                return True
            
            # Determine version bump
            current_version = current_info.get('version', 'v0.0.0')
            if auto_version and current_info:
                # Load old data for comparison
                old_data = current_info.get('data', {})
                change_type = self._determine_version_bump(spec_path, old_data, data)
                new_version = self._bump_version(current_version.lstrip('v'), change_type)
                new_version = f"v{new_version}"
            else:
                new_version = f"v{self._bump_version(current_version.lstrip('v'), ChangeType.PATCH)}"
            
            # Update metadata
            now = datetime.datetime.now(datetime.timezone.utc)
            if 'metadata' not in data:
                data['metadata'] = {}
            
            data['metadata'].update({
                'version': new_version,
                'created': current_info.get('created', now.date().isoformat()),
                'modified': now.isoformat(),
                'schema_version': '1.0.0',
                'checksum': checksum
            })
            
            # Save updated file
            with open(spec_path, 'w') as f:
                yaml.dump(data, f, default_flow_style=False, sort_keys=False)
            
            # Update version database
            self.version_db[file_key] = {
                'version': new_version,
                'created': data['metadata']['created'],
                'modified': data['metadata']['modified'],
                'schema_version': '1.0.0',
                'checksum': checksum,
                'data': data
            }
            
            print(f"Updated {spec_path}: {current_version} -> {new_version}")
            return True
            
        except Exception as e:
            print(f"Error updating agent spec {spec_path}: {e}")
            return False
    
    def update_all_rules(self, auto_version: bool = True) -> bool:
        """Update all cursor rules"""
        success = True
        if self.cursor_rules_dir.exists():
            for rule_file in self.cursor_rules_dir.glob('*.yaml'):
                if not self.update_cursor_rule(rule_file, auto_version):
                    success = False
        
        self._save_version_db()
        return success
    
    def update_all_agent_specs(self, auto_version: bool = True) -> bool:
        """Update all agent specifications"""
        success = True
        if self.agent_specs_dir.exists():
            for spec_file in self.agent_specs_dir.rglob('*.yaml'):
                if not self.update_agent_spec(spec_file, auto_version):
                    success = False
        
        self._save_version_db()
        return success
    
    def update_all(self, auto_version: bool = True) -> bool:
        """Update all files"""
        rules_success = self.update_all_rules(auto_version)
        specs_success = self.update_all_agent_specs(auto_version)
        return rules_success and specs_success

def main():
    parser = argparse.ArgumentParser(description='Automatic versioning for cursor rules and agent specs')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(), help='Workspace root directory')
    parser.add_argument('--file', type=Path, help='Update specific file')
    parser.add_argument('--rules', action='store_true', help='Update all cursor rules')
    parser.add_argument('--specs', action='store_true', help='Update all agent specs')
    parser.add_argument('--all', action='store_true', help='Update all files')
    parser.add_argument('--no-auto-version', action='store_true', help='Disable automatic version bumping')
    parser.add_argument('--validate-only', action='store_true', help='Only validate, don\'t update')
    
    args = parser.parse_args()
    
    manager = AutoVersionManager(args.workspace)
    auto_version = not args.no_auto_version
    
    if args.validate_only:
        print("Validation mode - no files will be updated")
        auto_version = False
    
    success = True
    
    if args.file:
        if args.file.suffix == '.yaml':
            if 'rules' in str(args.file):
                success = manager.update_cursor_rule(args.file, auto_version)
            elif 'agents' in str(args.file):
                success = manager.update_agent_spec(args.file, auto_version)
            else:
                print(f"Unknown file type: {args.file}")
                success = False
        else:
            print(f"Only YAML files are supported: {args.file}")
            success = False
    elif args.rules:
        success = manager.update_all_rules(auto_version)
    elif args.specs:
        success = manager.update_all_agent_specs(auto_version)
    elif args.all:
        success = manager.update_all(auto_version)
    else:
        print("No action specified. Use --help for options.")
        success = False
    
    if not success:
        sys.exit(1)
    
    print("Version management completed successfully!")

if __name__ == '__main__':
    main() 