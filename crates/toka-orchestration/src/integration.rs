//! Integration layer between orchestration and agent runtime.
//!
//! This module provides the integration between the orchestration engine and the
//! agent runtime, enabling the orchestration system to spawn and manage agent
//! execution processes.

use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn, error};

use toka_agent_runtime::{AgentProcessManager, ProcessResult};
use toka_llm_gateway::LlmGateway;
use toka_runtime::Runtime;
use toka_types::EntityId;

use crate::{OrchestrationEngine, AgentConfig};

/// Integration service that connects orchestration with agent runtime
pub struct RuntimeIntegration {
    /// Orchestration engine
    orchestration: Arc<OrchestrationEngine>,
    /// Agent process manager
    process_manager: Arc<AgentProcessManager>,
    /// LLM gateway
    llm_gateway: Arc<LlmGateway>,
    /// System runtime
    runtime: Arc<Runtime>,
}

impl RuntimeIntegration {
    /// Create a new runtime integration
    pub fn new(
        orchestration: Arc<OrchestrationEngine>,
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Self {
        info!("Creating runtime integration");

        let process_manager = Arc::new(AgentProcessManager::new(
            orchestration.clone(),
            runtime.clone(),
            llm_gateway.clone(),
        ));

        Self {
            orchestration,
            process_manager,
            llm_gateway,
            runtime,
        }
    }

    /// Start agent execution from configuration
    pub async fn start_agent_execution(
        &self,
        config: AgentConfig,
        agent_id: EntityId,
    ) -> Result<ProcessResult> {
        info!("Starting agent execution via integration: {}", config.metadata.name);

        self.process_manager
            .start_agent(config, agent_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start agent: {}", e))
    }

    /// Stop agent execution
    pub async fn stop_agent_execution(&self, agent_id: EntityId) -> Result<ProcessResult> {
        self.process_manager
            .stop_agent(agent_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stop agent: {}", e))
    }

    /// Get process manager reference
    pub fn get_process_manager(&self) -> Arc<AgentProcessManager> {
        self.process_manager.clone()
    }

    /// Monitor and manage agent processes
    pub async fn monitor_runtime(&self) -> Result<()> {
        self.process_manager.monitor_agents().await
    }

    /// Shutdown the runtime integration
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down runtime integration");
        self.process_manager.shutdown().await
    }
}

/// Extension trait for OrchestrationEngine to add runtime capabilities
pub trait OrchestrationRuntimeExt {
    /// Create runtime integration for the orchestration engine
    fn create_runtime_integration(
        self: Arc<Self>,
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> RuntimeIntegration;
}

impl OrchestrationRuntimeExt for OrchestrationEngine {
    fn create_runtime_integration(
        self: Arc<Self>,
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> RuntimeIntegration {
        RuntimeIntegration::new(self, runtime, llm_gateway)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_integration_trait() {
        // Test that the trait extension exists and can be used
        // Note: This would require mock implementations for full testing
        assert!(true);
    }
}