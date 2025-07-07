//! Agent specification system for dynamic composability
//!
//! This module provides the core infrastructure for loading, validating, and composing
//! agents dynamically using the JSON schemas. It integrates agent behaviors into the
//! Rust crate system for standardization and unification.

pub mod specification;
pub mod composer;
pub mod behaviors;
pub mod orchestration;
pub mod validation;

// Re-export key types for convenience
pub use specification::{AgentSpec, AgentMetadata, AgentCapabilities, AgentObjective, AgentTask};
pub use composer::{AgentComposer, CompositionConfig, AgentTemplate};
pub use behaviors::{BehavioralDirectives, RiskMitigation, SuccessCriteria};
pub use orchestration::{AgentOrchestrator, OrchestrationPlan, WorkstreamCoordinator};
pub use validation::{AgentValidator, ValidationResult, SchemaValidator};

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::core::ToolRegistry;

/// Agent system that provides dynamic composability and standardization
pub struct AgentSystem {
    registry: Arc<ToolRegistry>,
    composer: AgentComposer,
    orchestrator: AgentOrchestrator,
    validator: AgentValidator,
}

impl AgentSystem {
    /// Create a new agent system
    pub async fn new(registry: Arc<ToolRegistry>) -> Result<Self> {
        let composer = AgentComposer::new().await?;
        let orchestrator = AgentOrchestrator::new().await?;
        let validator = AgentValidator::new().await?;
        
        Ok(Self {
            registry,
            composer,
            orchestrator,
            validator,
        })
    }
    
    /// Load and validate an agent specification from a file
    pub async fn load_agent_spec(&self, path: &Path) -> Result<AgentSpec> {
        let spec = self.composer.load_spec_from_file(path).await?;
        self.validator.validate_spec(&spec)?;
        Ok(spec)
    }
    
    /// Compose a new agent from a template and configuration
    pub async fn compose_agent(
        &self,
        template: &AgentTemplate,
        config: &CompositionConfig,
    ) -> Result<AgentSpec> {
        let spec = self.composer.compose_from_template(template, config).await?;
        self.validator.validate_spec(&spec)?;
        Ok(spec)
    }
    
    /// Register an agent specification as tools in the tool registry
    pub async fn register_agent_as_tools(&self, spec: &AgentSpec) -> Result<()> {
        self.composer.register_as_tools(&self.registry, spec).await
    }
    
    /// Create an orchestration plan for multiple agents
    pub async fn create_orchestration_plan(
        &self,
        agents: &[AgentSpec],
    ) -> Result<OrchestrationPlan> {
        self.orchestrator.create_plan(agents).await
    }
    
    /// List available agent templates
    pub async fn list_templates(&self) -> Vec<String> {
        self.composer.list_templates().await
    }
}

/// Unified configuration for the agent system
#[derive(Debug, Clone)]
pub struct AgentSystemConfig {
    /// Path to agent templates directory
    pub templates_dir: std::path::PathBuf,
    /// Path to schemas directory
    pub schemas_dir: std::path::PathBuf,
    /// Enable validation
    pub enable_validation: bool,
    /// Enable orchestration features
    pub enable_orchestration: bool,
}

impl Default for AgentSystemConfig {
    fn default() -> Self {
        Self {
            templates_dir: std::path::PathBuf::from("templates"),
            schemas_dir: std::path::PathBuf::from(".cursor/schemas"),
            enable_validation: true,
            enable_orchestration: true,
        }
    }
}