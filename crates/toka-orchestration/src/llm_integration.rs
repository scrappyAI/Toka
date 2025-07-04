//! LLM integration for intelligent agent orchestration.
//!
//! This module provides integration between the orchestration system and the
//! LLM gateway, enabling agents to use language models for intelligent task
//! execution, problem-solving, and coordination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, info};

use toka_llm_gateway::{LlmGateway, LlmRequest, LlmResponse};
use toka_types::TaskSpec;

use crate::{AgentConfig, SpawnedAgent, OrchestrationPhase};

/// LLM integration coordinator for orchestration.
pub struct LlmOrchestrationIntegrator {
    /// LLM gateway instance
    llm_gateway: Arc<LlmGateway>,
    /// Agent LLM contexts
    agent_contexts: Arc<RwLock<HashMap<String, AgentLlmContext>>>,
    /// Orchestration prompts and templates
    prompt_templates: PromptTemplates,
    /// LLM usage metrics
    usage_metrics: Arc<RwLock<LlmUsageMetrics>>,
}

/// LLM context for an individual agent.
#[derive(Debug, Clone)]
pub struct AgentLlmContext {
    /// Agent identifier
    pub agent_name: String,
    /// Agent workstream
    pub workstream: String,
    /// Current task context
    pub current_task: Option<String>,
    /// Task history
    pub task_history: Vec<TaskExecutionRecord>,
    /// Agent-specific prompt context
    pub prompt_context: String,
    /// Last LLM interaction
    pub last_interaction: Option<DateTime<Utc>>,
    /// LLM usage for this agent
    pub usage_stats: AgentLlmStats,
}

/// Task execution record for LLM context.
#[derive(Debug, Clone)]
pub struct TaskExecutionRecord {
    /// Task description
    pub task: String,
    /// LLM request used
    pub request: String,
    /// LLM response received
    pub response: String,
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether task was successful
    pub success: bool,
    /// Execution notes
    pub notes: String,
}

/// LLM usage statistics for an agent.
#[derive(Debug, Clone, Default)]
pub struct AgentLlmStats {
    /// Total requests made
    pub total_requests: usize,
    /// Total tokens consumed
    pub total_tokens: usize,
    /// Successful requests
    pub successful_requests: usize,
    /// Failed requests
    pub failed_requests: usize,
    /// Average response time
    pub avg_response_time: Duration,
    /// Last request timestamp
    pub last_request: Option<DateTime<Utc>>,
}

/// Overall LLM usage metrics.
#[derive(Debug, Clone, Default)]
pub struct LlmUsageMetrics {
    /// Total requests across all agents
    pub total_requests: usize,
    /// Total tokens consumed
    pub total_tokens: usize,
    /// Requests by workstream
    pub requests_by_workstream: HashMap<String, usize>,
    /// Tokens by workstream
    pub tokens_by_workstream: HashMap<String, usize>,
    /// Average response time
    pub avg_response_time: Duration,
    /// Peak usage periods
    pub peak_usage: Vec<UsagePeak>,
}

/// Usage peak tracking.
#[derive(Debug, Clone)]
pub struct UsagePeak {
    /// Peak timestamp
    pub timestamp: DateTime<Utc>,
    /// Requests per minute
    pub requests_per_minute: usize,
    /// Tokens per minute
    pub tokens_per_minute: usize,
    /// Workstreams active during peak
    pub active_workstreams: Vec<String>,
}

/// Prompt templates for orchestration tasks.
pub struct PromptTemplates {
    /// Task execution prompts
    pub task_execution: HashMap<String, String>,
    /// Coordination prompts
    pub coordination: HashMap<String, String>,
    /// Problem-solving prompts
    pub problem_solving: HashMap<String, String>,
    /// Planning prompts
    pub planning: HashMap<String, String>,
}

impl LlmOrchestrationIntegrator {
    /// Create a new LLM orchestration integrator.
    pub fn new(llm_gateway: Arc<LlmGateway>) -> Self {
        Self {
            llm_gateway,
            agent_contexts: Arc::new(RwLock::new(HashMap::new())),
            prompt_templates: PromptTemplates::default(),
            usage_metrics: Arc::new(RwLock::new(LlmUsageMetrics::default())),
        }
    }

    /// Initialize LLM contexts for agents.
    pub async fn initialize_agent_contexts(&self, agents: &[AgentConfig]) -> Result<()> {
        info!("Initializing LLM contexts for {} agents", agents.len());

        let mut contexts = self.agent_contexts.write().await;

        for agent in agents {
            let context = AgentLlmContext {
                agent_name: agent.metadata.name.clone(),
                workstream: agent.metadata.workstream.clone(),
                current_task: None,
                task_history: Vec::new(),
                prompt_context: self.build_agent_prompt_context(agent),
                last_interaction: None,
                usage_stats: AgentLlmStats::default(),
            };

            contexts.insert(agent.metadata.name.clone(), context);
        }

        info!("LLM contexts initialized for {} agents", agents.len());
        Ok(())
    }

    /// Execute a task using LLM assistance.
    pub async fn execute_task_with_llm(
        &self,
        agent_name: &str,
        task: &TaskSpec,
        context: Option<&str>,
    ) -> Result<TaskExecutionResult> {
        debug!("Executing task with LLM for agent {}: {}", agent_name, task.description);

        // Get agent context
        let agent_context = {
            let contexts = self.agent_contexts.read().await;
            contexts.get(agent_name).cloned()
                .ok_or_else(|| anyhow::anyhow!("Agent context not found: {}", agent_name))?
        };

        // Build LLM request
        let prompt = self.build_task_execution_prompt(task, &agent_context, context)?;
        let request = LlmRequest::new(prompt)?;

        // Execute LLM request
        let start_time = std::time::Instant::now();
        let response = self.llm_gateway.complete(request.clone()).await?;
        let execution_time = start_time.elapsed();

        // Process response
        let result = self.process_task_response(&response, task, &agent_context).await?;

        // Update agent context
        self.update_agent_context_after_task(
            agent_name,
            task,
            &request.prompt(),
            response.content(),
            &result,
            execution_time,
        ).await?;

        // Update usage metrics
        self.update_usage_metrics(&agent_context.workstream, response.usage(), execution_time).await;

        debug!("Task execution completed for agent {}", agent_name);
        Ok(result)
    }

    /// Generate a coordination plan using LLM.
    pub async fn generate_coordination_plan(
        &self,
        phase: &OrchestrationPhase,
        agents: &[SpawnedAgent],
        context: &str,
    ) -> Result<CoordinationPlan> {
        info!("Generating coordination plan for phase: {:?}", phase);

        let prompt = self.build_coordination_prompt(phase, agents, context)?;
        let request = LlmRequest::new(prompt)?;

        let response = self.llm_gateway.complete(request).await?;
        let plan = self.parse_coordination_plan(response.content())?;

        info!("Coordination plan generated for {} agents", agents.len());
        Ok(plan)
    }

    /// Solve orchestration problems using LLM.
    pub async fn solve_orchestration_problem(
        &self,
        problem_description: &str,
        context: &str,
        constraints: &[String],
    ) -> Result<ProblemSolution> {
        info!("Solving orchestration problem: {}", problem_description);

        let prompt = self.build_problem_solving_prompt(problem_description, context, constraints)?;
        let request = LlmRequest::new(prompt)?;

        let response = self.llm_gateway.complete(request).await?;
        let solution = self.parse_problem_solution(response.content())?;

        info!("Problem solution generated");
        Ok(solution)
    }

    /// Get LLM usage metrics.
    pub async fn get_usage_metrics(&self) -> LlmUsageMetrics {
        self.usage_metrics.read().await.clone()
    }

    /// Get agent LLM context.
    pub async fn get_agent_context(&self, agent_name: &str) -> Option<AgentLlmContext> {
        let contexts = self.agent_contexts.read().await;
        contexts.get(agent_name).cloned()
    }

    /// Build agent-specific prompt context.
    fn build_agent_prompt_context(&self, agent: &AgentConfig) -> String {
        format!(
            "Agent: {}\nWorkstream: {}\nDomain: {}\nPrimary Capabilities: {}\nObjectives:\n{}",
            agent.spec.name,
            agent.metadata.workstream,
            agent.spec.domain,
            agent.capabilities.primary.join(", "),
            agent.objectives.iter()
                .map(|obj| format!("- {}: {}", obj.description, obj.deliverable))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Build task execution prompt.
    fn build_task_execution_prompt(
        &self,
        task: &TaskSpec,
        agent_context: &AgentLlmContext,
        additional_context: Option<&str>,
    ) -> Result<String> {
        let base_template = self.prompt_templates.task_execution
            .get("default")
            .ok_or_else(|| anyhow::anyhow!("No default task execution template"))?;

        let context_section = additional_context
            .map(|ctx| format!("\nAdditional Context:\n{}", ctx))
            .unwrap_or_default();

        let history_section = if !agent_context.task_history.is_empty() {
            let recent_history = agent_context.task_history.iter()
                .rev()
                .take(3)
                .map(|record| format!("- {}: {}", record.task, if record.success { "SUCCESS" } else { "FAILED" }))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\nRecent Task History:\n{}", recent_history)
        } else {
            String::new()
        };

        Ok(format!(
            "{}\n\nAgent Context:\n{}\n\nTask: {}\n{}{}",
            base_template,
            agent_context.prompt_context,
            task.description,
            context_section,
            history_section
        ))
    }

    /// Build coordination prompt.
    fn build_coordination_prompt(
        &self,
        phase: &OrchestrationPhase,
        agents: &[SpawnedAgent],
        context: &str,
    ) -> Result<String> {
        let template = self.prompt_templates.coordination
            .get("default")
            .ok_or_else(|| anyhow::anyhow!("No default coordination template"))?;

        let agents_info = agents.iter()
            .map(|agent| format!(
                "- {}: {} (State: {:?}, Progress: {:.1}%)",
                agent.config.metadata.name,
                agent.config.spec.domain,
                agent.state,
                agent.metrics.tasks_completed as f64 / agent.metrics.tasks_assigned.max(1) as f64 * 100.0
            ))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            "{}\n\nOrchestration Phase: {:?}\n\nAgents:\n{}\n\nContext:\n{}",
            template, phase, agents_info, context
        ))
    }

    /// Build problem-solving prompt.
    fn build_problem_solving_prompt(
        &self,
        problem: &str,
        context: &str,
        constraints: &[String],
    ) -> Result<String> {
        let template = self.prompt_templates.problem_solving
            .get("default")
            .ok_or_else(|| anyhow::anyhow!("No default problem solving template"))?;

        let constraints_section = if !constraints.is_empty() {
            format!("\nConstraints:\n{}", constraints.iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"))
        } else {
            String::new()
        };

        Ok(format!(
            "{}\n\nProblem: {}\n\nContext:\n{}{}",
            template, problem, context, constraints_section
        ))
    }

    /// Process task execution response.
    async fn process_task_response(
        &self,
        response: &LlmResponse,
        _task: &TaskSpec,
        _agent_context: &AgentLlmContext,
    ) -> Result<TaskExecutionResult> {
        // Parse the LLM response to extract task execution details
        let content = response.content();
        
        // Simple parsing - in a real implementation, this would be more sophisticated
        let success = !content.to_lowercase().contains("error") && 
                     !content.to_lowercase().contains("failed") &&
                     !content.to_lowercase().contains("cannot");

        Ok(TaskExecutionResult {
            success,
            output: content.to_string(),
            artifacts: Vec::new(), // Would be extracted from response
            next_steps: Vec::new(), // Would be extracted from response
            estimated_duration: Duration::from_secs(3600), // Default estimate
            confidence: if success { 0.8 } else { 0.3 },
        })
    }

    /// Parse coordination plan from LLM response.
    fn parse_coordination_plan(&self, _response: &str) -> Result<CoordinationPlan> {
        // Simple parsing - in a real implementation, this would use structured parsing
        Ok(CoordinationPlan {
            phase_actions: vec![
                CoordinationAction {
                    action_type: ActionType::SpawnAgents,
                    description: "Spawn agents in dependency order".to_string(),
                    timeline: Duration::from_secs(300),
                    dependencies: Vec::new(),
                    agents_involved: Vec::new(),
                }
            ],
            estimated_duration: Duration::from_secs(3600),
            risk_factors: Vec::new(),
            success_criteria: Vec::new(),
        })
    }

    /// Parse problem solution from LLM response.
    fn parse_problem_solution(&self, _response: &str) -> Result<ProblemSolution> {
        // Simple parsing - in a real implementation, this would use structured parsing
        Ok(ProblemSolution {
            solution_steps: vec![
                SolutionStep {
                    description: "Analyze problem root cause".to_string(),
                    estimated_time: Duration::from_secs(600),
                    prerequisites: Vec::new(),
                    expected_outcome: "Problem identified".to_string(),
                }
            ],
            confidence: 0.7,
            alternative_approaches: Vec::new(),
            estimated_effort: Duration::from_secs(1800),
        })
    }

    /// Update agent context after task execution.
    async fn update_agent_context_after_task(
        &self,
        agent_name: &str,
        task: &TaskSpec,
        request: &str,
        response: &str,
        result: &TaskExecutionResult,
        execution_time: Duration,
    ) -> Result<()> {
        let mut contexts = self.agent_contexts.write().await;
        
        if let Some(context) = contexts.get_mut(agent_name) {
            // Add to task history
            context.task_history.push(TaskExecutionRecord {
                task: task.description.clone(),
                request: request.to_string(),
                response: response.to_string(),
                timestamp: Utc::now(),
                success: result.success,
                notes: result.output.clone(),
            });

            // Update stats
            context.usage_stats.total_requests += 1;
            if result.success {
                context.usage_stats.successful_requests += 1;
            } else {
                context.usage_stats.failed_requests += 1;
            }
            
            // Update average response time
            let current_avg = context.usage_stats.avg_response_time;
            let total_requests = context.usage_stats.total_requests;
            context.usage_stats.avg_response_time = 
                (current_avg * (total_requests - 1) as u32 + execution_time) / total_requests as u32;
            
            context.last_interaction = Some(Utc::now());

            // Keep only recent history
            if context.task_history.len() > 10 {
                context.task_history.drain(0..context.task_history.len() - 10);
            }
        }

        Ok(())
    }

    /// Update global usage metrics.
    async fn update_usage_metrics(
        &self,
        workstream: &str,
        usage: &toka_llm_gateway::TokenUsage,
        execution_time: Duration,
    ) {
        let mut metrics = self.usage_metrics.write().await;
        
        metrics.total_requests += 1;
        metrics.total_tokens += usage.total_tokens as usize;
        
        *metrics.requests_by_workstream.entry(workstream.to_string()).or_insert(0) += 1;
        *metrics.tokens_by_workstream.entry(workstream.to_string()).or_insert(0) += usage.total_tokens as usize;
        
        // Update average response time
        let current_avg = metrics.avg_response_time;
        metrics.avg_response_time = 
            (current_avg * (metrics.total_requests - 1) as u32 + execution_time) / metrics.total_requests as u32;
    }
}

/// Result of task execution with LLM assistance.
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Task output/result
    pub output: String,
    /// Generated artifacts
    pub artifacts: Vec<String>,
    /// Suggested next steps
    pub next_steps: Vec<String>,
    /// Estimated duration for completion
    pub estimated_duration: Duration,
    /// Confidence in the result (0.0 to 1.0)
    pub confidence: f64,
}

/// Coordination plan generated by LLM.
#[derive(Debug, Clone)]
pub struct CoordinationPlan {
    /// Actions to take in this phase
    pub phase_actions: Vec<CoordinationAction>,
    /// Estimated duration for phase
    pub estimated_duration: Duration,
    /// Identified risk factors
    pub risk_factors: Vec<String>,
    /// Success criteria
    pub success_criteria: Vec<String>,
}

/// Individual coordination action.
#[derive(Debug, Clone)]
pub struct CoordinationAction {
    /// Type of action
    pub action_type: ActionType,
    /// Action description
    pub description: String,
    /// Estimated timeline
    pub timeline: Duration,
    /// Action dependencies
    pub dependencies: Vec<String>,
    /// Agents involved
    pub agents_involved: Vec<String>,
}

/// Types of coordination actions.
#[derive(Debug, Clone)]
pub enum ActionType {
    /// Spawn new agents
    SpawnAgents,
    /// Assign tasks
    AssignTasks,
    /// Coordinate between agents
    CoordinateAgents,
    /// Monitor progress
    MonitorProgress,
    /// Resolve conflicts
    ResolveConflicts,
}

/// Problem solution generated by LLM.
#[derive(Debug, Clone)]
pub struct ProblemSolution {
    /// Steps to solve the problem
    pub solution_steps: Vec<SolutionStep>,
    /// Confidence in solution (0.0 to 1.0)
    pub confidence: f64,
    /// Alternative approaches
    pub alternative_approaches: Vec<String>,
    /// Estimated effort required
    pub estimated_effort: Duration,
}

/// Individual solution step.
#[derive(Debug, Clone)]
pub struct SolutionStep {
    /// Step description
    pub description: String,
    /// Estimated time for this step
    pub estimated_time: Duration,
    /// Prerequisites for this step
    pub prerequisites: Vec<String>,
    /// Expected outcome
    pub expected_outcome: String,
}

impl Default for PromptTemplates {
    fn default() -> Self {
        let mut task_execution = HashMap::new();
        task_execution.insert(
            "default".to_string(),
            "You are an intelligent agent assistant helping with task execution. \
             Analyze the given task and provide a detailed execution plan with steps, \
             potential issues, and expected outcomes.".to_string(),
        );

        let mut coordination = HashMap::new();
        coordination.insert(
            "default".to_string(),
            "You are an orchestration coordinator. Analyze the current phase and agent states \
             to provide coordination recommendations, dependency management, and optimization suggestions.".to_string(),
        );

        let mut problem_solving = HashMap::new();
        problem_solving.insert(
            "default".to_string(),
            "You are a problem-solving assistant for agent orchestration. \
             Analyze the given problem and provide step-by-step solutions, \
             considering constraints and potential alternatives.".to_string(),
        );

        let mut planning = HashMap::new();
        planning.insert(
            "default".to_string(),
            "You are a planning assistant for multi-agent orchestration. \
             Create detailed plans with timelines, dependencies, and resource allocation.".to_string(),
        );

        Self {
            task_execution,
            coordination,
            problem_solving,
            planning,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require mock LLM gateway for proper testing
    #[test]
    fn test_prompt_templates_default() {
        let templates = PromptTemplates::default();
        assert!(templates.task_execution.contains_key("default"));
        assert!(templates.coordination.contains_key("default"));
        assert!(templates.problem_solving.contains_key("default"));
        assert!(templates.planning.contains_key("default"));
    }
} 