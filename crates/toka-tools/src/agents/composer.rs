//! Agent composer for dynamic agent creation and composition
//!
//! This module provides functionality to compose agents from templates,
//! load specifications from files, and integrate with the tool registry.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::core::ToolRegistry;
use super::specification::*;

/// Agent composer for dynamic agent creation
pub struct AgentComposer {
    templates: HashMap<String, AgentTemplate>,
    template_dir: std::path::PathBuf,
}

/// Agent template for creating new agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub name: String,
    pub domain: AgentDomain,
    pub base_spec: AgentSpec,
    pub configurable_fields: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub template_metadata: TemplateMetadata,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub created: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
}

/// Configuration for agent composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionConfig {
    pub name: String,
    pub workstream: String,
    pub priority: AgentPriority,
    pub custom_capabilities: Vec<String>,
    pub custom_objectives: Vec<AgentObjective>,
    pub custom_tasks: Vec<AgentTask>,
    pub environment_overrides: HashMap<String, serde_json::Value>,
}

impl AgentComposer {
    /// Create a new agent composer
    pub async fn new() -> Result<Self> {
        let template_dir = std::path::PathBuf::from("templates/agents");
        let mut composer = Self {
            templates: HashMap::new(),
            template_dir,
        };
        
        // Load default templates
        composer.load_default_templates().await?;
        
        Ok(composer)
    }

    /// Create composer with custom template directory
    pub async fn with_template_dir(template_dir: std::path::PathBuf) -> Result<Self> {
        let mut composer = Self {
            templates: HashMap::new(),
            template_dir,
        };
        
        composer.load_templates().await?;
        
        Ok(composer)
    }

    /// Load agent specification from file
    pub async fn load_spec_from_file(&self, path: &Path) -> Result<AgentSpec> {
        let content = fs::read_to_string(path).await?;
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            Ok(serde_yaml::from_str(&content)?)
        } else {
            Ok(serde_json::from_str(&content)?)
        }
    }

    /// Compose agent from template and configuration
    pub async fn compose_from_template(
        &self,
        template: &AgentTemplate,
        config: &CompositionConfig,
    ) -> Result<AgentSpec> {
        let mut spec = template.base_spec.clone();
        
        // Apply configuration overrides
        self.apply_composition_config(&mut spec, config)?;
        
        // Update metadata
        spec.metadata.name = config.name.to_lowercase().replace(" ", "-");
        spec.metadata.workstream = config.workstream.clone();
        spec.metadata.branch = format!("feature/{}", spec.metadata.name);
        
        // Update spec details
        spec.spec.name = config.name.clone();
        spec.spec.priority = config.priority.clone();
        
        // Add custom capabilities
        spec.capabilities.primary.extend(config.custom_capabilities.clone());
        spec.capabilities.primary.extend(template.required_capabilities.clone());
        
        // Add custom objectives and tasks
        spec.objectives.extend(config.custom_objectives.clone());
        spec.tasks.default.extend(config.custom_tasks.clone());
        
        Ok(spec)
    }

    /// Register agent specification as tools in the registry
    pub async fn register_as_tools(
        &self,
        registry: &Arc<ToolRegistry>,
        spec: &AgentSpec,
    ) -> Result<()> {
        // Create tools for each agent capability
        for capability in &spec.capabilities.primary {
            let tool_name = format!("{}_{}", spec.metadata.name, capability);
            
            // Create tool configuration based on agent spec
            let tool_config = self.create_tool_config_from_spec(spec, capability)?;
            
            // Register with the tool registry
            registry.register_tool(tool_name, tool_config).await?;
        }
        
        Ok(())
    }

    /// List available templates
    pub async fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Load default templates
    async fn load_default_templates(&mut self) -> Result<()> {
        // Create default templates based on the canonical schema
        
        // GitHub Integration Template
        let github_template = self.create_github_template()?;
        self.templates.insert("github-integration".to_string(), github_template);
        
        // Build System Template
        let build_template = self.create_build_system_template()?;
        self.templates.insert("build-system".to_string(), build_template);
        
        // Testing Infrastructure Template
        let testing_template = self.create_testing_template()?;
        self.templates.insert("testing-infrastructure".to_string(), testing_template);
        
        // Security Template
        let security_template = self.create_security_template()?;
        self.templates.insert("security".to_string(), security_template);
        
        Ok(())
    }

    /// Load templates from directory
    async fn load_templates(&mut self) -> Result<()> {
        if !self.template_dir.exists() {
            return Ok(());
        }
        
        let mut entries = fs::read_dir(&self.template_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(&path).await?;
                let template: AgentTemplate = serde_yaml::from_str(&content)?;
                
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                self.templates.insert(name, template);
            }
        }
        
        Ok(())
    }

    /// Apply composition configuration to spec
    fn apply_composition_config(
        &self,
        spec: &mut AgentSpec,
        config: &CompositionConfig,
    ) -> Result<()> {
        // Apply environment overrides
        for (key, value) in &config.environment_overrides {
            match key.as_str() {
                "max_memory" => {
                    if let Some(mem) = value.as_str() {
                        spec.security.resource_limits.max_memory = mem.to_string();
                    }
                }
                "max_cpu" => {
                    if let Some(cpu) = value.as_str() {
                        spec.security.resource_limits.max_cpu = cpu.to_string();
                    }
                }
                "timeout" => {
                    if let Some(timeout) = value.as_str() {
                        spec.security.resource_limits.timeout = timeout.to_string();
                    }
                }
                _ => {
                    // Handle other overrides as needed
                }
            }
        }
        
        Ok(())
    }

    /// Create tool configuration from agent spec
    fn create_tool_config_from_spec(
        &self,
        spec: &AgentSpec,
        capability: &str,
    ) -> Result<serde_json::Value> {
        let config = serde_json::json!({
            "name": format!("{}_{}", spec.metadata.name, capability),
            "description": format!("Agent capability: {}", capability),
            "agent_spec": spec,
            "capability": capability,
            "domain": spec.spec.domain,
            "priority": spec.spec.priority,
            "security": {
                "sandbox": spec.security.sandbox,
                "capabilities": spec.security.capabilities_required,
                "resource_limits": spec.security.resource_limits
            }
        });
        
        Ok(config)
    }

    /// Create GitHub integration template
    fn create_github_template(&self) -> Result<AgentTemplate> {
        let base_spec = AgentSpec::new(
            "GitHub Integration Agent".to_string(),
            AgentDomain::GithubIntegration,
            AgentPriority::High,
            "GitHub API Integration & Management".to_string(),
        );
        
        Ok(AgentTemplate {
            name: "github-integration".to_string(),
            domain: AgentDomain::GithubIntegration,
            base_spec,
            configurable_fields: vec![
                "api_endpoints".to_string(),
                "webhook_events".to_string(),
                "permissions".to_string(),
                "rate_limiting".to_string(),
            ],
            required_capabilities: vec![
                "github-api-client".to_string(),
                "repository-management".to_string(),
                "webhook-handling".to_string(),
            ],
            template_metadata: TemplateMetadata {
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: "1.0.0".to_string(),
                description: "Template for GitHub integration agents".to_string(),
                author: "toka-tools".to_string(),
                tags: vec!["github".to_string(), "integration".to_string()],
            },
        })
    }

    /// Create build system template
    fn create_build_system_template(&self) -> Result<AgentTemplate> {
        let base_spec = AgentSpec::new(
            "Build System Agent".to_string(),
            AgentDomain::Infrastructure,
            AgentPriority::Critical,
            "Build System Management".to_string(),
        );
        
        Ok(AgentTemplate {
            name: "build-system".to_string(),
            domain: AgentDomain::Infrastructure,
            base_spec,
            configurable_fields: vec![
                "dependency_management".to_string(),
                "build_validation".to_string(),
                "ci_integration".to_string(),
            ],
            required_capabilities: vec![
                "dependency-conflict-resolution".to_string(),
                "build-validation".to_string(),
                "workspace-management".to_string(),
            ],
            template_metadata: TemplateMetadata {
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: "1.0.0".to_string(),
                description: "Template for build system management agents".to_string(),
                author: "toka-tools".to_string(),
                tags: vec!["build".to_string(), "infrastructure".to_string()],
            },
        })
    }

    /// Create testing infrastructure template
    fn create_testing_template(&self) -> Result<AgentTemplate> {
        let base_spec = AgentSpec::new(
            "Testing Infrastructure Agent".to_string(),
            AgentDomain::QualityAssurance,
            AgentPriority::High,
            "Testing Infrastructure Expansion".to_string(),
        );
        
        Ok(AgentTemplate {
            name: "testing-infrastructure".to_string(),
            domain: AgentDomain::QualityAssurance,
            base_spec,
            configurable_fields: vec![
                "test_types".to_string(),
                "coverage_requirements".to_string(),
                "performance_benchmarks".to_string(),
            ],
            required_capabilities: vec![
                "integration-testing".to_string(),
                "property-testing".to_string(),
                "performance-benchmarking".to_string(),
            ],
            template_metadata: TemplateMetadata {
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: "1.0.0".to_string(),
                description: "Template for testing infrastructure agents".to_string(),
                author: "toka-tools".to_string(),
                tags: vec!["testing".to_string(), "qa".to_string()],
            },
        })
    }

    /// Create security template
    fn create_security_template(&self) -> Result<AgentTemplate> {
        let base_spec = AgentSpec::new(
            "Security Agent".to_string(),
            AgentDomain::Security,
            AgentPriority::High,
            "Security Framework Extension".to_string(),
        );
        
        Ok(AgentTemplate {
            name: "security".to_string(),
            domain: AgentDomain::Security,
            base_spec,
            configurable_fields: vec![
                "security_policies".to_string(),
                "authentication_methods".to_string(),
                "audit_requirements".to_string(),
            ],
            required_capabilities: vec![
                "security-policy-enforcement".to_string(),
                "authentication-management".to_string(),
                "audit-logging".to_string(),
            ],
            template_metadata: TemplateMetadata {
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: "1.0.0".to_string(),
                description: "Template for security framework agents".to_string(),
                author: "toka-tools".to_string(),
                tags: vec!["security".to_string(), "auth".to_string()],
            },
        })
    }
}