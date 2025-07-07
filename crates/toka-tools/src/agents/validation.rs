//! Agent specification validation
//!
//! This module provides validation functionality for agent specifications,
//! ensuring they comply with the canonical schema and meet quality standards.

use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use regex::Regex;
use tokio::fs;

use super::specification::*;

/// Agent specification validator
pub struct AgentValidator {
    schema_rules: ValidationRules,
    custom_validators: HashMap<String, Box<dyn CustomValidator>>,
}

/// Schema validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub metadata_rules: MetadataRules,
    pub spec_rules: SpecRules,
    pub capability_rules: CapabilityRules,
    pub security_rules: SecurityRules,
    pub dependency_rules: DependencyRules,
}

/// Metadata validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRules {
    pub name_pattern: String,
    pub version_pattern: String,
    pub branch_pattern: String,
    pub workstream_min_length: usize,
    pub workstream_max_length: usize,
}

/// Spec validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecRules {
    pub name_min_length: usize,
    pub name_max_length: usize,
    pub description_min_length: usize,
    pub description_max_length: usize,
    pub valid_domains: Vec<String>,
    pub valid_priorities: Vec<String>,
}

/// Capability validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRules {
    pub capability_pattern: String,
    pub primary_min_count: usize,
    pub primary_max_count: usize,
    pub secondary_max_count: usize,
}

/// Security validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRules {
    pub memory_pattern: String,
    pub cpu_pattern: String,
    pub timeout_pattern: String,
    pub valid_capabilities: Vec<String>,
}

/// Dependency validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRules {
    pub dependency_name_pattern: String,
    pub max_required_dependencies: usize,
    pub max_optional_dependencies: usize,
}

/// Custom validator trait
pub trait CustomValidator: Send + Sync {
    fn validate(&self, spec: &AgentSpec) -> Result<ValidationResult>;
    fn name(&self) -> &str;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub score: f64,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: String,
    pub severity: ErrorSeverity,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: String,
    pub suggestion: Option<String>,
}

/// Error severity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Schema validator for JSON schema compliance
pub struct SchemaValidator {
    schema_path: std::path::PathBuf,
    schema_content: Option<String>,
}

impl AgentValidator {
    /// Create a new validator
    pub async fn new() -> Result<Self> {
        let schema_rules = ValidationRules::default();
        
        Ok(Self {
            schema_rules,
            custom_validators: HashMap::new(),
        })
    }

    /// Create validator with custom rules
    pub async fn with_rules(rules: ValidationRules) -> Result<Self> {
        Ok(Self {
            schema_rules: rules,
            custom_validators: HashMap::new(),
        })
    }

    /// Validate agent specification
    pub fn validate_spec(&self, spec: &AgentSpec) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            score: 100.0,
        };

        // Validate metadata
        self.validate_metadata(&spec.metadata, &mut result)?;

        // Validate spec details
        self.validate_spec_details(&spec.spec, &mut result)?;

        // Validate capabilities
        self.validate_capabilities(&spec.capabilities, &mut result)?;

        // Validate objectives
        self.validate_objectives(&spec.objectives, &mut result)?;

        // Validate tasks
        self.validate_tasks(&spec.tasks, &mut result)?;

        // Validate dependencies
        self.validate_dependencies(&spec.dependencies, &mut result)?;

        // Validate security
        self.validate_security(&spec.security, &mut result)?;

        // Validate behavioral directives
        self.validate_behavioral_directives(&spec.behavioral_directives, &mut result)?;

        // Validate risk mitigation
        self.validate_risk_mitigation(&spec.risk_mitigation, &mut result)?;

        // Validate success criteria
        self.validate_success_criteria(&spec.success_criteria, &mut result)?;

        // Run custom validators
        for validator in self.custom_validators.values() {
            let custom_result = validator.validate(spec)?;
            result.merge(custom_result);
        }

        // Calculate final score
        result.score = self.calculate_score(&result);
        result.valid = result.errors.is_empty();

        Ok(result)
    }

    /// Add custom validator
    pub fn add_custom_validator(&mut self, validator: Box<dyn CustomValidator>) {
        self.custom_validators.insert(validator.name().to_string(), validator);
    }

    /// Validate metadata
    fn validate_metadata(&self, metadata: &AgentMetadata, result: &mut ValidationResult) -> Result<()> {
        let name_regex = Regex::new(&self.schema_rules.metadata_rules.name_pattern)?;
        if !name_regex.is_match(&metadata.name) {
            result.add_error(
                "invalid_name_format",
                "Agent name must be in kebab-case format",
                "metadata.name",
                ErrorSeverity::Critical,
            );
        }

        let version_regex = Regex::new(&self.schema_rules.metadata_rules.version_pattern)?;
        if !version_regex.is_match(&metadata.version) {
            result.add_error(
                "invalid_version_format",
                "Version must follow vX.Y.Z format",
                "metadata.version",
                ErrorSeverity::Critical,
            );
        }

        let branch_regex = Regex::new(&self.schema_rules.metadata_rules.branch_pattern)?;
        if !branch_regex.is_match(&metadata.branch) {
            result.add_error(
                "invalid_branch_format",
                "Branch must follow feature/kebab-case format",
                "metadata.branch",
                ErrorSeverity::High,
            );
        }

        let workstream_len = metadata.workstream.len();
        if workstream_len < self.schema_rules.metadata_rules.workstream_min_length {
            result.add_error(
                "workstream_too_short",
                "Workstream name is too short",
                "metadata.workstream",
                ErrorSeverity::Medium,
            );
        }
        if workstream_len > self.schema_rules.metadata_rules.workstream_max_length {
            result.add_error(
                "workstream_too_long",
                "Workstream name is too long",
                "metadata.workstream",
                ErrorSeverity::Medium,
            );
        }

        Ok(())
    }

    /// Validate spec details
    fn validate_spec_details(&self, spec: &AgentSpecDetails, result: &mut ValidationResult) -> Result<()> {
        let name_len = spec.name.len();
        if name_len < self.schema_rules.spec_rules.name_min_length {
            result.add_error(
                "name_too_short",
                "Agent name is too short",
                "spec.name",
                ErrorSeverity::Medium,
            );
        }
        if name_len > self.schema_rules.spec_rules.name_max_length {
            result.add_error(
                "name_too_long",
                "Agent name is too long",
                "spec.name",
                ErrorSeverity::Medium,
            );
        }

        if let Some(description) = &spec.description {
            let desc_len = description.len();
            if desc_len < self.schema_rules.spec_rules.description_min_length {
                result.add_warning(
                    "description_too_short",
                    "Agent description is too short",
                    "spec.description",
                    Some("Consider adding more details about the agent's purpose".to_string()),
                );
            }
            if desc_len > self.schema_rules.spec_rules.description_max_length {
                result.add_error(
                    "description_too_long",
                    "Agent description is too long",
                    "spec.description",
                    ErrorSeverity::Medium,
                );
            }
        }

        Ok(())
    }

    /// Validate capabilities
    fn validate_capabilities(&self, capabilities: &AgentCapabilities, result: &mut ValidationResult) -> Result<()> {
        let capability_regex = Regex::new(&self.schema_rules.capability_rules.capability_pattern)?;

        if capabilities.primary.len() < self.schema_rules.capability_rules.primary_min_count {
            result.add_error(
                "insufficient_primary_capabilities",
                "Agent must have at least one primary capability",
                "capabilities.primary",
                ErrorSeverity::Critical,
            );
        }

        if capabilities.primary.len() > self.schema_rules.capability_rules.primary_max_count {
            result.add_error(
                "too_many_primary_capabilities",
                "Agent has too many primary capabilities",
                "capabilities.primary",
                ErrorSeverity::Medium,
            );
        }

        if capabilities.secondary.len() > self.schema_rules.capability_rules.secondary_max_count {
            result.add_error(
                "too_many_secondary_capabilities",
                "Agent has too many secondary capabilities",
                "capabilities.secondary",
                ErrorSeverity::Medium,
            );
        }

        for capability in &capabilities.primary {
            if !capability_regex.is_match(capability) {
                result.add_error(
                    "invalid_capability_format",
                    "Capability must be in kebab-case format",
                    "capabilities.primary",
                    ErrorSeverity::High,
                );
            }
        }

        for capability in &capabilities.secondary {
            if !capability_regex.is_match(capability) {
                result.add_error(
                    "invalid_capability_format",
                    "Capability must be in kebab-case format",
                    "capabilities.secondary",
                    ErrorSeverity::High,
                );
            }
        }

        Ok(())
    }

    /// Validate objectives
    fn validate_objectives(&self, objectives: &[AgentObjective], result: &mut ValidationResult) -> Result<()> {
        if objectives.is_empty() {
            result.add_error(
                "no_objectives",
                "Agent must have at least one objective",
                "objectives",
                ErrorSeverity::Critical,
            );
        }

        for (i, objective) in objectives.iter().enumerate() {
            if objective.description.len() < 10 {
                result.add_error(
                    "objective_description_too_short",
                    "Objective description is too short",
                    &format!("objectives[{}].description", i),
                    ErrorSeverity::Medium,
                );
            }

            if objective.deliverable.len() < 10 {
                result.add_error(
                    "objective_deliverable_too_short",
                    "Objective deliverable is too short",
                    &format!("objectives[{}].deliverable", i),
                    ErrorSeverity::Medium,
                );
            }

            if objective.validation.len() < 10 {
                result.add_error(
                    "objective_validation_too_short",
                    "Objective validation is too short",
                    &format!("objectives[{}].validation", i),
                    ErrorSeverity::Medium,
                );
            }
        }

        Ok(())
    }

    /// Validate tasks
    fn validate_tasks(&self, tasks: &AgentTasks, result: &mut ValidationResult) -> Result<()> {
        if tasks.default.is_empty() {
            result.add_error(
                "no_default_tasks",
                "Agent must have at least one default task",
                "tasks.default",
                ErrorSeverity::Critical,
            );
        }

        for (i, task) in tasks.default.iter().enumerate() {
            if task.description.len() < 10 {
                result.add_error(
                    "task_description_too_short",
                    "Task description is too short",
                    &format!("tasks.default[{}].description", i),
                    ErrorSeverity::Medium,
                );
            }
        }

        Ok(())
    }

    /// Validate dependencies
    fn validate_dependencies(&self, dependencies: &AgentDependencies, result: &mut ValidationResult) -> Result<()> {
        let dependency_regex = Regex::new(&self.schema_rules.dependency_rules.dependency_name_pattern)?;

        if dependencies.required.len() > self.schema_rules.dependency_rules.max_required_dependencies {
            result.add_warning(
                "too_many_required_dependencies",
                "Agent has many required dependencies",
                "dependencies.required",
                Some("Consider reducing dependencies for better modularity".to_string()),
            );
        }

        if dependencies.optional.len() > self.schema_rules.dependency_rules.max_optional_dependencies {
            result.add_warning(
                "too_many_optional_dependencies",
                "Agent has many optional dependencies",
                "dependencies.optional",
                Some("Consider reducing dependencies for better modularity".to_string()),
            );
        }

        for dep_name in dependencies.required.keys() {
            if !dependency_regex.is_match(dep_name) {
                result.add_error(
                    "invalid_dependency_name",
                    "Dependency name must be in kebab-case format",
                    "dependencies.required",
                    ErrorSeverity::High,
                );
            }
        }

        for dep_name in dependencies.optional.keys() {
            if !dependency_regex.is_match(dep_name) {
                result.add_error(
                    "invalid_dependency_name",
                    "Dependency name must be in kebab-case format",
                    "dependencies.optional",
                    ErrorSeverity::High,
                );
            }
        }

        Ok(())
    }

    /// Validate security
    fn validate_security(&self, security: &AgentSecurity, result: &mut ValidationResult) -> Result<()> {
        let memory_regex = Regex::new(&self.schema_rules.security_rules.memory_pattern)?;
        if !memory_regex.is_match(&security.resource_limits.max_memory) {
            result.add_error(
                "invalid_memory_format",
                "Memory limit must be in valid format (e.g., 256MB, 1GB)",
                "security.resource_limits.max_memory",
                ErrorSeverity::High,
            );
        }

        let cpu_regex = Regex::new(&self.schema_rules.security_rules.cpu_pattern)?;
        if !cpu_regex.is_match(&security.resource_limits.max_cpu) {
            result.add_error(
                "invalid_cpu_format",
                "CPU limit must be in percentage format (e.g., 50%)",
                "security.resource_limits.max_cpu",
                ErrorSeverity::High,
            );
        }

        let timeout_regex = Regex::new(&self.schema_rules.security_rules.timeout_pattern)?;
        if !timeout_regex.is_match(&security.resource_limits.timeout) {
            result.add_error(
                "invalid_timeout_format",
                "Timeout must be in valid format (e.g., 1h, 30m)",
                "security.resource_limits.timeout",
                ErrorSeverity::High,
            );
        }

        for capability in &security.capabilities_required {
            if !self.schema_rules.security_rules.valid_capabilities.contains(capability) {
                result.add_warning(
                    "unknown_capability",
                    "Unknown security capability",
                    "security.capabilities_required",
                    Some(format!("Verify that '{}' is a valid capability", capability)),
                );
            }
        }

        Ok(())
    }

    /// Validate behavioral directives
    fn validate_behavioral_directives(&self, directives: &BehavioralDirectives, result: &mut ValidationResult) -> Result<()> {
        if directives.operational_focus.is_empty() {
            result.add_error(
                "no_operational_focus",
                "Agent must have at least one operational focus directive",
                "behavioral_directives.operational_focus",
                ErrorSeverity::High,
            );
        }

        if directives.error_handling.is_empty() {
            result.add_error(
                "no_error_handling",
                "Agent must have at least one error handling directive",
                "behavioral_directives.error_handling",
                ErrorSeverity::High,
            );
        }

        if directives.coordination.is_empty() {
            result.add_error(
                "no_coordination",
                "Agent must have at least one coordination directive",
                "behavioral_directives.coordination",
                ErrorSeverity::High,
            );
        }

        Ok(())
    }

    /// Validate risk mitigation
    fn validate_risk_mitigation(&self, risk_mitigation: &RiskMitigation, result: &mut ValidationResult) -> Result<()> {
        if risk_mitigation.high_priority_risks.is_empty() {
            result.add_warning(
                "no_high_priority_risks",
                "Agent should identify high priority risks",
                "risk_mitigation.high_priority_risks",
                Some("Consider identifying potential risks and mitigation strategies".to_string()),
            );
        }

        if risk_mitigation.monitoring.is_empty() {
            result.add_warning(
                "no_monitoring",
                "Agent should have monitoring strategies",
                "risk_mitigation.monitoring",
                Some("Consider adding monitoring and alerting strategies".to_string()),
            );
        }

        Ok(())
    }

    /// Validate success criteria
    fn validate_success_criteria(&self, criteria: &SuccessCriteria, result: &mut ValidationResult) -> Result<()> {
        if criteria.final_validation.is_empty() {
            result.add_error(
                "no_final_validation",
                "Agent must have at least one final validation criterion",
                "success_criteria.final_validation",
                ErrorSeverity::High,
            );
        }

        Ok(())
    }

    /// Calculate validation score
    fn calculate_score(&self, result: &ValidationResult) -> f64 {
        let mut score = 100.0;

        for error in &result.errors {
            match error.severity {
                ErrorSeverity::Critical => score -= 25.0,
                ErrorSeverity::High => score -= 15.0,
                ErrorSeverity::Medium => score -= 10.0,
                ErrorSeverity::Low => score -= 5.0,
            }
        }

        for _ in &result.warnings {
            score -= 2.0;
        }

        score.max(0.0)
    }
}

impl ValidationResult {
    /// Add error to result
    pub fn add_error(&mut self, code: &str, message: &str, field: &str, severity: ErrorSeverity) {
        self.errors.push(ValidationError {
            code: code.to_string(),
            message: message.to_string(),
            field: field.to_string(),
            severity,
        });
    }

    /// Add warning to result
    pub fn add_warning(&mut self, code: &str, message: &str, field: &str, suggestion: Option<String>) {
        self.warnings.push(ValidationWarning {
            code: code.to_string(),
            message: message.to_string(),
            field: field.to_string(),
            suggestion,
        });
    }

    /// Merge another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Check if result is valid
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Validation: {} errors, {} warnings, score: {:.1}",
            self.errors.len(),
            self.warnings.len(),
            self.score
        )
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            metadata_rules: MetadataRules {
                name_pattern: r"^[a-z][a-z0-9-]*[a-z0-9]$".to_string(),
                version_pattern: r"^v\d+\.\d+\.\d+$".to_string(),
                branch_pattern: r"^feature/[a-z][a-z0-9-]*[a-z0-9]$".to_string(),
                workstream_min_length: 5,
                workstream_max_length: 100,
            },
            spec_rules: SpecRules {
                name_min_length: 5,
                name_max_length: 100,
                description_min_length: 10,
                description_max_length: 500,
                valid_domains: vec![
                    "infrastructure".to_string(),
                    "devops-infrastructure".to_string(),
                    "quality-assurance".to_string(),
                    "kernel-architecture".to_string(),
                    "storage-architecture".to_string(),
                    "security".to_string(),
                    "operations".to_string(),
                    "ai-integration".to_string(),
                    "documentation".to_string(),
                    "github-integration".to_string(),
                    "api-management".to_string(),
                    "cli-tooling".to_string(),
                ],
                valid_priorities: vec![
                    "critical".to_string(),
                    "high".to_string(),
                    "medium".to_string(),
                    "low".to_string(),
                ],
            },
            capability_rules: CapabilityRules {
                capability_pattern: r"^[a-z][a-z0-9-]*[a-z0-9]$".to_string(),
                primary_min_count: 1,
                primary_max_count: 10,
                secondary_max_count: 15,
            },
            security_rules: SecurityRules {
                memory_pattern: r"^\d+[KMGT]?B$".to_string(),
                cpu_pattern: r"^\d+%$".to_string(),
                timeout_pattern: r"^\d+[smhd]$".to_string(),
                valid_capabilities: vec![
                    "filesystem-read".to_string(),
                    "filesystem-write".to_string(),
                    "network-access".to_string(),
                    "github-api-access".to_string(),
                    "database-access".to_string(),
                    "cargo-execution".to_string(),
                    "git-access".to_string(),
                    "ci-integration".to_string(),
                    "admin-privileges".to_string(),
                ],
            },
            dependency_rules: DependencyRules {
                dependency_name_pattern: r"^[a-z][a-z0-9-]*[a-z0-9]$".to_string(),
                max_required_dependencies: 5,
                max_optional_dependencies: 10,
            },
        }
    }
}

impl SchemaValidator {
    /// Create new schema validator
    pub fn new() -> Self {
        Self {
            schema_path: std::path::PathBuf::from(".cursor/schemas/agent-spec-schema.yaml"),
            schema_content: None,
        }
    }

    /// Create schema validator with custom path
    pub fn with_schema_path(path: std::path::PathBuf) -> Self {
        Self {
            schema_path: path,
            schema_content: None,
        }
    }

    /// Load schema from file
    pub async fn load_schema(&mut self) -> Result<()> {
        if self.schema_path.exists() {
            let content = fs::read_to_string(&self.schema_path).await?;
            self.schema_content = Some(content);
        }
        Ok(())
    }

    /// Validate agent spec against schema
    pub fn validate_against_schema(&self, _spec: &AgentSpec) -> Result<ValidationResult> {
        // In a real implementation, this would use a JSON schema validator
        // For now, return a simple success result
        Ok(ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
            score: 100.0,
        })
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}