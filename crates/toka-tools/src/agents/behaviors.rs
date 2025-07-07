//! Agent behavioral directives and risk management
//!
//! This module provides functionality for managing agent behavioral directives,
//! risk mitigation strategies, and success criteria validation.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use super::specification::{BehavioralDirectives, RiskMitigation, SuccessCriteria, RiskItem, RiskLevel};

/// Behavioral directive manager
pub struct BehaviorManager {
    directive_templates: HashMap<String, BehavioralDirectiveTemplate>,
    risk_assessments: HashMap<String, RiskAssessment>,
    success_validators: HashMap<String, SuccessValidator>,
}

/// Template for behavioral directives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDirectiveTemplate {
    pub domain: String,
    pub operational_focus: Vec<String>,
    pub error_handling: Vec<String>,
    pub coordination: Vec<String>,
    pub contextual_variations: HashMap<String, BehavioralDirectives>,
}

/// Risk assessment for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub agent_domain: String,
    pub common_risks: Vec<RiskItem>,
    pub mitigation_strategies: Vec<MitigationStrategy>,
    pub monitoring_requirements: Vec<MonitoringRequirement>,
}

/// Mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub risk_type: String,
    pub strategy: String,
    pub implementation: String,
    pub validation_criteria: Vec<String>,
}

/// Monitoring requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringRequirement {
    pub metric: String,
    pub threshold: String,
    pub alert_condition: String,
    pub escalation_procedure: String,
}

/// Success criteria validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessValidator {
    pub domain: String,
    pub validation_rules: Vec<ValidationRule>,
    pub phase_requirements: HashMap<String, Vec<String>>,
}

/// Validation rule for success criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub validation_type: ValidationType,
    pub criteria: String,
    pub weight: f64,
}

/// Type of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationType {
    Quantitative,
    Qualitative,
    Binary,
    Threshold,
    Trend,
}

/// Context for behavioral adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralContext {
    pub environment: String,
    pub workload: String,
    pub priority: String,
    pub constraints: HashMap<String, String>,
}

impl BehaviorManager {
    /// Create a new behavior manager
    pub fn new() -> Self {
        let mut manager = Self {
            directive_templates: HashMap::new(),
            risk_assessments: HashMap::new(),
            success_validators: HashMap::new(),
        };
        
        manager.load_default_templates();
        
        manager
    }
    
    /// Generate behavioral directives for an agent
    pub fn generate_directives(
        &self,
        domain: &str,
        context: &BehavioralContext,
    ) -> Result<BehavioralDirectives> {
        let template = self.directive_templates.get(domain)
            .ok_or_else(|| anyhow::anyhow!("No template found for domain: {}", domain))?;
        
        let mut directives = BehavioralDirectives {
            operational_focus: template.operational_focus.clone(),
            error_handling: template.error_handling.clone(),
            coordination: template.coordination.clone(),
        };
        
        // Apply contextual variations
        if let Some(variation) = template.contextual_variations.get(&context.environment) {
            directives.operational_focus.extend(variation.operational_focus.clone());
            directives.error_handling.extend(variation.error_handling.clone());
            directives.coordination.extend(variation.coordination.clone());
        }
        
        // Apply priority-based modifications
        self.apply_priority_modifiers(&mut directives, &context.priority)?;
        
        Ok(directives)
    }
    
    /// Generate risk mitigation for an agent
    pub fn generate_risk_mitigation(
        &self,
        domain: &str,
        context: &BehavioralContext,
    ) -> Result<RiskMitigation> {
        let assessment = self.risk_assessments.get(domain)
            .ok_or_else(|| anyhow::anyhow!("No risk assessment found for domain: {}", domain))?;
        
        let mut risks = assessment.common_risks.clone();
        
        // Add context-specific risks
        if context.environment == "production" {
            risks.push(RiskItem {
                risk: "Production system disruption".to_string(),
                mitigation: "Implement comprehensive rollback procedures and monitoring".to_string(),
                probability: Some(RiskLevel::Low),
                impact: Some(RiskLevel::VeryHigh),
            });
        }
        
        let monitoring = assessment.monitoring_requirements.iter()
            .map(|req| req.metric.clone())
            .collect();
        
        Ok(RiskMitigation {
            high_priority_risks: risks,
            monitoring,
        })
    }
    
    /// Generate success criteria for an agent
    pub fn generate_success_criteria(
        &self,
        domain: &str,
        objectives: &[String],
    ) -> Result<SuccessCriteria> {
        let validator = self.success_validators.get(domain)
            .ok_or_else(|| anyhow::anyhow!("No success validator found for domain: {}", domain))?;
        
        let mut criteria = SuccessCriteria {
            phase_1: validator.phase_requirements.get("phase_1").cloned(),
            phase_2: validator.phase_requirements.get("phase_2").cloned(),
            final_validation: vec![],
        };
        
        // Generate final validation criteria based on objectives
        for objective in objectives {
            let validation_rule = self.find_validation_rule(validator, objective)?;
            criteria.final_validation.push(validation_rule.description.clone());
        }
        
        Ok(criteria)
    }
    
    /// Validate success criteria against actual outcomes
    pub fn validate_success(
        &self,
        domain: &str,
        criteria: &SuccessCriteria,
        outcomes: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult> {
        let validator = self.success_validators.get(domain)
            .ok_or_else(|| anyhow::anyhow!("No success validator found for domain: {}", domain))?;
        
        let mut result = ValidationResult {
            passed: 0,
            failed: 0,
            total: 0,
            details: vec![],
        };
        
        // Validate final validation criteria
        for criterion in &criteria.final_validation {
            result.total += 1;
            
            if let Some(rule) = validator.validation_rules.iter().find(|r| r.description == *criterion) {
                if self.validate_rule(rule, outcomes)? {
                    result.passed += 1;
                    result.details.push(format!("✓ {}", criterion));
                } else {
                    result.failed += 1;
                    result.details.push(format!("✗ {}", criterion));
                }
            } else {
                result.failed += 1;
                result.details.push(format!("? {} (no validation rule)", criterion));
            }
        }
        
        Ok(result)
    }
    
    /// Load default behavioral templates
    fn load_default_templates(&mut self) {
        // GitHub Integration template
        self.directive_templates.insert(
            "github-integration".to_string(),
            BehavioralDirectiveTemplate {
                domain: "github-integration".to_string(),
                operational_focus: vec![
                    "Respect GitHub API rate limits and implement intelligent backoff strategies".to_string(),
                    "Ensure all GitHub operations are idempotent and can be safely retried".to_string(),
                    "Maintain comprehensive audit logs of all GitHub API interactions".to_string(),
                ],
                error_handling: vec![
                    "Implement exponential backoff for rate limit and temporary failures".to_string(),
                    "Distinguish between retryable and non-retryable GitHub API errors".to_string(),
                    "Provide clear error messages that help diagnose GitHub integration issues".to_string(),
                ],
                coordination: vec![
                    "Coordinate with CLI tooling agent for consistent GitHub integration".to_string(),
                    "Share GitHub API rate limit information with other GitHub-dependent agents".to_string(),
                ],
                contextual_variations: HashMap::new(),
            }
        );
        
        // Build System template
        self.directive_templates.insert(
            "infrastructure".to_string(),
            BehavioralDirectiveTemplate {
                domain: "infrastructure".to_string(),
                operational_focus: vec![
                    "Prioritize build system stability over feature additions".to_string(),
                    "Ensure backward compatibility with existing developer workflows".to_string(),
                    "Validate changes against all workspace crates before applying".to_string(),
                ],
                error_handling: vec![
                    "Fail fast on dependency conflicts - do not proceed with partial fixes".to_string(),
                    "Rollback changes if any workspace crate fails to build".to_string(),
                    "Alert immediately on CI pipeline failures".to_string(),
                ],
                coordination: vec![
                    "Block other workstreams until critical dependency conflicts resolved".to_string(),
                    "Communicate build status changes to all dependent workstreams".to_string(),
                ],
                contextual_variations: HashMap::new(),
            }
        );
        
        // Load risk assessments
        self.load_risk_assessments();
        
        // Load success validators
        self.load_success_validators();
    }
    
    /// Load risk assessments
    fn load_risk_assessments(&mut self) {
        // GitHub Integration risks
        self.risk_assessments.insert(
            "github-integration".to_string(),
            RiskAssessment {
                agent_domain: "github-integration".to_string(),
                common_risks: vec![
                    RiskItem {
                        risk: "GitHub API rate limits block critical operations".to_string(),
                        mitigation: "Implement intelligent rate limiting with priority queues".to_string(),
                        probability: Some(RiskLevel::Medium),
                        impact: Some(RiskLevel::High),
                    },
                    RiskItem {
                        risk: "GitHub API tokens compromised or expired".to_string(),
                        mitigation: "Implement automatic token rotation and secure storage".to_string(),
                        probability: Some(RiskLevel::Low),
                        impact: Some(RiskLevel::VeryHigh),
                    },
                ],
                mitigation_strategies: vec![],
                monitoring_requirements: vec![
                    MonitoringRequirement {
                        metric: "github_api_rate_limit_remaining".to_string(),
                        threshold: "< 100 requests".to_string(),
                        alert_condition: "Rate limit approaching".to_string(),
                        escalation_procedure: "Throttle non-critical operations".to_string(),
                    },
                ],
            }
        );
        
        // Infrastructure risks
        self.risk_assessments.insert(
            "infrastructure".to_string(),
            RiskAssessment {
                agent_domain: "infrastructure".to_string(),
                common_risks: vec![
                    RiskItem {
                        risk: "Breaking changes to existing build processes".to_string(),
                        mitigation: "Test all changes in isolated environment before applying".to_string(),
                        probability: Some(RiskLevel::Medium),
                        impact: Some(RiskLevel::High),
                    },
                    RiskItem {
                        risk: "Dependency resolution causes compatibility issues".to_string(),
                        mitigation: "Maintain rollback plan and test against all supported platforms".to_string(),
                        probability: Some(RiskLevel::Low),
                        impact: Some(RiskLevel::VeryHigh),
                    },
                ],
                mitigation_strategies: vec![],
                monitoring_requirements: vec![
                    MonitoringRequirement {
                        metric: "build_success_rate".to_string(),
                        threshold: "< 95%".to_string(),
                        alert_condition: "Build reliability degradation".to_string(),
                        escalation_procedure: "Investigate and rollback if necessary".to_string(),
                    },
                ],
            }
        );
    }
    
    /// Load success validators
    fn load_success_validators(&mut self) {
        // GitHub Integration validator
        self.success_validators.insert(
            "github-integration".to_string(),
            SuccessValidator {
                domain: "github-integration".to_string(),
                validation_rules: vec![
                    ValidationRule {
                        name: "api_client_functional".to_string(),
                        description: "GitHub API client successfully authenticates and performs basic operations".to_string(),
                        validation_type: ValidationType::Binary,
                        criteria: "API authentication success rate > 99%".to_string(),
                        weight: 1.0,
                    },
                    ValidationRule {
                        name: "rate_limiting_effective".to_string(),
                        description: "Rate limiting prevents API exhaustion while maintaining throughput".to_string(),
                        validation_type: ValidationType::Threshold,
                        criteria: "No rate limit violations AND throughput > baseline".to_string(),
                        weight: 0.8,
                    },
                ],
                phase_requirements: HashMap::from([
                    ("phase_1".to_string(), vec![
                        "GitHub API client successfully authenticates and performs basic operations".to_string(),
                        "Rate limiting prevents API exhaustion while maintaining operation throughput".to_string(),
                    ]),
                    ("phase_2".to_string(), vec![
                        "Webhook handling processes GitHub events in real-time".to_string(),
                        "Error handling and retry logic handle all common failure scenarios".to_string(),
                    ]),
                ]),
            }
        );
        
        // Infrastructure validator
        self.success_validators.insert(
            "infrastructure".to_string(),
            SuccessValidator {
                domain: "infrastructure".to_string(),
                validation_rules: vec![
                    ValidationRule {
                        name: "dependency_conflicts_resolved".to_string(),
                        description: "All dependency conflicts completely resolved".to_string(),
                        validation_type: ValidationType::Binary,
                        criteria: "Zero dependency conflicts in workspace".to_string(),
                        weight: 1.0,
                    },
                    ValidationRule {
                        name: "build_system_stable".to_string(),
                        description: "All workspace crates build without warnings or errors".to_string(),
                        validation_type: ValidationType::Binary,
                        criteria: "100% build success rate".to_string(),
                        weight: 1.0,
                    },
                ],
                phase_requirements: HashMap::from([
                    ("phase_1".to_string(), vec![
                        "Dependency conflicts completely resolved".to_string(),
                        "Build system stability verified".to_string(),
                    ]),
                ]),
            }
        );
    }
    
    /// Apply priority-based modifications to directives
    fn apply_priority_modifiers(
        &self,
        directives: &mut BehavioralDirectives,
        priority: &str,
    ) -> Result<()> {
        match priority {
            "critical" => {
                directives.operational_focus.push("Prioritize stability and reliability over performance".to_string());
                directives.error_handling.push("Implement comprehensive error recovery and rollback procedures".to_string());
            }
            "high" => {
                directives.operational_focus.push("Balance feature delivery with system stability".to_string());
            }
            "medium" => {
                directives.operational_focus.push("Optimize for development velocity while maintaining quality".to_string());
            }
            "low" => {
                directives.operational_focus.push("Focus on long-term improvements and technical debt reduction".to_string());
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Find validation rule for an objective
    fn find_validation_rule(
        &self,
        validator: &SuccessValidator,
        objective: &str,
    ) -> Result<&ValidationRule> {
        validator.validation_rules.iter()
            .find(|rule| objective.contains(&rule.name) || rule.description.contains(objective))
            .ok_or_else(|| anyhow::anyhow!("No validation rule found for objective: {}", objective))
    }
    
    /// Validate a single rule against outcomes
    fn validate_rule(
        &self,
        rule: &ValidationRule,
        outcomes: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        match rule.validation_type {
            ValidationType::Binary => {
                // Check if outcome exists and is true
                if let Some(value) = outcomes.get(&rule.name) {
                    Ok(value.as_bool().unwrap_or(false))
                } else {
                    Ok(false)
                }
            }
            ValidationType::Threshold => {
                // Parse threshold criteria and compare
                if let Some(value) = outcomes.get(&rule.name) {
                    if let Some(number) = value.as_f64() {
                        // Simple threshold check (could be enhanced with more complex parsing)
                        Ok(number > 0.0)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            ValidationType::Quantitative | ValidationType::Qualitative => {
                // Check if outcome meets quantitative/qualitative criteria
                outcomes.get(&rule.name).map(|_| true).unwrap_or(false).into()
            }
            ValidationType::Trend => {
                // Check trend analysis (simplified)
                outcomes.get(&rule.name).map(|_| true).unwrap_or(false).into()
            }
        }
    }
}

/// Result of success criteria validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub details: Vec<String>,
}

impl ValidationResult {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
    
    pub fn is_successful(&self) -> bool {
        self.failed == 0 && self.passed > 0
    }
}

impl Default for BehaviorManager {
    fn default() -> Self {
        Self::new()
    }
}