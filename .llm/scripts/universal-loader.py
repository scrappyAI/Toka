#!/usr/bin/env python3
"""
Universal Extension Loader

This script provides agnostic loading of extension configurations from any
extension directory (.llm, .agent, .cursor, etc.) and manages the universal
schema system.
"""

import json
import os
import sys
import argparse
from pathlib import Path
from typing import Dict, Any, Optional, List
import jsonschema
from jsonschema import RefResolver


class UniversalExtensionLoader:
    """Universal loader for extension configurations and schemas."""
    
    def __init__(self, extension_root: Optional[str] = None):
        """Initialize the loader with an extension root directory."""
        self.extension_root = Path(extension_root) if extension_root else self._find_extension_root()
        self.config: Optional[Dict[str, Any]] = None
        self.schemas: Dict[str, Any] = {}
        self.resolver: Optional[RefResolver] = None
        
    def _find_extension_root(self) -> Path:
        """Find the extension root directory by looking for common patterns."""
        current = Path.cwd()
        
        # Common extension directory names
        extension_dirs = ['.llm', '.agent', '.cursor', '.ai', '.schema']
        
        for ext_dir in extension_dirs:
            ext_path = current / ext_dir
            if ext_path.exists() and ext_path.is_dir():
                return ext_path
                
        # If we're already in an extension directory, use current
        if current.name.startswith('.'):
            return current
                
        # If no extension directory found, default to .llm
        return current / '.llm'
    
    def load_config(self) -> Dict[str, Any]:
        """Load the extension configuration."""
        config_file = self.extension_root / "extension-config.json"
        
        if not config_file.exists():
            raise FileNotFoundError(f"Extension config not found: {config_file}")
            
        with open(config_file, 'r') as f:
            self.config = json.load(f)
            
        return self.config or {}
    
    def load_schemas(self) -> Dict[str, Any]:
        """Load all schemas from the schema root."""
        if not self.config:
            raise RuntimeError("Config must be loaded before loading schemas")
            
        schema_root = self.extension_root / self.config.get('schemaRoot', 'schemas')
        
        if not schema_root.exists():
            raise FileNotFoundError(f"Schema root not found: {schema_root}")
            
        # Load all JSON schema files
        for schema_file in schema_root.glob("*.json"):
            try:
                with open(schema_file, 'r') as f:
                    schema_data = json.load(f)
                    schema_id = schema_data.get('$id', schema_file.name)
                    self.schemas[schema_id] = schema_data
            except Exception as e:
                print(f"Warning: Failed to load schema {schema_file}: {e}")
                
        return self.schemas
    
    def setup_resolver(self) -> RefResolver:
        """Set up the JSON schema resolver with all loaded schemas."""
        if not self.schemas:
            self.load_schemas()
            
        # Create resolver with all schemas
        self.resolver = RefResolver.from_schema(
            self.schemas.get(self.config.get('defaultSchema', 'resource-envelope-v1.json')),
            store=self.schemas
        )
        
        return self.resolver
    
    def validate_resource(self, resource_data: Dict[str, Any], kind: str = None) -> Dict[str, Any]:
        """Validate a resource against the appropriate schema."""
        if not self.resolver:
            self.setup_resolver()
            
        # Determine the schema to use
        if kind:
            schema_key = f"{kind}-schema.json"
        else:
            schema_key = resource_data.get('$schema', self.config.get('defaultSchema'))
            
        if schema_key not in self.schemas:
            raise ValueError(f"Schema not found: {schema_key}")
            
        schema = self.schemas[schema_key]
        
        try:
            jsonschema.validate(resource_data, schema, resolver=self.resolver)
            return {"valid": True, "errors": []}
        except jsonschema.ValidationError as e:
            return {
                "valid": False,
                "errors": [{
                    "path": "/".join(str(p) for p in e.path),
                    "message": e.message,
                    "severity": "error"
                }]
            }
    
    def list_resources(self, kind: str = None) -> List[Dict[str, Any]]:
        """List all resources of a specific kind."""
        rule_root = self.extension_root / self.config.get('ruleRoot', 'rules')
        tool_root = self.extension_root / self.config.get('toolRoot', 'tools')
        agent_root = self.extension_root / "agents"
        
        resources = []
        
        # Map kinds to directories
        kind_dirs = {
            'rule': rule_root,
            'tool': tool_root,
            'agent': agent_root
        }
        
        if kind and kind in kind_dirs:
            dir_path = kind_dirs[kind]
            if dir_path.exists():
                for resource_file in dir_path.glob("*.json"):
                    try:
                        with open(resource_file, 'r') as f:
                            resource_data = json.load(f)
                            resources.append({
                                'file': str(resource_file),
                                'name': resource_data.get('metadata', {}).get('name', resource_file.stem),
                                'version': resource_data.get('metadata', {}).get('version', 'unknown'),
                                'kind': resource_data.get('kind', kind)
                            })
                    except Exception as e:
                        print(f"Warning: Failed to load resource {resource_file}: {e}")
        else:
            # List all resources
            for kind_name, dir_path in kind_dirs.items():
                if dir_path.exists():
                    for resource_file in dir_path.glob("*.json"):
                        try:
                            with open(resource_file, 'r') as f:
                                resource_data = json.load(f)
                                resources.append({
                                    'file': str(resource_file),
                                    'name': resource_data.get('metadata', {}).get('name', resource_file.stem),
                                    'version': resource_data.get('metadata', {}).get('version', 'unknown'),
                                    'kind': resource_data.get('kind', kind_name)
                                })
                        except Exception as e:
                            print(f"Warning: Failed to load resource {resource_file}: {e}")
                            
        return resources
    
    def get_extension_info(self) -> Dict[str, Any]:
        """Get information about the loaded extension."""
        return {
            'name': self.config.get('name'),
            'version': self.config.get('version'),
            'description': self.config.get('description'),
            'extensionRoot': str(self.extension_root),
            'schemaRoot': self.config.get('schemaRoot'),
            'ruleRoot': self.config.get('ruleRoot'),
            'toolRoot': self.config.get('toolRoot'),
            'supportedKinds': self.config.get('supportedKinds', []),
            'defaultSchema': self.config.get('defaultSchema'),
            'loadedSchemas': len(self.schemas),
            'loadedResources': len(self.list_resources())
        }


def main():
    """Main CLI interface for the universal loader."""
    parser = argparse.ArgumentParser(description="Universal Extension Loader")
    parser.add_argument('--extension-root', '-r', help='Extension root directory')
    parser.add_argument('--command', '-c', required=True, 
                       choices=['info', 'validate', 'list', 'load'],
                       help='Command to execute')
    parser.add_argument('--file', '-f', help='File to validate or load')
    parser.add_argument('--kind', '-k', help='Resource kind for validation')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    try:
        loader = UniversalExtensionLoader(args.extension_root)
        loader.load_config()
        
        if args.command == 'info':
            info = loader.get_extension_info()
            if args.output == 'json':
                print(json.dumps(info, indent=2))
            else:
                print(f"Extension: {info['name']} v{info['version']}")
                print(f"Description: {info['description']}")
                print(f"Root: {info['extensionRoot']}")
                print(f"Supported Kinds: {', '.join(info['supportedKinds'])}")
                print(f"Loaded Schemas: {info['loadedSchemas']}")
                print(f"Loaded Resources: {info['loadedResources']}")
                
        elif args.command == 'validate':
            if not args.file:
                print("Error: --file is required for validate command")
                sys.exit(1)
                
            with open(args.file, 'r') as f:
                resource_data = json.load(f)
                
            result = loader.validate_resource(resource_data, args.kind)
            print(json.dumps(result, indent=2))
            
        elif args.command == 'list':
            resources = loader.list_resources(args.kind)
            if args.output == 'json':
                print(json.dumps(resources, indent=2))
            else:
                for resource in resources:
                    print(f"{resource['kind']}: {resource['name']} v{resource['version']} ({resource['file']})")
                    
        elif args.command == 'load':
            loader.load_schemas()
            loader.setup_resolver()
            print("Schemas loaded successfully")
            
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 