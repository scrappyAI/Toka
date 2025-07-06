//! Runtime integration for unified tool system
//!
//! This module provides the integration layer between the unified tool system and
//! the Toka agent runtime, enabling hot-swappable, composable tool execution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::core::{Tool, ToolParams, ToolResult, ToolRegistry};
use crate::wrappers::{UnifiedToolRegistry, DiscoveredTool, ToolType};
use crate::manifest::ToolManifest;

/// Unified tool manifest in YAML format for agent integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedToolManifest {
    pub metadata: ToolMetadata,
    pub spec: ToolSpec,
    pub interface: ToolInterface,
    pub protocols: Vec<ProtocolMapping>,
    pub outputs: ToolOutputs,
    pub dependencies: ToolDependencies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub version: String,
    pub category: String,
    pub description: String,
    pub author: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub executable: ExecutableSpec,
    pub capabilities: CapabilitiesSpec,
    pub security: SecuritySpec,
    pub parameters: Vec<ParameterSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableSpec {
    #[serde(rename = "type")]
    pub exec_type: String,
    pub path: String,
    pub interpreter: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesSpec {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub level: String,
    pub sandbox: SandboxSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    pub memory_limit: String,
    pub cpu_limit: String,
    pub timeout: String,
    pub allow_network: bool,
    #[serde(default)]
    pub network_restrictions: Vec<String>,
    pub readonly_paths: Vec<String>,
    pub writable_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: Option<bool>,
    pub default: Option<serde_yaml::Value>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInterface {
    pub discovery: DiscoveryConfig,
    pub execution: ExecutionConfig,
    pub integration: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub auto_discover: bool,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub hot_swappable: bool,
    pub parallel_safe: bool,
    pub resource_intensive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub agent_types: Vec<String>,
    pub runtime_events: Vec<String>,
    pub compatible_backends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMapping {
    #[serde(rename = "type")]
    pub protocol_type: String,
    pub function_name: Option<String>,
    pub action: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutputs {
    pub primary: Vec<OutputSpec>,
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    #[serde(rename = "type")]
    pub output_type: String,
    pub formats: Vec<String>,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependencies {
    pub system: Vec<String>,
    #[serde(default)]
    pub python: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    pub workspace: Vec<String>,
}

/// Runtime tool registry that integrates with agent system
pub struct RuntimeToolRegistry {
    unified_registry: UnifiedToolRegistry,
    manifest_cache: Arc<RwLock<HashMap<String, UnifiedToolManifest>>>,
    tool_manifests_dir: PathBuf,
    runtime_hooks: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RuntimeToolRegistry {
    /// Create a new runtime tool registry
    pub async fn new(tools_root: impl AsRef<Path>) -> Result<Self> {
        let tools_root = tools_root.as_ref();
        let manifest_dir = tools_root.join("manifests");
        
        info!("Initializing runtime tool registry from: {}", tools_root.display());
        
        let unified_registry = UnifiedToolRegistry::new().await
            .context("Failed to create unified tool registry")?;
        
        let mut registry = Self {
            unified_registry,
            manifest_cache: Arc::new(RwLock::new(HashMap::new())),
            tool_manifests_dir: manifest_dir,
            runtime_hooks: Arc::new(RwLock::new(HashMap::new())),
        };
        
        registry.load_all_manifests().await?;
        registry.register_discovered_tools().await?;
        
        info!("Runtime tool registry initialized successfully");
        Ok(registry)
    }
    
    /// Load all YAML tool manifests
    async fn load_all_manifests(&mut self) -> Result<()> {
        info!("Loading tool manifests from: {}", self.tool_manifests_dir.display());
        
        let mut manifest_cache = self.manifest_cache.write().await;
        
        let mut entries = tokio::fs::read_dir(&self.tool_manifests_dir).await
            .context("Failed to read manifests directory")?;
        
        let mut loaded_count = 0;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_manifest(&path).await {
                    Ok(manifest) => {
                        let tool_name = manifest.metadata.name.clone();
                        manifest_cache.insert(tool_name.clone(), manifest);
                        loaded_count += 1;
                        debug!("Loaded manifest for tool: {}", tool_name);
                    }
                    Err(e) => {
                        warn!("Failed to load manifest {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        info!("Loaded {} tool manifests", loaded_count);
        Ok(())
    }
    
    /// Load a single YAML tool manifest
    async fn load_manifest(&self, path: &Path) -> Result<UnifiedToolManifest> {
        let content = tokio::fs::read_to_string(path).await
            .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;
        
        let manifest: UnifiedToolManifest = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML manifest: {}", path.display()))?;
        
        Ok(manifest)
    }
    
    /// Register discovered tools with the unified registry
    async fn register_discovered_tools(&self) -> Result<()> {
        info!("Registering discovered tools with unified registry");
        
        let registered_count = self.unified_registry.auto_register_tools().await
            .context("Failed to auto-register tools")?;
        
        info!("Registered {} tools with unified registry", registered_count);
        Ok(())
    }
    
    /// Execute a tool with runtime integration
    pub async fn execute_tool_runtime(
        &self,
        tool_name: &str,
        params: &ToolParams,
        agent_capabilities: &[String],
        runtime_context: &RuntimeContext,
    ) -> Result<RuntimeToolResult> {
        debug!("Executing tool '{}' with runtime integration", tool_name);
        
        // Check if tool exists in manifest cache
        let manifest = {
            let cache = self.manifest_cache.read().await;
            cache.get(tool_name).cloned()
        };
        
        let manifest = manifest
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found in manifest cache", tool_name))?;
        
        // Validate agent capabilities against tool requirements
        self.validate_capabilities(agent_capabilities, &manifest.spec.capabilities)?;
        
        // Execute with unified registry
        let result = self.unified_registry
            .execute_tool_secure(tool_name, params, agent_capabilities)
            .await
            .context("Tool execution failed")?;
        
        // Process runtime hooks
        self.process_runtime_hooks(tool_name, &result, runtime_context).await?;
        
        Ok(RuntimeToolResult {
            tool_result: result,
            manifest: manifest.clone(),
            runtime_context: runtime_context.clone(),
            execution_metadata: RuntimeExecutionMetadata {
                agent_id: runtime_context.agent_id.clone(),
                execution_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
            },
        })
    }
    
    /// Validate agent capabilities against tool requirements
    fn validate_capabilities(
        &self,
        agent_capabilities: &[String],
        tool_capabilities: &CapabilitiesSpec,
    ) -> Result<()> {
        for required_cap in &tool_capabilities.required {
            if !agent_capabilities.contains(required_cap) {
                return Err(anyhow::anyhow!(
                    "Agent missing required capability: {}",
                    required_cap
                ));
            }
        }
        Ok(())
    }
    
    /// Process runtime hooks for tool execution
    async fn process_runtime_hooks(
        &self,
        tool_name: &str,
        result: &ToolResult,
        runtime_context: &RuntimeContext,
    ) -> Result<()> {
        let hooks = self.runtime_hooks.read().await;
        
        if let Some(tool_hooks) = hooks.get(tool_name) {
            for hook in tool_hooks {
                debug!("Processing runtime hook '{}' for tool '{}'", hook, tool_name);
                
                match hook.as_str() {
                    "agent_lifecycle" => {
                        self.emit_agent_lifecycle_event(tool_name, result, runtime_context).await?;
                    }
                    "task_completion" => {
                        self.emit_task_completion_event(tool_name, result, runtime_context).await?;
                    }
                    _ => {
                        warn!("Unknown runtime hook: {}", hook);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Emit agent lifecycle event
    async fn emit_agent_lifecycle_event(
        &self,
        tool_name: &str,
        result: &ToolResult,
        runtime_context: &RuntimeContext,
    ) -> Result<()> {
        debug!("Emitting agent lifecycle event for tool: {}", tool_name);
        // Integration with toka-bus-core would go here
        Ok(())
    }
    
    /// Emit task completion event
    async fn emit_task_completion_event(
        &self,
        tool_name: &str,
        result: &ToolResult,
        runtime_context: &RuntimeContext,
    ) -> Result<()> {
        debug!("Emitting task completion event for tool: {}", tool_name);
        // Integration with toka-bus-core would go here
        Ok(())
    }
    
    /// Get tool manifest
    pub async fn get_tool_manifest(&self, tool_name: &str) -> Option<UnifiedToolManifest> {
        let cache = self.manifest_cache.read().await;
        cache.get(tool_name).cloned()
    }
    
    /// List all available tools
    pub async fn list_available_tools(&self) -> Vec<String> {
        let cache = self.manifest_cache.read().await;
        cache.keys().cloned().collect()
    }
    
    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: &str) -> Vec<String> {
        let cache = self.manifest_cache.read().await;
        cache
            .values()
            .filter(|manifest| manifest.metadata.category == category)
            .map(|manifest| manifest.metadata.name.clone())
            .collect()
    }
    
    /// Hot-swap a tool
    pub async fn hot_swap_tool(&self, tool_name: &str, new_manifest_path: &Path) -> Result<()> {
        info!("Hot-swapping tool: {}", tool_name);
        
        // Load new manifest
        let new_manifest = self.load_manifest(new_manifest_path).await?;
        
        // Validate hot-swappability
        if !new_manifest.interface.execution.hot_swappable {
            return Err(anyhow::anyhow!("Tool '{}' is not hot-swappable", tool_name));
        }
        
        // Update manifest cache
        {
            let mut cache = self.manifest_cache.write().await;
            cache.insert(tool_name.to_string(), new_manifest);
        }
        
        info!("Successfully hot-swapped tool: {}", tool_name);
        Ok(())
    }
}

/// Runtime context for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub agent_id: String,
    pub agent_type: String,
    pub workstream: Option<String>,
    pub execution_environment: String,
    pub capabilities: Vec<String>,
}

/// Runtime tool execution result
#[derive(Debug, Clone)]
pub struct RuntimeToolResult {
    pub tool_result: ToolResult,
    pub manifest: UnifiedToolManifest,
    pub runtime_context: RuntimeContext,
    pub execution_metadata: RuntimeExecutionMetadata,
}

/// Runtime execution metadata
#[derive(Debug, Clone)]
pub struct RuntimeExecutionMetadata {
    pub agent_id: String,
    pub execution_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
} 