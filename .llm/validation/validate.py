#!/usr/bin/env python3
"""
Strict Schema Validation System

Prevents LLMs from generating invalid schemas or tools by enforcing
strict validation against general schemas with detailed error reporting.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timezone
import jsonschema
from jsonschema import validate, ValidationError, Draft7Validator
import click


@dataclass
class ValidationResult:
    """Result of schema validation."""
    valid: bool
    errors: List[str]
    warnings: List[str]
    schema_type: str
    resource_name: Optional[str] = None


class SchemaValidator:
    """Strict schema validator for LLM-generated resources."""
    
    def __init__(self, schemas_dir: str = "schemas"):
        """Initialize validator with schema directory."""
        self.schemas_dir = Path(schemas_dir)
        self.schemas = {}
        self.validation_rules = {}
        self._load_schemas()
        self._load_validation_rules()
    
    def _load_schemas(self) -> None:
        """Load all general schemas."""
        schema_files = {
            'agent': 'agent.json',
            'tool': 'tool.json',
            'policy': 'policy.json',
            'contract': 'contract.json'
        }
        
        for schema_type, filename in schema_files.items():
            schema_path = self.schemas_dir / filename
            if schema_path.exists():
                with open(schema_path, 'r') as f:
                    self.schemas[schema_type] = json.load(f)
                print(f"📋 Loaded {schema_type} schema from {filename}")
            else:
                print(f"⚠️  Schema file not found: {schema_path}")
    
    def _load_validation_rules(self) -> None:
        """Load validation rules."""
        rules_path = Path("validation/rules.json")
        if rules_path.exists():
            with open(rules_path, 'r') as f:
                self.validation_rules = json.load(f)
            print(f"📋 Loaded validation rules from {rules_path}")
        else:
            # Default rules if file doesn't exist
            self.validation_rules = {
                "agent_generation": {
                    "max_capabilities": 10,
                    "required_fields": ["metadata", "spec"],
                    "forbidden_capabilities": ["root-access", "network-unrestricted"]
                },
                "tool_generation": {
                    "max_memory": "1GB",
                    "max_timeout": "10m",
                    "required_security": ["sandbox", "capability-restricted"]
                }
            }
            print("📋 Using default validation rules")
    
    def validate_agent(self, agent_data: Dict[str, Any]) -> ValidationResult:
        """Validate agent against general agent schema."""
        return self._validate_resource(agent_data, 'agent')
    
    def validate_tool(self, tool_data: Dict[str, Any]) -> ValidationResult:
        """Validate tool against general tool schema."""
        return self._validate_resource(tool_data, 'tool')
    
    def validate_policy(self, policy_data: Dict[str, Any]) -> ValidationResult:
        """Validate policy against general policy schema.""" 
        return self._validate_resource(policy_data, 'policy')
    
    def validate_contract(self, contract_data: Dict[str, Any]) -> ValidationResult:
        """Validate contract against general contract schema."""
        return self._validate_resource(contract_data, 'contract')
    
    def _validate_resource(self, data: Dict[str, Any], schema_type: str) -> ValidationResult:
        """Internal validation method."""
        if schema_type not in self.schemas:
            return ValidationResult(
                valid=False,
                errors=[f"Schema type '{schema_type}' not available"],
                warnings=[],
                schema_type=schema_type
            )
        
        errors = []
        warnings = []
        resource_name = None
        
        try:
            # Extract resource name for tracking
            if 'metadata' in data and 'name' in data['metadata']:
                resource_name = data['metadata']['name']
            
            # JSON Schema validation
            schema = self.schemas[schema_type]
            validator = Draft7Validator(schema)
            
            schema_errors = list(validator.iter_errors(data))
            if schema_errors:
                for error in schema_errors:
                    error_path = " -> ".join(str(p) for p in error.absolute_path)
                    errors.append(f"Schema error at {error_path}: {error.message}")
            
            # Additional business rule validation
            additional_errors, additional_warnings = self._validate_business_rules(data, schema_type)
            errors.extend(additional_errors)
            warnings.extend(additional_warnings)
            
        except Exception as e:
            errors.append(f"Validation failed: {str(e)}")
        
        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
            schema_type=schema_type,
            resource_name=resource_name
        )
    
    def _validate_business_rules(self, data: Dict[str, Any], schema_type: str) -> Tuple[List[str], List[str]]:
        """Validate business rules beyond JSON schema."""
        errors = []
        warnings = []
        
        if schema_type == 'agent':
            errors.extend(self._validate_agent_rules(data))
        elif schema_type == 'tool':
            errors.extend(self._validate_tool_rules(data))
        
        # Check for current date usage
        date_errors = self._validate_dates(data)
        if date_errors:
            warnings.extend(date_errors)
        
        return errors, warnings
    
    def _validate_agent_rules(self, data: Dict[str, Any]) -> List[str]:
        """Validate agent-specific business rules."""
        errors = []
        rules = self.validation_rules.get('agent_generation', {})
        
        # Check capability limits
        if 'spec' in data and 'capabilities' in data['spec']:
            capabilities = data['spec']['capabilities']
            max_caps = rules.get('max_capabilities', 10)
            if len(capabilities) > max_caps:
                errors.append(f"Too many capabilities ({len(capabilities)}), maximum is {max_caps}")
            
            # Check forbidden capabilities
            forbidden = rules.get('forbidden_capabilities', [])
            for cap in capabilities:
                if cap in forbidden:
                    errors.append(f"Forbidden capability: {cap}")
        
        # Check required fields
        required_fields = rules.get('required_fields', [])
        for field in required_fields:
            if field not in data:
                errors.append(f"Required field missing: {field}")
        
        return errors
    
    def _validate_tool_rules(self, data: Dict[str, Any]) -> List[str]:
        """Validate tool-specific business rules."""
        errors = []
        rules = self.validation_rules.get('tool_generation', {})
        
        # Check memory limits
        if 'spec' in data and 'performance' in data['spec']:
            perf = data['spec']['performance']
            if 'memory_limit' in perf:
                memory = perf['memory_limit']
                max_memory = rules.get('max_memory', '1GB')
                if not self._is_within_limit(memory, max_memory):
                    errors.append(f"Memory limit {memory} exceeds maximum {max_memory}")
        
        # Check security requirements
        if 'spec' in data and 'security' in data['spec']:
            security = data['spec']['security']
            required_security = rules.get('required_security', [])
            if 'level' in security:
                if security['level'] not in required_security and required_security:
                    errors.append(f"Security level must be one of: {required_security}")
        
        return errors
    
    def _validate_dates(self, data: Dict[str, Any]) -> List[str]:
        """Validate date usage to prevent hallucination."""
        warnings = []
        data_str = json.dumps(data)
        
        # Check for placeholder dates
        if '2025-07-07' in data_str:
            warnings.append("Contains placeholder date 2025-07-07 - should be replaced with actual date")
        
        # Check for hardcoded past dates (2024 and earlier)
        if '2024-' in data_str:
            warnings.append("Contains 2024 date reference - verify this is intentional")
        
        return warnings
    
    def _is_within_limit(self, value: str, limit: str) -> bool:
        """Check if a memory/time value is within limits."""
        # Simple implementation - could be enhanced
        try:
            # Extract numeric part and unit
            value_num = int(''.join(filter(str.isdigit, value)))
            limit_num = int(''.join(filter(str.isdigit, limit)))
            
            # Get units
            value_unit = ''.join(filter(str.isalpha, value)).upper()
            limit_unit = ''.join(filter(str.isalpha, limit)).upper()
            
            # Convert to same unit for comparison (simplified)
            if value_unit == limit_unit:
                return value_num <= limit_num
            
            # Could add more sophisticated unit conversion
            return True  # Default to allowing if units differ
        except:
            return True  # Default to allowing if parsing fails
    
    def reject_invalid(self, result: ValidationResult) -> None:
        """Strictly reject invalid schemas with detailed errors."""
        if not result.valid:
            print(f"\n❌ VALIDATION FAILED for {result.schema_type}")
            if result.resource_name:
                print(f"Resource: {result.resource_name}")
            
            print("\n🚨 ERRORS:")
            for i, error in enumerate(result.errors, 1):
                print(f"  {i}. {error}")
            
            if result.warnings:
                print("\n⚠️  WARNINGS:")
                for i, warning in enumerate(result.warnings, 1):
                    print(f"  {i}. {warning}")
            
            print(f"\n🚫 Resource REJECTED - Fix all errors before proceeding")
            sys.exit(1)
    
    def validate_file(self, file_path: str, schema_type: str) -> ValidationResult:
        """Validate a file against a schema."""
        try:
            with open(file_path, 'r') as f:
                data = json.load(f)
            
            if schema_type == 'agent':
                return self.validate_agent(data)
            elif schema_type == 'tool':
                return self.validate_tool(data)
            elif schema_type == 'policy':
                return self.validate_policy(data)
            elif schema_type == 'contract':
                return self.validate_contract(data)
            else:
                return ValidationResult(
                    valid=False,
                    errors=[f"Unknown schema type: {schema_type}"],
                    warnings=[],
                    schema_type=schema_type
                )
        except Exception as e:
            return ValidationResult(
                valid=False,
                errors=[f"Failed to load file {file_path}: {str(e)}"],
                warnings=[],
                schema_type=schema_type
            )
    
    def validate_directory(self, directory: str) -> Dict[str, ValidationResult]:
        """Validate all JSON files in a directory."""
        results = {}
        dir_path = Path(directory)
        
        if not dir_path.exists():
            print(f"❌ Directory not found: {directory}")
            return results
        
        # Map subdirectories to schema types
        schema_mapping = {
            'agents': 'agent',
            'tools': 'tool', 
            'policies': 'policy',
            'contracts': 'contract'
        }
        
        for json_file in dir_path.glob('**/*.json'):
            # Determine schema type from path
            schema_type = None
            for subdir, stype in schema_mapping.items():
                if subdir in json_file.parts:
                    schema_type = stype
                    break
            
            if schema_type:
                result = self.validate_file(str(json_file), schema_type)
                results[str(json_file)] = result
                
                if result.valid:
                    print(f"✅ {json_file.name} - Valid {schema_type}")
                else:
                    print(f"❌ {json_file.name} - Invalid {schema_type}")
            else:
                print(f"⚠️  {json_file.name} - Unknown schema type, skipping")
        
        return results


@click.command()
@click.argument('target', type=str)
@click.option('--schema-type', '-t', type=click.Choice(['agent', 'tool', 'policy', 'contract']), 
              help='Schema type to validate against')
@click.option('--directory', '-d', is_flag=True, help='Validate entire directory')
@click.option('--strict', '-s', is_flag=True, default=True, help='Use strict validation (default)')
@click.option('--report', '-r', type=str, help='Save validation report to file')
def main(target: str, schema_type: Optional[str], directory: bool, strict: bool, report: Optional[str]):
    """
    Strict schema validator for LLM-generated resources.
    
    TARGET: File or directory to validate
    """
    print("🔍 Toka Schema Validator")
    print("=" * 50)
    
    validator = SchemaValidator()
    
    if directory:
        # Validate entire directory
        results = validator.validate_directory(target)
        
        # Summary
        total = len(results)
        valid = sum(1 for r in results.values() if r.valid)
        invalid = total - valid
        
        print(f"\n📊 VALIDATION SUMMARY")
        print(f"Total files: {total}")
        print(f"Valid: {valid}")
        print(f"Invalid: {invalid}")
        
        if invalid > 0:
            print(f"\n❌ {invalid} files failed validation")
            if strict:
                sys.exit(1)
        else:
            print(f"\n✅ All files passed validation")
        
    else:
        # Validate single file
        if not schema_type:
            print("❌ Schema type required for single file validation")
            print("Use --schema-type agent|tool|policy|contract")
            sys.exit(1)
        
        result = validator.validate_file(target, schema_type)
        
        if result.valid:
            print(f"✅ {target} is valid {schema_type}")
            if result.warnings:
                print("\n⚠️  Warnings:")
                for warning in result.warnings:
                    print(f"  - {warning}")
        else:
            validator.reject_invalid(result)
    
    # Save report if requested
    if report and directory:
        report_data = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "summary": {
                "total": len(results),
                "valid": sum(1 for r in results.values() if r.valid),
                "invalid": sum(1 for r in results.values() if not r.valid)
            },
            "results": {
                path: {
                    "valid": result.valid,
                    "errors": result.errors,
                    "warnings": result.warnings,
                    "schema_type": result.schema_type
                }
                for path, result in results.items()
            }
        }
        
        with open(report, 'w') as f:
            json.dump(report_data, f, indent=2)
        print(f"\n📄 Validation report saved to {report}")


if __name__ == "__main__":
    main() 