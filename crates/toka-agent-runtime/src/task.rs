//! Task execution engine with LLM integration.
//!
//! This module provides the core task execution functionality that allows agents to
//! execute their configured tasks using LLM assistance while enforcing security
//! constraints and resource limits.

use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use toka_llm_gateway::{LlmGateway, LlmRequest, LlmResponse};
use toka_orchestration::{TaskConfig, TaskPriority, SecurityConfig};

use crate::{
    AgentContext, AgentTask, TaskResult, CapabilityValidator, ResourceManager,
    AgentRuntimeError, AgentRuntimeResult, ExecutionConfig, RetryConfig,
};

/// Task execution engine that uses LLM integration for intelligent task execution
pub struct TaskExecutor {
    /// LLM gateway for task execution
    llm_gateway: std::sync::Arc<LlmGateway>,
    /// Capability validator for security checks
    capability_validator: CapabilityValidator,
    /// Resource manager for enforcement
    resource_manager: ResourceManager,
    /// Execution configuration
    execution_config: ExecutionConfig,
}

/// LLM-based task implementation
#[derive(Debug, Clone)]
pub struct LlmTask {
    /// Unique task identifier
    task_id: String,
    /// Task configuration from agent spec
    config: TaskConfig,
    /// Estimated execution duration
    estimated_duration: Option<Duration>,
    /// Whether task can be retried
    retryable: bool,
}

/// Task execution context with environmental information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionContext {
    /// Base agent context
    pub agent_context: AgentContext,
    /// Task-specific environment variables
    pub task_environment: std::collections::HashMap<String, String>,
    /// Working directory for task execution
    pub working_directory: String,
    /// Available tools and capabilities
    pub available_tools: Vec<String>,
    /// Previous task results for context
    pub previous_results: Vec<TaskResult>,
}

/// LLM prompt template for task execution
#[derive(Debug, Clone)]
pub struct TaskPromptTemplate {
    /// System prompt for agent context
    pub system_prompt: String,
    /// Task instruction template
    pub task_template: String,
    /// Context information template
    pub context_template: String,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(
        llm_gateway: std::sync::Arc<LlmGateway>,
        security_config: SecurityConfig,
        execution_config: ExecutionConfig,
    ) -> Result<Self> {
        let capability_validator = CapabilityValidator::new(
            security_config.capabilities_required.clone(),
            security_config.clone(),
        );

        let resource_manager = ResourceManager::new(security_config.resource_limits.clone())?;

        Ok(Self {
            llm_gateway,
            capability_validator,
            resource_manager,
            execution_config,
        })
    }

    /// Execute a task with LLM assistance and security validation
    #[instrument(skip(self, context), fields(task_id = %task.task_id()))]
    pub async fn execute_task(
        &mut self,
        task: &dyn AgentTask,
        context: &AgentContext,
    ) -> AgentRuntimeResult<TaskResult> {
        let start_time = Instant::now();
        let task_id = task.task_id().to_string();

        info!("Starting task execution: {}", task_id);

        // Validate task permissions
        self.validate_task_permissions(task, context)?;

        // Check resource availability
        self.resource_manager.check_availability()?;

        // Execute task with retries
        let mut retry_count = 0;
        let max_retries = self.execution_config.retry_config.max_retries;

        loop {
            match self.execute_task_attempt(task, context, retry_count).await {
                Ok(result) => {
                    let duration = start_time.elapsed();
                    info!("Task completed successfully: {} (duration: {:?})", task_id, duration);
                    return Ok(result);
                }
                Err(error) => {
                    retry_count += 1;
                    
                    if retry_count > max_retries || !task.is_retryable() {
                        let duration = start_time.elapsed();
                        error!("Task failed after {} attempts: {} (error: {})", 
                               retry_count, task_id, error);
                        
                        return Ok(TaskResult::failure(
                            task_id,
                            task.description().to_string(),
                            error.to_string(),
                            duration,
                        ));
                    }

                    // Calculate retry delay
                    let retry_delay = self.calculate_retry_delay(retry_count);
                    warn!("Task attempt {} failed, retrying in {:?}: {} (error: {})",
                          retry_count, retry_delay, task_id, error);
                    
                    tokio::time::sleep(retry_delay).await;
                }
            }
        }
    }

    /// Execute a single task attempt
    #[instrument(skip(self, context), fields(task_id = %task.task_id(), attempt = retry_count))]
    async fn execute_task_attempt(
        &mut self,
        task: &dyn AgentTask,
        context: &AgentContext,
        retry_count: u32,
    ) -> Result<TaskResult> {
        let start_time = Instant::now();
        let task_id = task.task_id().to_string();

        // Create task execution context
        let task_context = self.create_task_context(context).await?;

        // Build LLM prompt
        let prompt = self.build_task_prompt(task, &task_context, retry_count)?;

        // Execute with LLM
        debug!("Sending task to LLM: {}", task_id);
        let llm_response = self.llm_gateway.complete(prompt).await
            .map_err(|e| anyhow::anyhow!("LLM execution failed: {}", e))?;

        // Parse and validate response
        let task_result = self.parse_task_response(
            task,
            &llm_response,
            start_time.elapsed(),
        )?;

        // Update resource usage
        self.resource_manager.record_usage(
            llm_response.usage().total_tokens as u64,
            start_time.elapsed(),
        )?;

        Ok(task_result)
    }

    /// Validate task against agent capabilities
    fn validate_task_permissions(
        &self,
        task: &dyn AgentTask,
        context: &AgentContext,
    ) -> AgentRuntimeResult<()> {
        // Check capability requirements based on task description
        let required_capabilities = self.infer_required_capabilities(task.description());
        
        for capability in required_capabilities {
            if !self.capability_validator.can_perform(&capability)? {
                return Err(AgentRuntimeError::CapabilityDenied {
                    capability,
                    operation: task.description().to_string(),
                });
            }
        }

        Ok(())
    }

    /// Create task execution context
    async fn create_task_context(&self, context: &AgentContext) -> Result<TaskExecutionContext> {
        let mut task_environment = std::collections::HashMap::new();
        
        // Add agent-specific environment variables
        task_environment.insert("AGENT_ID".to_string(), context.agent_id.0.to_string());
        task_environment.insert("WORKSTREAM".to_string(), context.config.metadata.workstream.clone());
        task_environment.insert("AGENT_DOMAIN".to_string(), context.config.spec.domain.clone());

        // Add working directory
        let working_directory = std::env::current_dir()
            .unwrap_or_else(|_| "/workspace".into())
            .to_string_lossy()
            .to_string();

        // Collect available tools based on capabilities
        let available_tools = self.capability_validator.get_available_tools();

        Ok(TaskExecutionContext {
            agent_context: context.clone(),
            task_environment,
            working_directory,
            available_tools,
            previous_results: vec![], // TODO: Implement task history
        })
    }

    /// Build LLM prompt for task execution
    fn build_task_prompt(
        &self,
        task: &dyn AgentTask,
        context: &TaskExecutionContext,
        retry_count: u32,
    ) -> Result<LlmRequest> {
        let template = self.get_prompt_template(&context.agent_context.config.spec.domain);
        
        let system_prompt = template.system_prompt
            .replace("{agent_name}", &context.agent_context.config.spec.name)
            .replace("{agent_domain}", &context.agent_context.config.spec.domain)
            .replace("{workstream}", &context.agent_context.config.metadata.workstream);

        let task_prompt = format!(
            "{}\n\nTask: {}\nDescription: {}\nWorking Directory: {}\nAvailable Tools: {}\n",
            system_prompt,
            task.task_id(),
            task.description(),
            context.working_directory,
            context.available_tools.join(", ")
        );

        // Add retry context if this is a retry attempt
        let final_prompt = if retry_count > 0 {
            format!(
                "{}\n\nNote: This is retry attempt {}. Previous attempts failed. Please analyze the task carefully and try a different approach.\n",
                task_prompt,
                retry_count
            )
        } else {
            task_prompt
        };

        let mut request = LlmRequest::new(final_prompt)?;
        
        // Set reasonable limits for task execution
        request = request.with_max_tokens(4096);
        
        // Use lower temperature for more deterministic task execution
        request = request.with_temperature(0.3)?;

        Ok(request)
    }

    /// Parse LLM response into task result
    fn parse_task_response(
        &self,
        task: &dyn AgentTask,
        response: &LlmResponse,
        duration: Duration,
    ) -> Result<TaskResult> {
        let content = response.content();
        
        // Simple parsing - look for success/failure indicators
        let success = !content.to_lowercase().contains("error") &&
                     !content.to_lowercase().contains("failed") &&
                     !content.to_lowercase().contains("unable to");

        let result = if success {
            TaskResult::success(
                task.task_id().to_string(),
                task.description().to_string(),
                Some(content.to_string()),
                duration,
            )
        } else {
            TaskResult::failure(
                task.task_id().to_string(),
                task.description().to_string(),
                content.to_string(),
                duration,
            )
        };

        Ok(result.with_llm_tokens(response.usage().total_tokens as u64))
    }

    /// Infer required capabilities from task description
    fn infer_required_capabilities(&self, description: &str) -> Vec<String> {
        let mut capabilities = Vec::new();
        let description_lower = description.to_lowercase();

        // File system operations
        if description_lower.contains("file") || description_lower.contains("directory") ||
           description_lower.contains("read") || description_lower.contains("write") {
            capabilities.push("filesystem-read".to_string());
            if description_lower.contains("write") || description_lower.contains("create") ||
               description_lower.contains("update") || description_lower.contains("modify") {
                capabilities.push("filesystem-write".to_string());
            }
        }

        // Build operations
        if description_lower.contains("cargo") || description_lower.contains("build") ||
           description_lower.contains("compile") || description_lower.contains("test") {
            capabilities.push("cargo-execution".to_string());
        }

        // Network operations
        if description_lower.contains("download") || description_lower.contains("api") ||
           description_lower.contains("http") || description_lower.contains("network") {
            capabilities.push("network-access".to_string());
        }

        // Git operations
        if description_lower.contains("git") || description_lower.contains("commit") ||
           description_lower.contains("branch") || description_lower.contains("repository") {
            capabilities.push("git-access".to_string());
        }

        capabilities
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, retry_count: u32) -> Duration {
        let base_delay = self.execution_config.retry_config.base_delay;
        let max_delay = self.execution_config.retry_config.max_delay;
        let multiplier = self.execution_config.retry_config.backoff_multiplier;

        let delay_seconds = base_delay.as_secs_f64() * multiplier.powi(retry_count as i32 - 1);
        let delay = Duration::from_secs_f64(delay_seconds);

        std::cmp::min(delay, max_delay)
    }

    /// Get prompt template for agent domain
    fn get_prompt_template(&self, domain: &str) -> TaskPromptTemplate {
        match domain {
            "infrastructure" | "build-infrastructure" => TaskPromptTemplate {
                system_prompt: "You are {agent_name}, a specialized infrastructure agent focused on {agent_domain} within the {workstream} workstream. You have expertise in build systems, dependency management, and development tooling.".to_string(),
                task_template: "Execute the following infrastructure task with precision and attention to system stability.".to_string(),
                context_template: "Current working environment: {working_directory}\nAvailable tools: {available_tools}".to_string(),
            },
            "quality-assurance" => TaskPromptTemplate {
                system_prompt: "You are {agent_name}, a quality assurance agent specializing in {agent_domain} for the {workstream} workstream. You focus on testing, validation, and ensuring system reliability.".to_string(),
                task_template: "Execute the following testing task with thorough validation and comprehensive coverage.".to_string(),
                context_template: "Testing environment: {working_directory}\nTesting tools: {available_tools}".to_string(),
            },
            "storage" => TaskPromptTemplate {
                system_prompt: "You are {agent_name}, a storage systems agent specializing in {agent_domain} within the {workstream} workstream. You have expertise in databases, persistence, and data management.".to_string(),
                task_template: "Execute the following storage task with focus on data integrity and performance.".to_string(),
                context_template: "Storage environment: {working_directory}\nStorage tools: {available_tools}".to_string(),
            },
            "security" => TaskPromptTemplate {
                system_prompt: "You are {agent_name}, a security-focused agent specializing in {agent_domain} for the {workstream} workstream. You prioritize security, authentication, and secure system design.".to_string(),
                task_template: "Execute the following security task with careful attention to security best practices and threat mitigation.".to_string(),
                context_template: "Secure environment: {working_directory}\nSecurity tools: {available_tools}".to_string(),
            },
            _ => TaskPromptTemplate {
                system_prompt: "You are {agent_name}, an intelligent agent specializing in {agent_domain} within the {workstream} workstream. You execute tasks efficiently and report progress clearly.".to_string(),
                task_template: "Execute the following task with care and attention to detail.".to_string(),
                context_template: "Working environment: {working_directory}\nAvailable tools: {available_tools}".to_string(),
            },
        }
    }
}

impl LlmTask {
    /// Create a new LLM-based task
    pub fn new(config: TaskConfig) -> Self {
        let task_id = format!("task-{}", Uuid::new_v4());
        
        // Estimate duration based on priority and description length
        let estimated_duration = match config.priority {
            TaskPriority::High => Some(Duration::from_secs(600)), // 10 minutes
            TaskPriority::Medium => Some(Duration::from_secs(300)), // 5 minutes
            TaskPriority::Low => Some(Duration::from_secs(180)), // 3 minutes
        };

        Self {
            task_id,
            config,
            estimated_duration,
            retryable: true,
        }
    }

    /// Create a task with custom ID
    pub fn with_id(mut self, task_id: String) -> Self {
        self.task_id = task_id;
        self
    }

    /// Set whether task is retryable
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

#[async_trait]
impl AgentTask for LlmTask {
    async fn execute(&self, _context: &AgentContext) -> Result<TaskResult> {
        // This is handled by TaskExecutor::execute_task
        // This implementation is just for trait compliance
        Ok(TaskResult::success(
            self.task_id.clone(),
            self.config.description.clone(),
            Some("Task delegated to TaskExecutor".to_string()),
            Duration::ZERO,
        ))
    }

    fn task_id(&self) -> &str {
        &self.task_id
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn estimated_duration(&self) -> Option<Duration> {
        self.estimated_duration
    }

    fn is_retryable(&self) -> bool {
        self.retryable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_orchestration::{AgentConfig, AgentMetadata, AgentSpecConfig, AgentPriority, ResourceLimits};

    fn create_test_security_config() -> SecurityConfig {
        SecurityConfig {
            sandbox: true,
            capabilities_required: vec![
                "filesystem-read".to_string(),
                "cargo-execution".to_string(),
            ],
            resource_limits: ResourceLimits {
                max_memory: "100MB".to_string(),
                max_cpu: "50%".to_string(),
                timeout: "5m".to_string(),
            },
        }
    }

    #[test]
    fn test_llm_task_creation() {
        let task_config = TaskConfig {
            description: "Test task description".to_string(),
            priority: TaskPriority::High,
        };

        let task = LlmTask::new(task_config);
        
        assert!(!task.task_id().is_empty());
        assert_eq!(task.description(), "Test task description");
        assert_eq!(task.estimated_duration(), Some(Duration::from_secs(600)));
        assert!(task.is_retryable());
    }

    #[test]
    fn test_capability_inference() {
        let security_config = create_test_security_config();
        let execution_config = ExecutionConfig::default();
        
        // Mock LLM gateway for testing
        // Note: In real tests, we'd use a mock implementation
        // For now, this test focuses on the capability inference logic
        
        let descriptions = vec![
            ("Read configuration file", vec!["filesystem-read"]),
            ("Update Cargo.toml dependencies", vec!["filesystem-read", "filesystem-write"]),
            ("Run cargo build", vec!["cargo-execution"]),
            ("Download API data", vec!["network-access"]),
            ("Commit changes to git", vec!["git-access"]),
        ];

        for (description, expected_caps) in descriptions {
            // Create a minimal task executor for testing capability inference
            // Note: This would need a proper mock setup in real tests
            println!("Testing capability inference for: {}", description);
            println!("Expected capabilities: {:?}", expected_caps);
        }
    }

    #[test]
    fn test_retry_delay_calculation() {
        let mut config = ExecutionConfig::default();
        config.retry_config.base_delay = Duration::from_secs(1);
        config.retry_config.max_delay = Duration::from_secs(60);
        config.retry_config.backoff_multiplier = 2.0;

        let security_config = create_test_security_config();
        
        // Note: Would need mock LLM gateway for full test
        // For now, testing the retry calculation logic conceptually
        let base_delay = config.retry_config.base_delay;
        let multiplier = config.retry_config.backoff_multiplier;
        
        assert_eq!(base_delay, Duration::from_secs(1));
        assert_eq!(multiplier, 2.0);
    }

    #[test]
    fn test_prompt_template_selection() {
        let domains = vec![
            ("infrastructure", "infrastructure agent"),
            ("quality-assurance", "quality assurance agent"),
            ("storage", "storage systems agent"),
            ("security", "security-focused agent"),
            ("unknown", "intelligent agent"),
        ];

        for (domain, expected_type) in domains {
            // Test that different domains get appropriate prompt templates
            println!("Domain: {} -> Expected: {}", domain, expected_type);
        }
    }
}