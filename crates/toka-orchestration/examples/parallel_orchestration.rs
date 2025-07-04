//! Parallel Agent Orchestration Example
//!
//! This example demonstrates how to use the Toka orchestration system to spawn
//! and coordinate multiple agents in parallel, managing dependencies and monitoring
//! progress across different workstreams.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tracing::{error, info, warn};
use tracing_subscriber;

use toka_auth::MockTokenValidator;
use toka_orchestration::{OrchestrationConfig, OrchestrationEngine};
use toka_runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting parallel agent orchestration example");

    // Create mock configuration for demonstration
    let config = create_demo_configuration()?;

    // Initialize Toka runtime
    let runtime = Arc::new(
        Runtime::new(
            RuntimeConfig::default(),
            Arc::new(MockTokenValidator::new()),
        )
        .await?,
    );

    info!("Toka runtime initialized");

    // Create orchestration engine
    let engine = Arc::new(OrchestrationEngine::new(config.clone(), runtime.clone()).await?);

    info!(
        "Orchestration engine created with {} agents",
        config.agents.len()
    );

    // Optional: Add LLM integration for intelligent coordination
    // let llm_config = toka_llm_gateway::Config::from_env()?;
    // let llm_gateway = Arc::new(LlmGateway::new(llm_config).await?);
    // let engine = engine.with_llm_gateway(llm_gateway);

    // Start orchestration
    let session = engine.start_orchestration().await?;
    info!("Orchestration session started: {}", session.session_id());

    // Monitor progress
    let monitoring_task = tokio::spawn({
        let session = session;
        async move { monitor_orchestration_progress(session).await }
    });

    // Wait for completion or timeout
    let result = tokio::time::timeout(
        Duration::from_secs(300), // 5 minute timeout
        monitoring_task,
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Orchestration completed successfully");
        }
        Ok(Err(e)) => {
            error!("Orchestration failed: {}", e);
            return Err(e);
        }
        Err(_) => {
            warn!("Orchestration timed out after 5 minutes");
        }
    }

    info!("Parallel agent orchestration example completed");
    Ok(())
}

/// Monitor orchestration progress and display updates.
async fn monitor_orchestration_progress(
    session: toka_orchestration::OrchestrationSession,
) -> Result<()> {
    // Subscribe to progress events (if available)
    // This would require extending the session API to expose progress monitoring

    // For now, we'll poll the session state
    let mut last_progress = 0.0;

    loop {
        let state = session.get_state().await;

        if state.progress != last_progress {
            info!(
                "Orchestration progress: {:.1}% - Phase: {:?}",
                state.progress * 100.0,
                state.current_phase
            );
            last_progress = state.progress;
        }

        // Display agent status
        let agents = session.get_spawned_agents();
        if !agents.is_empty() {
            let active_count = agents
                .iter()
                .filter(|a| a.state == toka_orchestration::AgentState::Active)
                .count();
            let completed_count = agents
                .iter()
                .filter(|a| a.state == toka_orchestration::AgentState::Completed)
                .count();
            let failed_count = agents
                .iter()
                .filter(|a| a.state == toka_orchestration::AgentState::Failed)
                .count();

            info!(
                "Agents - Active: {}, Completed: {}, Failed: {}, Total: {}",
                active_count,
                completed_count,
                failed_count,
                agents.len()
            );
        }

        // Check for completion
        if state.completed {
            info!("Orchestration completed successfully!");
            break;
        }

        // Check for errors
        if let Some(error) = &state.error {
            error!("Orchestration error: {}", error);
            return Err(anyhow::anyhow!("Orchestration failed: {}", error));
        }

        // Wait before next check
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Get final results before consuming session
    let final_state = session.get_state().await;
    let final_agents = session.get_spawned_agents();

    // Wait for actual completion
    session.wait_for_completion().await?;

    // Display final results
    display_orchestration_results(final_state, final_agents).await;

    Ok(())
}

/// Display final orchestration results.
async fn display_orchestration_results(
    state: toka_orchestration::SessionState,
    agents: Vec<toka_orchestration::SpawnedAgent>,
) {
    info!("=== Orchestration Results ===");

    info!("Session ID: {}", state.session_id);
    info!("Started at: {}", state.started_at);
    info!("Final progress: {:.1}%", state.progress * 100.0);
    info!("Final phase: {:?}", state.current_phase);
    if !agents.is_empty() {
        info!("=== Agent Results ===");
        for agent in &agents {
            info!(
                "Agent: {} | State: {:?} | Tasks: {}/{} | Execution time: {:?}",
                agent.config.metadata.name,
                agent.state,
                agent.metrics.tasks_completed,
                agent.metrics.tasks_assigned,
                agent.metrics.execution_time
            );
        }

        // Calculate summary statistics
        let total_agents = agents.len();
        let completed_agents = agents
            .iter()
            .filter(|a| a.state == toka_orchestration::AgentState::Completed)
            .count();
        let failed_agents = agents
            .iter()
            .filter(|a| a.state == toka_orchestration::AgentState::Failed)
            .count();
        let success_rate = if total_agents > 0 {
            completed_agents as f64 / total_agents as f64 * 100.0
        } else {
            0.0
        };

        info!("=== Summary ===");
        info!("Total agents: {}", total_agents);
        info!("Completed: {}", completed_agents);
        info!("Failed: {}", failed_agents);
        info!("Success rate: {:.1}%", success_rate);
    }
}

/// Create a demo configuration with sample agents for testing.
fn create_demo_configuration() -> Result<OrchestrationConfig> {
    use std::collections::HashMap;
    use toka_orchestration::*;

    // Create sample agents for demonstration
    let agents = vec![
        // Critical infrastructure agent
        AgentConfig {
            metadata: AgentMetadata {
                name: "build-system-stabilization".to_string(),
                version: "v0.3.0".to_string(),
                created: "2025-01-04".to_string(),
                workstream: "build-system-stabilization".to_string(),
                branch: "feature/build-system-stabilization".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Build System Stabilization Agent".to_string(),
                domain: "build-infrastructure".to_string(),
                priority: AgentPriority::Critical,
            },
            capabilities: AgentCapabilities {
                primary: vec![
                    "build-analysis".to_string(),
                    "dependency-resolution".to_string(),
                ],
                secondary: vec!["testing".to_string()],
            },
            objectives: vec![AgentObjective {
                description: "Resolve base64ct dependency conflicts".to_string(),
                deliverable: "Updated Cargo.toml files with resolved dependencies".to_string(),
                validation: "Successful build across all workspace crates".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![
                    TaskConfig {
                        description: "Analyze current dependency conflicts".to_string(),
                        priority: TaskPriority::High,
                    },
                    TaskConfig {
                        description: "Update Cargo.toml dependencies".to_string(),
                        priority: TaskPriority::High,
                    },
                    TaskConfig {
                        description: "Test build across workspace".to_string(),
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
                channels: vec!["main-agent".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["build-tools".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "500MB".to_string(),
                    max_cpu: "100%".to_string(),
                    timeout: "30m".to_string(),
                },
            },
        },
        // Testing infrastructure agent (depends on build system)
        AgentConfig {
            metadata: AgentMetadata {
                name: "testing-infrastructure".to_string(),
                version: "v0.3.0".to_string(),
                created: "2025-01-04".to_string(),
                workstream: "testing-infrastructure".to_string(),
                branch: "feature/testing-infrastructure".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Testing Infrastructure Expansion Agent".to_string(),
                domain: "testing".to_string(),
                priority: AgentPriority::High,
            },
            capabilities: AgentCapabilities {
                primary: vec!["test-design".to_string(), "integration-testing".to_string()],
                secondary: vec!["performance-testing".to_string()],
            },
            objectives: vec![AgentObjective {
                description: "Implement cross-crate integration tests".to_string(),
                deliverable: "Comprehensive integration test suite".to_string(),
                validation: "All tests pass and provide adequate coverage".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![
                    TaskConfig {
                        description: "Design integration test framework".to_string(),
                        priority: TaskPriority::High,
                    },
                    TaskConfig {
                        description: "Implement runtime-storage integration tests".to_string(),
                        priority: TaskPriority::High,
                    },
                ],
            },
            dependencies: AgentDependencies {
                required: {
                    let mut deps = HashMap::new();
                    deps.insert(
                        "build-system-stabilization".to_string(),
                        "Requires stable build system".to_string(),
                    );
                    deps
                },
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: ReportingFrequency::Daily,
                channels: vec!["main-agent".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["test-execution".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "1GB".to_string(),
                    max_cpu: "75%".to_string(),
                    timeout: "1h".to_string(),
                },
            },
        },
        // Parallel development agents
        AgentConfig {
            metadata: AgentMetadata {
                name: "security-enhancement".to_string(),
                version: "v0.3.0".to_string(),
                created: "2025-01-04".to_string(),
                workstream: "security-enhancement".to_string(),
                branch: "feature/security-enhancement".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Security Framework Extension Agent".to_string(),
                domain: "security".to_string(),
                priority: AgentPriority::Medium,
            },
            capabilities: AgentCapabilities {
                primary: vec![
                    "security-analysis".to_string(),
                    "auth-enhancement".to_string(),
                ],
                secondary: vec!["audit-logging".to_string()],
            },
            objectives: vec![AgentObjective {
                description: "Implement JWT key rotation mechanism".to_string(),
                deliverable: "Automatic JWT key rotation system".to_string(),
                validation: "Secure key rotation without service interruption".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![TaskConfig {
                    description: "Design JWT rotation mechanism".to_string(),
                    priority: TaskPriority::High,
                }],
            },
            dependencies: AgentDependencies {
                required: {
                    let mut deps = HashMap::new();
                    deps.insert(
                        "build-system-stabilization".to_string(),
                        "Requires stable build system".to_string(),
                    );
                    deps
                },
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: ReportingFrequency::Daily,
                channels: vec!["main-agent".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["security-tools".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "300MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "45m".to_string(),
                },
            },
        },
    ];

    Ok(OrchestrationConfig {
        agents,
        global_timeout: Duration::from_secs(1800), // 30 minutes
        max_concurrent_agents: 5,
    })
}

/// Mock auth validator for the example.
mod mock_auth {
    use async_trait::async_trait;
    use toka_auth::{Claims, Result, TokenValidator};

    pub struct MockTokenValidator;

    impl MockTokenValidator {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl TokenValidator for MockTokenValidator {
        async fn validate(&self, _token: &str) -> Result<Claims> {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            Ok(Claims {
                sub: "demo-user".to_string(),
                vault: "demo-vault".to_string(),
                permissions: vec!["orchestration".to_string()],
                iat: now,
                exp: now + 3600, // 1 hour from now
                jti: uuid::Uuid::new_v4().to_string(),
            })
        }
    }
}
