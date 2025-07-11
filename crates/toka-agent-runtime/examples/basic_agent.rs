//! Basic agent runtime example.
//!
//! This example demonstrates how to create and run a simple agent using the
//! toka-agent-runtime. It shows the basic integration between the agent
//! runtime, orchestration, and LLM gateway.
//!
//! Generated: 2025-07-11 (UTC)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde_json::json;
use tokio::time::sleep;

use toka_agent_runtime::{AgentExecutor, AgentContext, AgentExecutionState, AgentMetrics};
use toka_types::{
    AgentConfig, AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities,
    AgentTasks, TaskConfig, TaskPriority, AgentDependencies, ReportingConfig,
    ReportingFrequency, SecurityConfig, ResourceLimits, AgentObjective, EntityId,
};

/// Mock LLM Gateway for demonstration
struct MockLlmGateway;

#[async_trait::async_trait]
impl toka_llm_gateway::LlmGateway for MockLlmGateway {
    async fn complete(&self, request: toka_llm_gateway::LlmRequest) -> Result<toka_llm_gateway::LlmResponse> {
        // Simulate LLM processing time
        sleep(Duration::from_millis(100)).await;
        
        let response_content = format!(
            "Task completed successfully. Analyzed: {}",
            request.messages().first().map(|m| &m.content).unwrap_or("unknown")
        );
        
        Ok(toka_llm_gateway::LlmResponse::new(
            response_content,
            toka_llm_gateway::Usage {
                prompt_tokens: 50,
                completion_tokens: 25,
                total_tokens: 75,
            }
        ))
    }
}

/// Mock Runtime Manager for demonstration
struct MockRuntimeManager;

#[async_trait::async_trait]
impl toka_runtime::RuntimeManager for MockRuntimeManager {
    async fn submit(&self, _message: toka_types::Message) -> Result<()> {
        // Mock implementation - just log the message
        println!("Runtime received message for processing");
        Ok(())
    }
}

/// Create a sample agent configuration for testing
fn create_sample_agent_config() -> AgentConfig {
    AgentConfig {
        metadata: AgentMetadata {
            name: "demo-agent".to_string(),
            version: "v1.0.0".to_string(),
            created: "2025-07-11".to_string(),
            workstream: "Demo Workstream".to_string(),
            branch: "feature/demo-agent".to_string(),
        },
        spec: AgentSpecConfig {
            name: "Demo Agent".to_string(),
            domain: "demonstration".to_string(),
            priority: AgentPriority::High,
        },
        capabilities: AgentCapabilities {
            primary: vec![
                "filesystem-read".to_string(),
                "analysis".to_string(),
                "reporting".to_string(),
            ],
            secondary: vec![
                "documentation".to_string(),
            ],
        },
        objectives: vec![
            AgentObjective {
                description: "Demonstrate agent execution capabilities".to_string(),
                deliverable: "Successful completion of demo tasks".to_string(),
                validation: "All tasks complete without errors".to_string(),
            },
            AgentObjective {
                description: "Show progress reporting functionality".to_string(),
                deliverable: "Real-time progress updates during execution".to_string(),
                validation: "Progress reports sent to orchestration system".to_string(),
            },
        ],
        tasks: AgentTasks {
            default: vec![
                TaskConfig {
                    description: "Initialize agent environment and validate configuration".to_string(),
                    priority: TaskPriority::High,
                },
                TaskConfig {
                    description: "Analyze current workspace structure and report findings".to_string(),
                    priority: TaskPriority::High,
                },
                TaskConfig {
                    description: "Demonstrate LLM integration with sample query".to_string(),
                    priority: TaskPriority::Medium,
                },
                TaskConfig {
                    description: "Generate completion report with metrics".to_string(),
                    priority: TaskPriority::Medium,
                },
            ],
        },
        dependencies: AgentDependencies {
            required: HashMap::new(),
            optional: HashMap::new(),
        },
        reporting: ReportingConfig {
            frequency: ReportingFrequency::Daily,
            channels: vec!["demo-channel".to_string()],
            metrics: vec![
                ("task_completion_rate".to_string(), "Percentage of tasks completed successfully".to_string()),
                ("response_time".to_string(), "Average task response time in seconds".to_string()),
            ].into_iter().collect(),
        },
        security: SecurityConfig {
            sandbox: true,
            capabilities_required: vec![
                "filesystem-read".to_string(),
                "analysis".to_string(),
            ],
            resource_limits: ResourceLimits {
                max_memory: "256MB".to_string(),
                max_cpu: "25%".to_string(),
                timeout: "10m".to_string(),
            },
        },
    }
}

/// Main example function
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,toka_agent_runtime=debug")
        .init();

    println!("🚀 Starting Toka Agent Runtime Demo");
    println!("Generated: 2025-07-11 (UTC)");
    println!("========================================");

    // Create mock services
    let llm_gateway = Arc::new(MockLlmGateway);
    let runtime_manager = Arc::new(MockRuntimeManager);

    // Create agent configuration
    let agent_config = create_sample_agent_config();
    let agent_id = EntityId(42);

    println!("✅ Created agent configuration: {}", agent_config.metadata.name);
    println!("   - Domain: {}", agent_config.spec.domain);
    println!("   - Priority: {:?}", agent_config.spec.priority);
    println!("   - Tasks: {}", agent_config.tasks.default.len());
    println!("   - Capabilities: {:?}", agent_config.capabilities.primary);

    // Create agent executor
    println!("\n🔧 Creating agent executor...");
    let agent_executor = AgentExecutor::new(
        agent_config,
        agent_id,
        runtime_manager,
        llm_gateway,
    ).await?;

    println!("✅ Agent executor created successfully");

    // Run the agent
    println!("\n🏃 Starting agent execution...");
    let start_time = std::time::Instant::now();

    match agent_executor.run().await {
        Ok(()) => {
            let duration = start_time.elapsed();
            println!("\n🎉 Agent execution completed successfully!");
            println!("   - Total duration: {:?}", duration);
            println!("   - Status: SUCCESS");
        }
        Err(error) => {
            let duration = start_time.elapsed();
            println!("\n❌ Agent execution failed!");
            println!("   - Error: {}", error);
            println!("   - Duration: {:?}", duration);
            println!("   - Status: FAILED");
            return Err(error);
        }
    }

    println!("\n📊 Demo completed. This shows:");
    println!("   ✅ Agent configuration loading");
    println!("   ✅ Task execution with LLM integration");
    println!("   ✅ Progress reporting");
    println!("   ✅ Resource management");
    println!("   ✅ Error handling and recovery");

    println!("\n🔗 To integrate with real services:");
    println!("   1. Replace MockLlmGateway with actual LLM service");
    println!("   2. Replace MockRuntimeManager with toka-runtime");
    println!("   3. Connect to toka-orchestration for coordination");
    println!("   4. Add real task implementations");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_config_creation() {
        let config = create_sample_agent_config();
        
        assert_eq!(config.metadata.name, "demo-agent");
        assert_eq!(config.spec.domain, "demonstration");
        assert_eq!(config.tasks.default.len(), 4);
        assert!(!config.capabilities.primary.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = create_sample_agent_config();
        
        // Verify required fields are present
        assert!(!config.metadata.name.is_empty());
        assert!(!config.metadata.version.is_empty());
        assert!(!config.metadata.created.is_empty());
        assert!(!config.spec.name.is_empty());
        assert!(!config.spec.domain.is_empty());
        
        // Verify security configuration
        assert!(config.security.sandbox);
        assert!(!config.security.capabilities_required.is_empty());
        assert!(!config.security.resource_limits.max_memory.is_empty());
    }

    #[tokio::test]
    async fn test_mock_llm_gateway() {
        let gateway = MockLlmGateway;
        let request = toka_llm_gateway::LlmRequest::new("Test prompt".to_string()).unwrap();
        
        let response = gateway.complete(request).await.unwrap();
        
        assert!(response.content().contains("Task completed successfully"));
        assert_eq!(response.usage().total_tokens, 75);
    }
}