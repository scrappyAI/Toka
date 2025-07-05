//! Validation logic for capability delegation
//!
//! This module provides comprehensive validation for delegation chains,
//! permission hierarchies, and time-based restrictions.

use crate::{
    DelegatedClaims, DelegationError, DelegationMetadata, DelegationRestrictions,
    TimeRestrictions, TimeWindow, PermissionHierarchy,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Timelike, NaiveTime, Datelike};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, warn};

/// Comprehensive delegation validator
/// 
/// This validator checks all aspects of delegation validity including:
/// - Token expiry
/// - Delegation chain integrity
/// - Permission subset validation
/// - Time-based restrictions
/// - Depth limits
/// - Circular delegation prevention
pub struct DelegationValidator {
    /// Permission hierarchy for implied permissions
    hierarchy: Arc<dyn PermissionHierarchy>,
    /// Validation configuration
    config: ValidationConfig,
}

impl DelegationValidator {
    /// Create a new delegation validator
    pub fn new(hierarchy: Arc<dyn PermissionHierarchy>, config: ValidationConfig) -> Self {
        Self {
            hierarchy,
            config,
        }
    }

    /// Create validator with default configuration
    pub fn with_hierarchy(hierarchy: Arc<dyn PermissionHierarchy>) -> Self {
        Self::new(hierarchy, ValidationConfig::default())
    }

    /// Validate a delegated token completely
    pub async fn validate_complete(
        &self,
        claims: &DelegatedClaims,
    ) -> Result<ValidationResult, DelegationError> {
        let mut result = ValidationResult::new();

        // Basic token validation
        self.validate_token_expiry(claims, &mut result).await?;

        // Delegation-specific validation
        if let Some(delegation) = &claims.delegation {
            self.validate_delegation_metadata(delegation, &mut result).await?;
            self.validate_delegation_chain(delegation, &mut result).await?;
            self.validate_permission_subset(claims, delegation, &mut result).await?;
            self.validate_time_restrictions(delegation, &mut result).await?;
            self.validate_depth_limits(delegation, &mut result).await?;
        }

        // Check overall validity
        if result.has_errors() {
            return Err(DelegationError::InvalidChain(
                result.errors.join("; ")
            ));
        }

        debug!(
            token_id = %claims.base.jti,
            is_delegated = %claims.is_delegated(),
            validation_score = %result.validation_score(),
            "Completed delegation validation"
        );

        Ok(result)
    }

    /// Validate token expiry
    async fn validate_token_expiry(
        &self,
        claims: &DelegatedClaims,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        let now = Utc::now().timestamp() as u64;
        
        if claims.base.exp < now {
            let error = format!("Token expired at {}", claims.base.exp);
            result.add_error(error.clone());
            return Err(DelegationError::DelegationExpired {
                expires_at: DateTime::from_timestamp(claims.base.exp as i64, 0)
                    .unwrap_or_else(|| Utc::now()),
            });
        }

        result.add_check("token_expiry", true);
        Ok(())
    }

    /// Validate delegation metadata
    async fn validate_delegation_metadata(
        &self,
        delegation: &DelegationMetadata,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        // Check if delegation is revoked
        if delegation.revoked {
            let error = format!("Delegation revoked: {}", 
                delegation.revocation_reason.as_ref().unwrap_or(&"No reason provided".to_string()));
            result.add_error(error.clone());
            return Err(DelegationError::DelegationRevoked {
                reason: delegation.revocation_reason.clone()
                    .unwrap_or_else(|| "Delegation revoked".to_string()),
            });
        }

        // Check delegation expiry
        if let Some(expires_at) = delegation.expires_at {
            if Utc::now() > expires_at {
                let error = format!("Delegation expired at {}", expires_at);
                result.add_error(error.clone());
                return Err(DelegationError::DelegationExpired { expires_at });
            }
        }

        // Check delegation chain is not empty
        if delegation.chain.is_empty() {
            let error = "Delegation chain is empty".to_string();
            result.add_error(error.clone());
            return Err(DelegationError::InvalidChain(error));
        }

        // Check permissions are not empty
        if delegation.delegated_permissions.is_empty() {
            let error = "Delegated permissions are empty".to_string();
            result.add_error(error.clone());
            return Err(DelegationError::InvalidScope(error));
        }

        result.add_check("delegation_metadata", true);
        Ok(())
    }

    /// Validate delegation chain integrity
    async fn validate_delegation_chain(
        &self,
        delegation: &DelegationMetadata,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        // Check for circular delegation
        let mut seen_subjects = HashSet::new();
        for entry in &delegation.chain {
            if seen_subjects.contains(&entry.delegatee) {
                let error = format!("Circular delegation detected involving {}", entry.delegatee);
                result.add_error(error.clone());
                return Err(DelegationError::CircularDelegation);
            }
            seen_subjects.insert(entry.delegator.clone());
        }

        // Validate chain continuity
        for i in 1..delegation.chain.len() {
            let prev_entry = &delegation.chain[i - 1];
            let curr_entry = &delegation.chain[i];
            
            if prev_entry.delegatee != curr_entry.delegator {
                let error = format!(
                    "Delegation chain break: {} -> {} != {} -> {}",
                    prev_entry.delegator, prev_entry.delegatee,
                    curr_entry.delegator, curr_entry.delegatee
                );
                result.add_error(error.clone());
                return Err(DelegationError::InvalidChain(error));
            }
        }

        result.add_check("delegation_chain", true);
        Ok(())
    }

    /// Validate permission subset requirement
    async fn validate_permission_subset(
        &self,
        claims: &DelegatedClaims,
        delegation: &DelegationMetadata,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        // Get base permissions (with hierarchy expansion)
        let base_permissions = self.expand_permissions(&claims.base.permissions).await?;
        let delegated_permissions = self.expand_permissions(&delegation.delegated_permissions).await?;

        // Check if delegated permissions are a subset of base permissions
        let base_set: HashSet<_> = base_permissions.iter().collect();
        let delegated_set: HashSet<_> = delegated_permissions.iter().collect();

        if !delegated_set.is_subset(&base_set) {
            let missing: Vec<_> = delegated_set.difference(&base_set).collect();
            let error = format!("Delegated permissions not subset of base permissions: {:?}", missing);
            result.add_error(error.clone());
            return Err(DelegationError::InsufficientPermissions(error));
        }

        result.add_check("permission_subset", true);
        Ok(())
    }

    /// Validate time-based restrictions
    async fn validate_time_restrictions(
        &self,
        delegation: &DelegationMetadata,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        if let Some(time_restrictions) = &delegation.restrictions.time_restrictions {
            let now = Utc::now();
            
            // Check allowed days
            if !time_restrictions.allowed_days.is_empty() {
                let current_day = now.weekday().num_days_from_monday() + 1; // 1 = Monday
                if !time_restrictions.allowed_days.contains(&current_day) {
                    let error = format!("Current day {} not in allowed days: {:?}", 
                        current_day, time_restrictions.allowed_days);
                    result.add_error(error.clone());
                    return Err(DelegationError::InvalidScope(error));
                }
            }

            // Check time windows
            if !time_restrictions.allowed_time_windows.is_empty() {
                if !self.is_within_time_windows(&time_restrictions.allowed_time_windows, now).await? {
                    let error = "Current time not within allowed time windows".to_string();
                    result.add_error(error.clone());
                    return Err(DelegationError::InvalidScope(error));
                }
            }
        }

        result.add_check("time_restrictions", true);
        Ok(())
    }

    /// Validate delegation depth limits
    async fn validate_depth_limits(
        &self,
        delegation: &DelegationMetadata,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError> {
        let current_depth = delegation.chain.len();
        let max_depth = delegation.restrictions.max_delegation_depth;

        if current_depth > max_depth {
            let error = format!("Delegation depth {} exceeds maximum {}", current_depth, max_depth);
            result.add_error(error.clone());
            return Err(DelegationError::DelegationDepthExceeded {
                current: current_depth,
                max: max_depth,
            });
        }

        result.add_check("depth_limits", true);
        Ok(())
    }

    /// Expand permissions using hierarchy
    async fn expand_permissions(&self, permissions: &[String]) -> Result<Vec<String>, DelegationError> {
        let mut expanded = HashSet::new();
        
        for permission in permissions {
            expanded.insert(permission.clone());
            
            // Add implied permissions
            match self.hierarchy.get_implied_permissions(permission).await {
                Ok(implied) => {
                    for implied_perm in implied {
                        expanded.insert(implied_perm);
                    }
                }
                Err(e) => {
                    warn!(
                        permission = %permission,
                        error = %e,
                        "Failed to get implied permissions"
                    );
                }
            }
        }

        Ok(expanded.into_iter().collect())
    }

    /// Check if current time is within allowed time windows
    async fn is_within_time_windows(
        &self,
        windows: &[TimeWindow],
        now: DateTime<Utc>,
    ) -> Result<bool, DelegationError> {
        let current_time = now.time();
        
        for window in windows {
            if self.is_time_in_window(&current_time, window).await? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Check if a time is within a specific time window
    async fn is_time_in_window(
        &self,
        time: &chrono::NaiveTime,
        window: &TimeWindow,
    ) -> Result<bool, DelegationError> {
        let start_time = NaiveTime::parse_from_str(&window.start_time, "%H:%M")
            .map_err(|e| DelegationError::InvalidScope(format!("Invalid start time format: {}", e)))?;
        
        let end_time = NaiveTime::parse_from_str(&window.end_time, "%H:%M")
            .map_err(|e| DelegationError::InvalidScope(format!("Invalid end time format: {}", e)))?;

        // Handle overnight windows (e.g., 22:00 to 06:00)
        if start_time <= end_time {
            Ok(*time >= start_time && *time <= end_time)
        } else {
            Ok(*time >= start_time || *time <= end_time)
        }
    }
}

/// Configuration for delegation validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to enforce strict permission subset validation
    pub strict_permission_validation: bool,
    /// Whether to check time restrictions
    pub enforce_time_restrictions: bool,
    /// Maximum allowed delegation depth
    pub max_delegation_depth: usize,
    /// Whether to allow circular delegation detection
    pub detect_circular_delegation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_permission_validation: true,
            enforce_time_restrictions: true,
            max_delegation_depth: 10,
            detect_circular_delegation: true,
        }
    }
}

/// Result of delegation validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
    /// Validation checks performed
    pub checks: std::collections::HashMap<String, bool>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            checks: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add a validation check result
    pub fn add_check(&mut self, check: &str, passed: bool) {
        self.checks.insert(check.to_string(), passed);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: String) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    /// Get validation score (0.0 to 1.0)
    pub fn validation_score(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }

        let passed_checks = self.checks.values().filter(|&&passed| passed).count();
        passed_checks as f64 / self.checks.len() as f64
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for custom validation rules
#[async_trait]
pub trait ValidationRule: Send + Sync {
    /// Name of the validation rule
    fn name(&self) -> &str;

    /// Execute the validation rule
    async fn validate(
        &self,
        claims: &DelegatedClaims,
        result: &mut ValidationResult,
    ) -> Result<(), DelegationError>;
}

/// Composite validator that applies multiple validation rules
pub struct CompositeValidator {
    /// List of validation rules
    rules: Vec<Box<dyn ValidationRule>>,
}

impl CompositeValidator {
    /// Create a new composite validator
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate using all rules
    pub async fn validate(&self, claims: &DelegatedClaims) -> Result<ValidationResult, DelegationError> {
        let mut result = ValidationResult::new();

        for rule in &self.rules {
            if let Err(e) = rule.validate(claims, &mut result).await {
                result.add_error(format!("Rule '{}' failed: {}", rule.name(), e));
            }
        }

        Ok(result)
    }
}

impl Default for CompositeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::SimplePermissionHierarchy;
    use crate::{DelegatedClaims, DelegationMetadata, DelegationEntry};
    use chrono::{Duration, TimeZone};
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_basic_validation() {
        let hierarchy = Arc::new(SimplePermissionHierarchy::new());
        let validator = DelegationValidator::with_hierarchy(hierarchy);

        let claims = DelegatedClaims::new(toka_capability_core::Claims {
            sub: "test_user".to_string(),
            vault: "test_vault".to_string(),
            permissions: vec!["read".to_string()],
            iat: Utc::now().timestamp() as u64,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        });

        let result = validator.validate_complete(&claims).await.unwrap();
        assert!(result.is_valid());
        assert_eq!(result.validation_score(), 1.0);
    }

    #[tokio::test]
    async fn test_expired_token_validation() {
        let hierarchy = Arc::new(SimplePermissionHierarchy::new());
        let validator = DelegationValidator::with_hierarchy(hierarchy);

        let claims = DelegatedClaims::new(toka_capability_core::Claims {
            sub: "test_user".to_string(),
            vault: "test_vault".to_string(),
            permissions: vec!["read".to_string()],
            iat: (Utc::now() - Duration::hours(2)).timestamp() as u64,
            exp: (Utc::now() - Duration::hours(1)).timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
        });

        let result = validator.validate_complete(&claims).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DelegationError::DelegationExpired { .. }));
    }

    #[tokio::test]
    async fn test_time_window_validation() {
        let now = Utc::now();
        let current_time = now.time();
        
        let window = TimeWindow {
            start_time: format!("{:02}:{:02}", current_time.hour(), current_time.minute()),
            end_time: format!("{:02}:{:02}", 
                (current_time.hour() + 1) % 24, 
                current_time.minute()
            ),
        };

        let hierarchy = Arc::new(SimplePermissionHierarchy::new());
        let validator = DelegationValidator::with_hierarchy(hierarchy);

        let is_within = validator.is_time_in_window(&current_time, &window).await.unwrap();
        assert!(is_within);
    }

    #[tokio::test]
    async fn test_overnight_time_window() {
        let validator = DelegationValidator::with_hierarchy(
            Arc::new(SimplePermissionHierarchy::new())
        );

        // Test overnight window (22:00 to 06:00)
        let window = TimeWindow {
            start_time: "22:00".to_string(),
            end_time: "06:00".to_string(),
        };

        let night_time = NaiveTime::from_hms_opt(23, 30, 0).unwrap();
        let early_morning = NaiveTime::from_hms_opt(5, 30, 0).unwrap();
        let day_time = NaiveTime::from_hms_opt(12, 0, 0).unwrap();

        assert!(validator.is_time_in_window(&night_time, &window).await.unwrap());
        assert!(validator.is_time_in_window(&early_morning, &window).await.unwrap());
        assert!(!validator.is_time_in_window(&day_time, &window).await.unwrap());
    }

    #[tokio::test]
    async fn test_validation_score() {
        let mut result = ValidationResult::new();
        
        result.add_check("test1", true);
        result.add_check("test2", true);
        result.add_check("test3", false);
        
        assert_eq!(result.validation_score(), 2.0 / 3.0);
    }
} 