#!/usr/bin/env python3
"""
Comprehensive Date Validation

Integrates date enforcer, tool registry, and schema contracts to ensure
deterministic, accurate dating across the entire codebase.
"""

import json
import os
import sys
import argparse
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Any, List


class DateValidator:
    """Comprehensive date validation system."""
    
    def __init__(self, contract_path: str = "date-contract.json"):
        """Initialize the date validator with contract."""
        self.contract_path = Path(contract_path)
        self.contract = self._load_contract()
        
    def _load_contract(self) -> Dict[str, Any]:
        """Load the date contract."""
        if not self.contract_path.exists():
            raise FileNotFoundError(f"Date contract not found: {self.contract_path}")
        
        with open(self.contract_path, 'r') as f:
            contract = json.load(f)
        
        return contract.get('dateContract', {})
    
    def get_canonical_date(self) -> str:
        """Get the canonical current date."""
        format_str = self.contract.get('enforcement', {}).get('format', '%Y-%m-%d')
        return datetime.now(timezone.utc).strftime(format_str)
    
    def get_canonical_datetime(self) -> str:
        """Get the canonical current datetime."""
        format_str = self.contract.get('enforcement', {}).get('datetimeFormat', '%Y-%m-%dT%H:%M:%SZ')
        return datetime.now(timezone.utc).strftime(format_str)
    
    def replace_placeholders(self, content: str) -> str:
        """Replace date placeholders with canonical dates."""
        placeholder = self.contract.get('validation', {}).get('placeholderPattern', '2025-07-07')
        
        # Replace date placeholders
        content = content.replace(placeholder, self.get_canonical_date())
        
        # Replace datetime placeholders
        datetime_placeholder = '2025-07-07T07:09:12Z'
        content = content.replace(datetime_placeholder, self.get_canonical_datetime())
        
        return content
    
    def validate_file_dates(self, file_path: str) -> Dict[str, Any]:
        """Validate dates in a single file."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Replace placeholders
            updated_content = self.replace_placeholders(content)
            
            # Check if content changed
            if content != updated_content:
                # Write updated content
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(updated_content)
                
                return {
                    "valid": True,
                    "corrected": True,
                    "message": f"Updated date placeholders in {file_path}",
                    "changes": {
                        "file": file_path,
                        "timestamp": self.get_canonical_datetime()
                    }
                }
            
            return {
                "valid": True,
                "corrected": False,
                "message": f"No date issues found in {file_path}"
            }
            
        except Exception as e:
            return {
                "valid": False,
                "corrected": False,
                "message": f"Error processing {file_path}: {e}"
            }
    
    def run_date_enforcer(self, command: str, args: List[str] = None) -> Dict[str, Any]:
        """Run the date enforcer tool."""
        enforcer_path = self.contract.get('tools', {}).get('dateEnforcer', 'scripts/date-enforcer.py')
        
        cmd = ['python3', enforcer_path, command]
        if args:
            cmd.extend(args)
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=Path.cwd())
            
            if result.returncode == 0:
                try:
                    return json.loads(result.stdout)
                except json.JSONDecodeError:
                    return {
                        "success": True,
                        "message": result.stdout.strip(),
                        "data": {}
                    }
            else:
                return {
                    "success": False,
                    "message": result.stderr.strip(),
                    "data": {}
                }
                
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to run date enforcer: {e}",
                "data": {}
            }
    
    def run_tool_registry(self, command: str, args: List[str] = None) -> Dict[str, Any]:
        """Run the tool registry."""
        registry_path = self.contract.get('tools', {}).get('toolRegistry', 'scripts/tool-registry.py')
        
        cmd = ['python3', registry_path, command]
        if args:
            cmd.extend(args)
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=Path.cwd())
            
            if result.returncode == 0:
                try:
                    return json.loads(result.stdout)
                except json.JSONDecodeError:
                    return {
                        "success": True,
                        "message": result.stdout.strip(),
                        "data": {}
                    }
            else:
                return {
                    "success": False,
                    "message": result.stderr.strip(),
                    "data": {}
                }
                
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to run tool registry: {e}",
                "data": {}
            }
    
    def validate_all_dates(self, directory: str = ".") -> Dict[str, Any]:
        """Validate all dates in the directory."""
        results = {
            "valid": True,
            "files_processed": 0,
            "files_corrected": 0,
            "errors": [],
            "details": []
        }
        
        directory_path = Path(directory)
        patterns = ['*.json', '*.md', '*.py', '*.rs', '*.yml', '*.yaml']
        
        for pattern in patterns:
            for file_path in directory_path.rglob(pattern):
                # Skip certain directories
                if any(part.startswith('.') for part in file_path.parts):
                    continue
                
                results["files_processed"] += 1
                result = self.validate_file_dates(str(file_path))
                
                if not result["valid"]:
                    results["valid"] = False
                    results["errors"].append(result["message"])
                
                if result.get("corrected", False):
                    results["files_corrected"] += 1
                
                results["details"].append(result)
        
        return results
    
    def register_all_tools(self, tools_directory: str = "tools") -> Dict[str, Any]:
        """Register all tools in the tools directory."""
        results = {
            "success": True,
            "tools_registered": 0,
            "tools_failed": 0,
            "details": []
        }
        
        tools_path = Path(tools_directory)
        if not tools_path.exists():
            return {
                "success": False,
                "message": f"Tools directory not found: {tools_directory}",
                "data": {}
            }
        
        for tool_file in tools_path.glob("*.json"):
            result = self.run_tool_registry("register", ["--tool-path", str(tool_file)])
            
            if result["success"]:
                results["tools_registered"] += 1
            else:
                results["tools_failed"] += 1
                results["success"] = False
            
            results["details"].append({
                "file": str(tool_file),
                "result": result
            })
        
        return results
    
    def comprehensive_validation(self) -> Dict[str, Any]:
        """Run comprehensive date validation."""
        print("🔍 Starting comprehensive date validation...")
        
        results = {
            "timestamp": self.get_canonical_datetime(),
            "contract_version": self.contract.get('version', 'unknown'),
            "valid": True,
            "steps": []
        }
        
        # Step 1: Update date contract with current dates
        print("📅 Updating date contract...")
        contract_result = self.validate_file_dates(self.contract_path)
        results["steps"].append({
            "step": "update_contract",
            "result": contract_result
        })
        
        # Step 2: Validate all dates in the codebase
        print("🔍 Validating all dates...")
        validation_result = self.validate_all_dates()
        results["steps"].append({
            "step": "validate_dates",
            "result": validation_result
        })
        
        if not validation_result["valid"]:
            results["valid"] = False
        
        # Step 3: Register all tools
        print("🔧 Registering tools...")
        registration_result = self.register_all_tools()
        results["steps"].append({
            "step": "register_tools",
            "result": registration_result
        })
        
        if not registration_result["success"]:
            results["valid"] = False
        
        # Step 4: Run date enforcer audit
        print("📊 Running date audit...")
        audit_result = self.run_date_enforcer("audit")
        results["steps"].append({
            "step": "date_audit",
            "result": audit_result
        })
        
        # Step 5: Run tool registry audit
        print("📋 Running tool registry audit...")
        registry_audit = self.run_tool_registry("audit")
        results["steps"].append({
            "step": "tool_audit",
            "result": registry_audit
        })
        
        return results


def main():
    """Main CLI interface for comprehensive date validation."""
    parser = argparse.ArgumentParser(description="Comprehensive Date Validation")
    parser.add_argument('--command', '-c', choices=['validate', 'register', 'audit', 'comprehensive'],
                       default='comprehensive', help='Command to execute')
    parser.add_argument('--directory', '-d', default='.', help='Directory to validate')
    parser.add_argument('--tools-dir', '-t', default='tools', help='Tools directory')
    parser.add_argument('--contract', default='date-contract.json', help='Date contract file')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    try:
        validator = DateValidator(args.contract)
        
        if args.command == 'validate':
            result = validator.validate_all_dates(args.directory)
            
        elif args.command == 'register':
            result = validator.register_all_tools(args.tools_dir)
            
        elif args.command == 'audit':
            date_audit = validator.run_date_enforcer("audit")
            tool_audit = validator.run_tool_registry("audit")
            result = {
                "date_audit": date_audit,
                "tool_audit": tool_audit
            }
            
        elif args.command == 'comprehensive':
            result = validator.comprehensive_validation()
        
        if args.output == 'json':
            print(json.dumps(result, indent=2))
        else:
            if args.command == 'comprehensive':
                print(f"📊 Comprehensive Date Validation Results:")
                print(f"  Timestamp: {result['timestamp']}")
                print(f"  Contract Version: {result['contract_version']}")
                print(f"  Overall Status: {'✅ Valid' if result['valid'] else '❌ Invalid'}")
                
                for step in result['steps']:
                    step_name = step['step'].replace('_', ' ').title()
                    step_result = step['result']
                    
                    if step_result.get('valid', True):
                        print(f"  ✅ {step_name}: Success")
                    else:
                        print(f"  ❌ {step_name}: Failed")
                        if 'message' in step_result:
                            print(f"    Error: {step_result['message']}")
            else:
                if result.get('valid', True):
                    print("✅ Validation completed successfully")
                else:
                    print("❌ Validation failed")
                    if 'message' in result:
                        print(f"Error: {result['message']}")
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 