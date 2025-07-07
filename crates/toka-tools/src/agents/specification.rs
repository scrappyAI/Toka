//! Agent specification types based on the canonical schema
//!
//! This module defines the core agent specification types that follow the
//! JSON schema defined in `.cursor/schemas/agent-spec-schema.yaml`.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Complete agent specification following the canonical schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub metadata: AgentMetadata,
    pub spec: AgentSpecDetails,
    pub capabilities: AgentCapabilities,
    pub objectives: Vec<AgentObjective>,
    pub tasks: AgentTasks,
    pub dependencies: AgentDependencies,
    pub reporting: AgentReporting,
    pub security: AgentSecurity,
    pub behavioral_directives: BehavioralDirectives,
    pub risk_mitigation: RiskMitigation,
    pub success_criteria: SuccessCriteria,
    
    /// Domain-specific extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_integration: Option<GitHubIntegration>,
}

/// Agent metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent identifier in kebab-case format
    pub name: String,
    /// Version following vX.Y.Z format
    pub version: String,
    /// Creation date
    pub created: String,
    /// Human-readable workstream name
    pub workstream: String,
    /// Git branch name for this agent's work
    pub branch: String,
    /// Last modification timestamp (automatically managed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<DateTime<Utc>>,
    /// Schema version this spec follows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// Content checksum for integrity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Agent specification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpecDetails {
    /// Human-readable agent name
    pub name: String,
    /// Agent domain for categorization
    pub domain: AgentDomain,
    /// Agent priority level
    pub priority: AgentPriority,
    /// Detailed description of agent purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Agent domain enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentDomain {
    Infrastructure,
    DevopsInfrastructure,
    QualityAssurance,
    KernelArchitecture,
    StorageArchitecture,
    Security,
    Operations,
    AiIntegration,
    Documentation,
    GithubIntegration,
    ApiManagement,
    CliTooling,
}

/// Agent priority enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Primary capabilities of this agent
    pub primary: Vec<String>,
    /// Secondary/supporting capabilities
    #[serde(default)]
    pub secondary: Vec<String>,
}

/// Agent objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObjective {
    /// Clear objective description
    pub description: String,
    /// Concrete deliverable expected
    pub deliverable: String,
    /// How to validate completion
    pub validation: String,
    /// Objective priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AgentPriority>,
}

/// Agent tasks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTasks {
    /// Default tasks
    pub default: Vec<AgentTask>,
    /// Conditional tasks based on context
    #[serde(default)]
    pub conditional: HashMap<String, Vec<AgentTask>>,
}

/// Individual agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: AgentPriority,
    /// Estimated duration (e.g., 2h, 1d, 1w)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<String>,
    /// Task dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Agent dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDependencies {
    /// Required dependencies (agent-name: reason)
    #[serde(default)]
    pub required: HashMap<String, String>,
    /// Optional dependencies (agent-name: reason)
    #[serde(default)]
    pub optional: HashMap<String, String>,
}

/// Agent reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReporting {
    /// Reporting frequency
    pub frequency: ReportingFrequency,
    /// Reporting channels
    pub channels: Vec<ReportingChannel>,
    /// Metrics to track
    pub metrics: Vec<AgentMetric>,
}

/// Reporting frequency enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportingFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    OnMilestone,
    OnCompletion,
}

/// Reporting channel enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportingChannel {
    MainAgent,
    KernelEvents,
    GithubEvents,
    SecurityEvents,
    PerformanceMetrics,
    Custom,
}

/// Agent metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetric {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: String,
    /// Metric type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub metric_type: Option<MetricType>,
    /// Metric unit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// Metric type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Agent security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSecurity {
    /// Whether agent runs in sandbox
    pub sandbox: bool,
    /// Required system capabilities
    pub capabilities_required: Vec<String>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage
    pub max_memory: String,
    /// Maximum CPU usage
    pub max_cpu: String,
    /// Maximum execution time
    pub timeout: String,
    /// Maximum disk usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_disk_usage: Option<String>,
    /// Maximum network bandwidth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_network_bandwidth: Option<String>,
}

/// Behavioral directives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDirectives {
    /// Core operational principles
    pub operational_focus: Vec<String>,
    /// Error handling principles
    pub error_handling: Vec<String>,
    /// Coordination with other agents
    pub coordination: Vec<String>,
}

/// Risk mitigation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigation {
    /// High priority risks and mitigations
    pub high_priority_risks: Vec<RiskItem>,
    /// Monitoring and alerting strategies
    pub monitoring: Vec<String>,
}

/// Individual risk item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    /// Risk description
    pub risk: String,
    /// Mitigation strategy
    pub mitigation: String,
    /// Risk probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probability: Option<RiskLevel>,
    /// Risk impact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<RiskLevel>,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Success criteria configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Phase 1 success criteria
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_1: Option<Vec<String>>,
    /// Phase 2 success criteria
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_2: Option<Vec<String>>,
    /// Final validation criteria
    pub final_validation: Vec<String>,
}

/// GitHub integration configuration (domain-specific extension)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIntegration {
    /// GitHub API endpoints used
    #[serde(default)]
    pub api_endpoints: Vec<String>,
    /// GitHub webhook events handled
    #[serde(default)]
    pub webhook_events: Vec<String>,
    /// Required GitHub permissions
    #[serde(default)]
    pub permissions: HashMap<String, String>,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limiting: HashMap<String, serde_json::Value>,
}

impl AgentSpec {
    /// Create a new agent specification with minimal required fields
    pub fn new(
        name: String,
        domain: AgentDomain,
        priority: AgentPriority,
        workstream: String,
    ) -> Self {
        let kebab_name = name.to_lowercase().replace(" ", "-");
        
        Self {
            metadata: AgentMetadata {
                name: kebab_name.clone(),
                version: "v1.0.0".to_string(),
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                workstream: workstream.clone(),
                branch: format!("feature/{}", kebab_name),
                modified: None,
                schema_version: Some("1.0.0".to_string()),
                checksum: None,
            },
            spec: AgentSpecDetails {
                name,
                domain,
                priority,
                description: None,
            },
            capabilities: AgentCapabilities {
                primary: vec![],
                secondary: vec![],
            },
            objectives: vec![],
            tasks: AgentTasks {
                default: vec![],
                conditional: HashMap::new(),
            },
            dependencies: AgentDependencies {
                required: HashMap::new(),
                optional: HashMap::new(),
            },
            reporting: AgentReporting {
                frequency: ReportingFrequency::Daily,
                channels: vec![ReportingChannel::MainAgent],
                metrics: vec![],
            },
            security: AgentSecurity {
                sandbox: true,
                capabilities_required: vec!["filesystem-read".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "256MB".to_string(),
                    max_cpu: "25%".to_string(),
                    timeout: "1h".to_string(),
                    max_disk_usage: None,
                    max_network_bandwidth: None,
                },
            },
            behavioral_directives: BehavioralDirectives {
                operational_focus: vec![],
                error_handling: vec![],
                coordination: vec![],
            },
            risk_mitigation: RiskMitigation {
                high_priority_risks: vec![],
                monitoring: vec![],
            },
            success_criteria: SuccessCriteria {
                phase_1: None,
                phase_2: None,
                final_validation: vec![],
            },
            github_integration: None,
        }
    }
    
    /// Add a primary capability
    pub fn with_primary_capability(mut self, capability: String) -> Self {
        self.capabilities.primary.push(capability);
        self
    }
    
    /// Add an objective
    pub fn with_objective(mut self, objective: AgentObjective) -> Self {
        self.objectives.push(objective);
        self
    }
    
    /// Add a task
    pub fn with_task(mut self, task: AgentTask) -> Self {
        self.tasks.default.push(task);
        self
    }
    
    /// Validate the specification against basic requirements
    pub fn validate(&self) -> Result<()> {
        if self.metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Agent name cannot be empty"));
        }
        
        if self.capabilities.primary.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one primary capability"));
        }
        
        if self.objectives.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one objective"));
        }
        
        if self.tasks.default.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one default task"));
        }
        
        Ok(())
    }
}