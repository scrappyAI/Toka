#!/usr/bin/env python3
"""
Migration script to convert legacy rule and agent spec files to envelope format
"""

import json
import sys
from pathlib import Path
from typing import Dict, Any, List
import argparse

class EnvelopeMigrator:
    """Migrates legacy files to envelope format"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.rules_dir = workspace_root / '.cursor' / 'rules'
        self.agents_dir = workspace_root / 'agents'
    
    def migrate_rule_to_envelope(self, legacy_rule: Dict[str, Any]) -> Dict[str, Any]:
        """Convert legacy rule format to envelope format"""
        envelope = {
            "kind": "rule",
            "metadata": {
                "name": self._pascalcase_to_kebabcase(legacy_rule.get("name", "unknown-rule")),
                "version": legacy_rule.get("version", "1.0.0"),
                "title": legacy_rule.get("name", "Unknown Rule"),
                "description": legacy_rule.get("description", ""),
                "tags": self._convert_category_to_tags(legacy_rule.get("category", "")),
                "priority": self._convert_numeric_priority(legacy_rule.get("priority", 50))
            },
            "spec": {
                "always_apply": legacy_rule.get("always_apply", True),
                "extends": [],  # Will need to be updated manually
                "objectives": legacy_rule.get("objectives", []),
                "guidelines": legacy_rule.get("guidelines", {})
            }
        }
        
        # Add optional fields if present
        if "commit_conventions" in legacy_rule:
            envelope["spec"]["commit_conventions"] = legacy_rule["commit_conventions"]
        
        if "versioning" in legacy_rule:
            envelope["spec"]["versioning"] = legacy_rule["versioning"]
        
        if "validation" in legacy_rule:
            envelope["spec"]["validation"] = legacy_rule["validation"]
        
        # Copy metadata if it exists
        if "metadata" in legacy_rule:
            legacy_metadata = legacy_rule["metadata"]
            if "created" in legacy_metadata:
                envelope["metadata"]["created"] = legacy_metadata["created"]
            if "modified" in legacy_metadata:
                envelope["metadata"]["modified"] = legacy_metadata["modified"]
            if "checksum" in legacy_metadata:
                envelope["metadata"]["checksum"] = legacy_metadata["checksum"]
        
        return envelope
    
    def _pascalcase_to_kebabcase(self, name: str) -> str:
        """Convert PascalCase to kebab-case"""
        result = ""
        for i, char in enumerate(name):
            if char.isupper() and i > 0:
                result += "-"
            result += char.lower()
        return result
    
    def _convert_category_to_tags(self, category: str) -> List[str]:
        """Convert category to hierarchical tags"""
        if not category:
            return ["general"]
        
        # Map common categories to hierarchical tags
        category_map = {
            "core": ["system.core"],
            "security": ["security.base"],
            "testing": ["quality.testing"],
            "documentation": ["process.documentation"],
            "process": ["process.base"],
            "architecture": ["system.architecture"],
            "project": ["project.management"],
            "github": ["integration.github"],
            "ai-integration": ["integration.ai"]
        }
        
        return category_map.get(category, [f"category.{category}"])
    
    def _convert_numeric_priority(self, priority: int) -> str:
        """Convert numeric priority to string"""
        if priority >= 80:
            return "critical"
        elif priority >= 60:
            return "high"
        elif priority >= 40:
            return "medium"
        else:
            return "low"
    
    def migrate_file(self, file_path: Path) -> bool:
        """Migrate a single file to envelope format"""
        try:
            print(f"Processing file: {file_path}")
            with open(file_path, 'r') as f:
                data = json.load(f)
            
            # Check if already in envelope format
            if 'kind' in data:
                print(f"File {file_path} is already in envelope format")
                return True
            
            # Determine file type and migrate
            if 'guidelines' in data and 'name' in data:
                # This is a rule file
                migrated = self.migrate_rule_to_envelope(data)
                print(f"Migrated rule file: {file_path}")
            else:
                print(f"Unknown file format: {file_path}")
                return False
            
            # Write migrated file
            with open(file_path, 'w') as f:
                json.dump(migrated, f, indent=2, sort_keys=False)
            
            return True
            
        except Exception as e:
            print(f"Error migrating {file_path}: {e}")
            return False
    
    def migrate_all_rules(self) -> bool:
        """Migrate all rule files to envelope format"""
        success = True
        
        if not self.rules_dir.exists():
            print(f"Rules directory not found: {self.rules_dir}")
            return False
        
        for rule_file in self.rules_dir.glob('*.json'):
            if not self.migrate_file(rule_file):
                success = False
        
        return success

def main():
    parser = argparse.ArgumentParser(description='Migrate legacy files to envelope format')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(), help='Workspace root directory')
    parser.add_argument('--file', type=Path, help='Migrate specific file')
    parser.add_argument('--rules', action='store_true', help='Migrate all rule files')
    parser.add_argument('--all', action='store_true', help='Migrate all files')
    
    args = parser.parse_args()
    
    migrator = EnvelopeMigrator(args.workspace)
    success = True
    
    if args.file:
        success = migrator.migrate_file(args.file)
    elif args.rules or args.all:
        success = migrator.migrate_all_rules()
    else:
        print("No action specified. Use --help for options.")
        success = False
    
    if success:
        print("Migration completed successfully!")
    else:
        print("Migration failed!")
        sys.exit(1)

if __name__ == '__main__':
    main() 