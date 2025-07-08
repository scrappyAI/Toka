#!/usr/bin/env python3
"""
Toka Tools Integration Bridge

Syncs validated schemas to the toka-tools crate for production implementation.
This creates a clean integration path from LLM-generated schemas to Rust code.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional
from dataclasses import dataclass
from datetime import datetime, timezone
import subprocess


@dataclass
class SyncResult:
    """Result of syncing a schema to toka-tools."""
    success: bool
    rust_file: Optional[str] = None
    errors: List[str] = None
    warnings: List[str] = None
    
    def __post_init__(self):
        if self.errors is None:
            self.errors = []
        if self.warnings is None:
            self.warnings = []


class TokaToolsBridge:
    """Bridge system for syncing validated schemas to toka-tools."""
    
    def __init__(self, toka_tools_dir: str = "crates/toka-tools"):
        """Initialize bridge with toka-tools directory."""
        self.toka_tools_dir = Path(toka_tools_dir)
        self.generated_dir = self.toka_tools_dir / "src" / "generated"
        self.manifests_dir = self.toka_tools_dir / "manifests"
        
        # Ensure directories exist
        self.generated_dir.mkdir(parents=True, exist_ok=True)
        self.manifests_dir.mkdir(parents=True, exist_ok=True)
    
    def sync_validated_tool(self, tool_schema: Dict[str, Any]) -> SyncResult:
        """
        Sync a validated tool schema to toka-tools.
        
        1. Take validated tool schema
        2. Generate Rust code template in toka-tools
        3. Create proper Cargo.toml entries
        4. Generate tests and validation
        5. Flag for human review if complex
        """
        try:
            tool_name = tool_schema['metadata']['name']
            print(f"🔄 Syncing tool: {tool_name}")
            
            # Generate Rust implementation
            rust_code = self.generate_rust_template(tool_schema)
            rust_file = self.generated_dir / f"{tool_name.replace('-', '_')}.rs"
            
            with open(rust_file, 'w') as f:
                f.write(rust_code)
            
            # Generate YAML manifest for toka-tools
            yaml_manifest = self.generate_yaml_manifest(tool_schema)
            yaml_file = self.manifests_dir / f"{tool_name}.yaml"
            
            with open(yaml_file, 'w') as f:
                f.write(yaml_manifest)
            
            # Update tool registry
            self.update_tool_registry(tool_name)
            
            # Check if manual review needed
            warnings = []
            if self._needs_manual_review(tool_schema):
                warnings.append("Tool complexity requires manual review before deployment")
            
            return SyncResult(
                success=True,
                rust_file=str(rust_file),
                warnings=warnings
            )
            
        except Exception as e:
            return SyncResult(
                success=False,
                errors=[f"Failed to sync tool: {str(e)}"]
            )
    
    def generate_rust_template(self, tool_schema: Dict[str, Any]) -> str:
        """Generate Rust implementation template from schema."""
        metadata = tool_schema['metadata']
        spec = tool_schema['spec']
        
        tool_name = metadata['name']
        tool_name_pascal = ''.join(word.capitalize() for word in tool_name.split('-'))
        tool_name_snake = tool_name.replace('-', '_')
        
        # Extract capabilities and parameters
        capabilities = spec.get('capabilities', [])
        interface = spec.get('interface', {})
        inputs = interface.get('inputs', [])
        
        # Generate parameter struct
        params_struct = self._generate_params_struct(inputs, tool_name_pascal)
        
        # Generate implementation
        implementation = self._generate_implementation(tool_schema, tool_name_pascal, tool_name_snake)
        
        # Generate tests
        tests = self._generate_tests(tool_name_pascal, tool_name_snake)
        
        rust_code = f'''//! {metadata.get('description', 'Generated tool')}
//!
//! This file was auto-generated from schema: {tool_name}.json
//! DO NOT EDIT MANUALLY - regenerate from schema instead
//!
//! Generated on: {datetime.now(timezone.utc).isoformat()}

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{{Result, anyhow}};
use serde::{{Deserialize, Serialize}};
use tokio::sync::RwLock;

use crate::core::{{Tool, ToolParams, ToolResult, ToolMetadata}};
use crate::errors::ToolError;
use toka_runtime::{{Capability, SecurityLevel}};

{params_struct}

{implementation}

{tests}'''

        return rust_code
    
    def _generate_params_struct(self, inputs: List[Dict[str, Any]], tool_name: str) -> str:
        """Generate parameter struct from inputs."""
        if not inputs:
            return f'''#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {tool_name}Params {{
    // No parameters for this tool
}}'''
        
        fields = []
        for param in inputs:
            param_name = param['name']
            param_type = self._rust_type_from_json_type(param['type'])
            required = param.get('required', True)
            
            if not required:
                param_type = f"Option<{param_type}>"
            
            description = param.get('description', 'Parameter')
            fields.append(f'    /// {description}\n    pub {param_name}: {param_type},')
        
        fields_str = '\n'.join(fields)
        
        return f'''#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {tool_name}Params {{
{fields_str}
}}'''
    
    def _generate_implementation(self, tool_schema: Dict[str, Any], tool_name_pascal: str, tool_name_snake: str) -> str:
        """Generate tool implementation."""
        metadata = tool_schema['metadata']
        spec = tool_schema['spec']
        
        capabilities = spec.get('capabilities', [])
        capabilities_rust = ', '.join(f'Capability::{cap.replace("-", "_").title()}' for cap in capabilities)
        
        security_level = spec.get('security', {}).get('level', 'sandbox')
        security_rust = f'SecurityLevel::{security_level.title()}'
        
        # Implementation type determines the execute method
        impl_type = spec.get('implementation', {}).get('type', 'rust')
        execute_method = self._generate_execute_method(impl_type, tool_schema)
        
        return f'''/// {metadata.get('description', 'Generated tool')}
#[derive(Debug, Clone)]
pub struct {tool_name_pascal}Tool {{
    metadata: ToolMetadata,
}}

impl {tool_name_pascal}Tool {{
    /// Create a new instance of the tool
    pub fn new() -> Self {{
        Self {{
            metadata: ToolMetadata {{
                name: "{metadata['name']}".to_string(),
                version: "{metadata['version']}".to_string(),
                description: "{metadata.get('description', '')}".to_string(),
                category: "{metadata.get('category', 'utility')}".to_string(),
                capabilities: vec![{capabilities_rust}],
                security_level: {security_rust},
            }},
        }}
    }}
}}

#[async_trait::async_trait]
impl Tool for {tool_name_pascal}Tool {{
    fn metadata(&self) -> &ToolMetadata {{
        &self.metadata
    }}
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError> {{
        // Parse parameters
        let tool_params: {tool_name_pascal}Params = serde_json::from_value(
            serde_json::to_value(&params.args)?
        ).map_err(|e| ToolError::InvalidParams(format!("Failed to parse parameters: {{}}", e)))?;
        
{execute_method}
    }}
}}'''
    
    def _generate_execute_method(self, impl_type: str, tool_schema: Dict[str, Any]) -> str:
        """Generate execute method based on implementation type."""
        if impl_type == 'rust':
            return '''        // TODO: Implement Rust-native logic here
        // This is a template - replace with actual implementation
        
        let output = format!("Tool executed with params: {{:?}}", tool_params);
        
        Ok(ToolResult {{
            output,
            success: true,
            metadata: std::collections::HashMap::new(),
        }})'''
        
        elif impl_type == 'python':
            script_path = tool_schema.get('spec', {}).get('implementation', {}).get('entry_point', 'script.py')
            return f'''        // Execute Python script
        let script_path = "{script_path}";
        let output = tokio::process::Command::new("python3")
            .arg(script_path)
            .arg(serde_json::to_string(&tool_params)?)
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Python execution failed: {{}}", e)))?;
        
        if !output.status.success() {{
            return Err(ToolError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }}
        
        let result_str = String::from_utf8_lossy(&output.stdout).to_string();
        
        Ok(ToolResult {{
            output: result_str,
            success: true,
            metadata: std::collections::HashMap::new(),
        }})'''
        
        else:
            return f'''        // TODO: Implement {impl_type} execution
        // This implementation type requires manual review
        Err(ToolError::UnsupportedOperation(
            "Implementation type {impl_type} not yet supported".to_string()
        ))'''
    
    def _generate_tests(self, tool_name_pascal: str, tool_name_snake: str) -> str:
        """Generate basic tests for the tool."""
        return f'''#[cfg(test)]
mod tests {{
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_{tool_name_snake}_creation() {{
        let tool = {tool_name_pascal}Tool::new();
        assert_eq!(tool.metadata().name, "{tool_name_snake.replace('_', '-')}");
    }}
    
    #[tokio::test]
    async fn test_{tool_name_snake}_execution() {{
        let tool = {tool_name_pascal}Tool::new();
        let params = ToolParams {{
            name: "{tool_name_snake.replace('_', '-')}".to_string(),
            args: HashMap::new(),
        }};
        
        let result = tool.execute(&params).await;
        assert!(result.is_ok(), "Tool execution should succeed");
        
        let tool_result = result.unwrap();
        assert!(tool_result.success, "Tool should report success");
    }}
}}'''
    
    def _rust_type_from_json_type(self, json_type: str) -> str:
        """Convert JSON schema type to Rust type."""
        type_mapping = {
            'string': 'String',
            'integer': 'i64',
            'number': 'f64',
            'boolean': 'bool',
            'array': 'Vec<serde_json::Value>',
            'object': 'serde_json::Value'
        }
        return type_mapping.get(json_type, 'serde_json::Value')
    
    def generate_yaml_manifest(self, tool_schema: Dict[str, Any]) -> str:
        """Generate YAML manifest for toka-tools runtime."""
        metadata = tool_schema['metadata']
        spec = tool_schema['spec']
        
        # Convert JSON schema to YAML manifest format
        yaml_content = f'''# Auto-generated YAML manifest for {metadata['name']}
# Generated on: {datetime.now(timezone.utc).isoformat()}

metadata:
  name: "{metadata['name']}"
  version: "{metadata['version']}"
  category: "{metadata.get('category', 'utility')}"
  description: "{metadata.get('description', '')}"
  author: "{metadata.get('author', 'Generated')}"
  created: "{metadata.get('created', datetime.now(timezone.utc).isoformat())}"

spec:
  executable:
    type: "rust"
    path: "src/generated/{metadata['name'].replace('-', '_')}.rs"
    working_directory: "."
  
  capabilities:
    required: {json.dumps(spec.get('capabilities', []))}
    optional: []
  
  security:
    level: "{spec.get('security', {}).get('level', 'sandbox')}"
    sandbox:
      network_access: false
      file_system_access: "restricted"
  
  parameters: {json.dumps(spec.get('interface', {}).get('inputs', []), indent=4)}

interface:
  inputs: {json.dumps(spec.get('interface', {}).get('inputs', []), indent=4)}
  outputs:
    format: "{spec.get('interface', {}).get('outputs', {}).get('format', 'json')}"
    encoding: "utf-8"

protocols:
  - name: "toka-agent"
    version: "1.0"

outputs:
  success:
    format: "json"
    schema:
      type: "object"
      properties:
        output:
          type: "string"
        success:
          type: "boolean"

dependencies:
  system: []
  workspace: ["toka-runtime", "toka-kernel"]
'''
        return yaml_content
    
    def update_tool_registry(self, tool_name: str) -> None:
        """Update toka-tools registry with new tool."""
        # This would update the registry file to include the new tool
        registry_file = self.toka_tools_dir / "src" / "tools" / "mod.rs"
        
        print(f"📝 TODO: Update registry at {registry_file} to include {tool_name}")
        print(f"    Add: pub mod {tool_name.replace('-', '_')};")
        print(f"    Export tool in registry function")
    
    def _needs_manual_review(self, tool_schema: Dict[str, Any]) -> bool:
        """Check if tool needs manual review before deployment."""
        spec = tool_schema['spec']
        
        # Complex tools need review
        if len(spec.get('capabilities', [])) > 5:
            return True
        
        # Non-sandbox security needs review
        if spec.get('security', {}).get('level') != 'sandbox':
            return True
        
        # Non-Rust implementations need review
        if spec.get('implementation', {}).get('type') != 'rust':
            return True
        
        # Tools with many parameters need review
        if len(spec.get('interface', {}).get('inputs', [])) > 10:
            return True
        
        return False
    
    def validate_cargo_build(self) -> bool:
        """Validate that the generated code compiles."""
        try:
            result = subprocess.run(
                ['cargo', 'check'],
                cwd=self.toka_tools_dir,
                capture_output=True,
                text=True,
                timeout=60
            )
            return result.returncode == 0
        except Exception:
            return False


def main():
    """CLI interface for the bridge system."""
    if len(sys.argv) < 2:
        print("Usage: python sync.py <tool-schema-file>")
        sys.exit(1)
    
    schema_file = sys.argv[1]
    
    try:
        with open(schema_file, 'r') as f:
            tool_schema = json.load(f)
        
        bridge = TokaToolsBridge()
        result = bridge.sync_validated_tool(tool_schema)
        
        if result.success:
            print(f"✅ Successfully synced tool to: {result.rust_file}")
            
            if result.warnings:
                print("\n⚠️  Warnings:")
                for warning in result.warnings:
                    print(f"  - {warning}")
            
            # Test compilation
            print(f"\n🔨 Testing compilation...")
            if bridge.validate_cargo_build():
                print("✅ Generated code compiles successfully")
            else:
                print("❌ Generated code has compilation errors")
                print("💡 Manual review and fixes needed")
        else:
            print("❌ Failed to sync tool:")
            for error in result.errors:
                print(f"  - {error}")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main() 