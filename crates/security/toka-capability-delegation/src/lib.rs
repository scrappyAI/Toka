#![forbid(unsafe_code)]

//! **toka-capability-delegation** – Hierarchical capability delegation for Toka
//!
//! This crate extends Toka's capability system with sophisticated delegation
//! primitives that enable secure, hierarchical permission management. It provides:
//!
//! * **Hierarchical Permissions**: Parent/child permission relationships
//! * **Delegation Chains**: Traceable permission delegation paths
//! * **Temporal Delegation**: Time-bound and revocable delegations
//! * **Scope Limitations**: Delegated permissions can be restricted subsets
//! * **Audit Trails**: Comprehensive delegation tracking for security
//!
//! The system integrates seamlessly with existing JWT capability tokens while
//! adding powerful delegation semantics that enable complex organizational
//! permission structures.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use toka_capability_core::{Claims, CapabilityToken, TokenValidator};
use uuid::Uuid;

pub mod delegation;
pub mod hierarchy;
pub mod tokens;
pub mod validation;

/// Delegation-specific error types
#[derive(Error, Debug)]
pub enum DelegationError {
    #[error("Insufficient permissions for delegation: {0}")]
    InsufficientPermissions(String),
    
    #[error("Invalid delegation chain: {0}")]
    InvalidChain(String),
    
    #[error("Delegation expired: {expires_at}")]
    DelegationExpired { expires_at: DateTime<Utc> },
    
    #[error("Delegation revoked: {reason}")]
    DelegationRevoked { reason: String },
    
    #[error("Circular delegation detected")]
    CircularDelegation,
    
    #[error("Permission not found: {permission}")]
    PermissionNotFound { permission: String },
    
    #[error("Delegation depth exceeded: {current}/{max}")]
    DelegationDepthExceeded { current: usize, max: usize },
    
    #[error("Invalid delegation scope: {0}")]
    InvalidScope(String),
}

/// Enhanced claims structure that supports delegation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegatedClaims {
    /// Base claims from the original token
    pub base: Claims,
    /// Delegation-specific metadata
    pub delegation: Option<DelegationMetadata>,
}

impl DelegatedClaims {
    /// Create new delegated claims from base claims
    pub fn new(base: Claims) -> Self {
        Self {
            base,
            delegation: None,
        }
    }

    /// Create delegated claims with delegation metadata
    pub fn with_delegation(base: Claims, delegation: DelegationMetadata) -> Self {
        Self {
            base,
            delegation: Some(delegation),
        }
    }

    /// Check if this token represents a delegated capability
    pub fn is_delegated(&self) -> bool {
        self.delegation.is_some()
    }

    /// Get the delegation chain depth
    pub fn delegation_depth(&self) -> usize {
        self.delegation.as_ref()
            .map(|d| d.chain.len())
            .unwrap_or(0)
    }

    /// Get effective permissions (intersection of base and delegated)
    pub fn effective_permissions(&self) -> Vec<String> {
        match &self.delegation {
            Some(delegation) => {
                // Intersection of base permissions and delegated permissions
                let base_perms: HashSet<_> = self.base.permissions.iter().collect();
                let delegated_perms: HashSet<_> = delegation.delegated_permissions.iter().collect();
                
                base_perms.intersection(&delegated_perms)
                    .map(|s| s.to_string())
                    .collect()
            }
            None => self.base.permissions.clone(),
        }
    }

    /// Check if a specific permission is available
    pub fn has_permission(&self, permission: &str) -> bool {
        self.effective_permissions().contains(&permission.to_string())
    }

    /// Get the original delegator in the chain
    pub fn original_delegator(&self) -> Option<&String> {
        self.delegation.as_ref()?
            .chain.first()
            .map(|entry| &entry.delegator)
    }

    /// Get the immediate delegator
    pub fn immediate_delegator(&self) -> Option<&String> {
        self.delegation.as_ref()?
            .chain.last()
            .map(|entry| &entry.delegator)
    }
}

/// Delegation metadata embedded in delegated tokens
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationMetadata {
    /// Unique identifier for this delegation
    pub delegation_id: Uuid,
    /// Permissions that have been delegated
    pub delegated_permissions: Vec<String>,
    /// Chain of delegation entries
    pub chain: Vec<DelegationEntry>,
    /// Restrictions applied to this delegation
    pub restrictions: DelegationRestrictions,
    /// When this delegation was created
    pub created_at: DateTime<Utc>,
    /// When this delegation expires (independent of token expiry)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this delegation has been revoked
    pub revoked: bool,
    /// Reason for revocation, if any
    pub revocation_reason: Option<String>,
}

impl DelegationMetadata {
    /// Create new delegation metadata
    pub fn new(
        delegated_permissions: Vec<String>,
        delegator: String,
        delegatee: String,
        restrictions: DelegationRestrictions,
    ) -> Self {
        Self {
            delegation_id: Uuid::new_v4(),
            delegated_permissions,
            chain: vec![DelegationEntry {
                delegator,
                delegatee,
                delegated_at: Utc::now(),
                delegation_id: Uuid::new_v4(),
            }],
            restrictions,
            created_at: Utc::now(),
            expires_at: None,
            revoked: false,
            revocation_reason: None,
        }
    }

    /// Add a new delegation to the chain
    pub fn extend_chain(
        &mut self,
        delegator: String,
        delegatee: String,
        new_permissions: Vec<String>,
    ) -> Result<(), DelegationError> {
        // Check delegation depth
        if self.chain.len() >= self.restrictions.max_delegation_depth {
            return Err(DelegationError::DelegationDepthExceeded {
                current: self.chain.len(),
                max: self.restrictions.max_delegation_depth,
            });
        }

        // Check for circular delegation (prevent delegatee from appearing as delegator in chain)
        if self.chain.iter().any(|entry| entry.delegator == delegatee) {
            return Err(DelegationError::CircularDelegation);
        }

        // Ensure new permissions are subset of current permissions
        let current_perms: HashSet<_> = self.delegated_permissions.iter().collect();
        let new_perms: HashSet<_> = new_permissions.iter().collect();
        
        if !new_perms.is_subset(&current_perms) {
            return Err(DelegationError::InsufficientPermissions(
                "Cannot delegate permissions not currently held".to_string()
            ));
        }

        // Add to chain
        self.chain.push(DelegationEntry {
            delegator,
            delegatee,
            delegated_at: Utc::now(),
            delegation_id: Uuid::new_v4(),
        });

        // Update delegated permissions to the intersection
        self.delegated_permissions = new_permissions;

        Ok(())
    }

    /// Check if delegation is currently valid
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        true
    }

    /// Revoke this delegation
    pub fn revoke(&mut self, reason: String) {
        self.revoked = true;
        self.revocation_reason = Some(reason);
    }
}

/// Single entry in a delegation chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationEntry {
    /// Who performed this delegation
    pub delegator: String,
    /// Who received the delegation
    pub delegatee: String,
    /// When this delegation occurred
    pub delegated_at: DateTime<Utc>,
    /// Unique identifier for this specific delegation
    pub delegation_id: Uuid,
}

/// Restrictions that can be applied to delegations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationRestrictions {
    /// Maximum depth of delegation chain
    pub max_delegation_depth: usize,
    /// Whether further delegation is allowed
    pub allow_further_delegation: bool,
    /// Time-based restrictions
    pub time_restrictions: Option<TimeRestrictions>,
    /// Resource-based restrictions
    pub resource_restrictions: Vec<String>,
    /// Custom metadata for additional restrictions
    pub custom_restrictions: HashMap<String, String>,
}

impl Default for DelegationRestrictions {
    fn default() -> Self {
        Self {
            max_delegation_depth: 5,
            allow_further_delegation: true,
            time_restrictions: None,
            resource_restrictions: Vec::new(),
            custom_restrictions: HashMap::new(),
        }
    }
}

/// Time-based delegation restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeRestrictions {
    /// Allowed time windows (e.g., business hours)
    pub allowed_time_windows: Vec<TimeWindow>,
    /// Timezone for time window evaluation
    pub timezone: String,
    /// Days of week when delegation is valid (1=Monday, 7=Sunday)
    pub allowed_days: Vec<u32>,
}

/// Time window definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeWindow {
    /// Start time (24-hour format, e.g., "09:00")
    pub start_time: String,
    /// End time (24-hour format, e.g., "17:00")
    pub end_time: String,
}

/// Delegation management interface
#[async_trait]
pub trait DelegationManager: Send + Sync {
    /// Create a new delegation
    async fn create_delegation(
        &self,
        delegator: &DelegatedClaims,
        delegatee: &str,
        permissions: Vec<String>,
        restrictions: DelegationRestrictions,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<DelegatedClaims, DelegationError>;

    /// Revoke a delegation
    async fn revoke_delegation(
        &self,
        delegation_id: &Uuid,
        reason: String,
    ) -> Result<(), DelegationError>;

    /// Check if delegation is valid
    async fn validate_delegation(
        &self,
        claims: &DelegatedClaims,
    ) -> Result<bool, DelegationError>;

    /// Get delegation chain information
    async fn get_delegation_chain(
        &self,
        delegation_id: &Uuid,
    ) -> Result<Vec<DelegationEntry>, DelegationError>;

    /// List all delegations for a subject
    async fn list_delegations(
        &self,
        subject: &str,
    ) -> Result<Vec<DelegationMetadata>, DelegationError>;
}

/// Permission hierarchy management
#[async_trait]
pub trait PermissionHierarchy: Send + Sync {
    /// Check if one permission implies another
    async fn implies(&self, parent: &str, child: &str) -> Result<bool>;

    /// Get all permissions implied by a parent permission
    async fn get_implied_permissions(&self, permission: &str) -> Result<Vec<String>>;

    /// Add a parent-child relationship
    async fn add_implication(&self, parent: &str, child: &str) -> Result<()>;

    /// Remove a parent-child relationship
    async fn remove_implication(&self, parent: &str, child: &str) -> Result<()>;

    /// Get the full hierarchy as a graph
    async fn get_hierarchy(&self) -> Result<HashMap<String, Vec<String>>>;
}

/// Delegation audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationAuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Type of delegation event
    pub event_type: DelegationEventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Subject who initiated the event
    pub subject: String,
    /// Delegation ID affected
    pub delegation_id: Uuid,
    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

/// Types of delegation events for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationEventType {
    /// A new delegation was created
    DelegationCreated,
    /// A delegation was extended to another party
    DelegationExtended,
    /// A delegation was revoked
    DelegationRevoked,
    /// A delegation was used to perform an action
    DelegationUsed,
    /// A delegation expired naturally
    DelegationExpired,
    /// A delegation validation failed
    ValidationFailed,
}

/// Convenience re-exports for common usage
pub mod prelude {
    pub use super::{
        DelegatedClaims, DelegationMetadata, DelegationEntry, DelegationRestrictions,
        DelegationManager, PermissionHierarchy, DelegationError,
        delegation::SimpleDelegationManager,
        hierarchy::SimplePermissionHierarchy,
        tokens::JwtDelegatedTokenGenerator,
        validation::DelegationValidator,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_capability_core::Claims;

    #[test]
    fn test_delegated_claims_creation() {
        let base_claims = Claims {
            sub: "user123".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let delegated_claims = DelegatedClaims::new(base_claims.clone());
        assert!(!delegated_claims.is_delegated());
        assert_eq!(delegated_claims.delegation_depth(), 0);
        assert_eq!(delegated_claims.effective_permissions(), base_claims.permissions);
    }

    #[test]
    fn test_delegation_metadata() {
        let mut metadata = DelegationMetadata::new(
            vec!["read".to_string()],
            "delegator1".to_string(),
            "delegatee1".to_string(),
            DelegationRestrictions::default(),
        );

        assert!(metadata.is_valid());
        assert_eq!(metadata.chain.len(), 1);

        // Test chain extension
        let result = metadata.extend_chain(
            "delegatee1".to_string(),
            "delegatee2".to_string(),
            vec!["read".to_string()],
        );
        assert!(result.is_ok());
        assert_eq!(metadata.chain.len(), 2);

        // Test revocation
        metadata.revoke("Test revocation".to_string());
        assert!(!metadata.is_valid());
        assert_eq!(metadata.revocation_reason, Some("Test revocation".to_string()));
    }

    #[test]
    fn test_circular_delegation_prevention() {
        let mut metadata = DelegationMetadata::new(
            vec!["read".to_string()],
            "user1".to_string(),
            "user2".to_string(),
            DelegationRestrictions::default(),
        );

        // Try to create circular delegation
        let result = metadata.extend_chain(
            "user2".to_string(),
            "user1".to_string(), // This would create a circle
            vec!["read".to_string()],
        );

        assert!(matches!(result, Err(DelegationError::CircularDelegation)));
    }

    #[test]
    fn test_permission_subset_enforcement() {
        let mut metadata = DelegationMetadata::new(
            vec!["read".to_string()],
            "user1".to_string(),
            "user2".to_string(),
            DelegationRestrictions::default(),
        );

        // Try to delegate permissions not currently held
        let result = metadata.extend_chain(
            "user2".to_string(),
            "user3".to_string(),
            vec!["read".to_string(), "write".to_string()], // "write" not in original
        );

        assert!(matches!(result, Err(DelegationError::InsufficientPermissions(_))));
    }

    #[test]
    fn test_delegation_depth_limit() {
        let mut metadata = DelegationMetadata::new(
            vec!["read".to_string()],
            "user1".to_string(),
            "user2".to_string(),
            DelegationRestrictions {
                max_delegation_depth: 2,
                ..Default::default()
            },
        );

        // First extension should succeed
        assert!(metadata.extend_chain(
            "user2".to_string(),
            "user3".to_string(),
            vec!["read".to_string()],
        ).is_ok());

        // Second extension should fail (would exceed depth of 2)
        let result = metadata.extend_chain(
            "user3".to_string(),
            "user4".to_string(),
            vec!["read".to_string()],
        );

        assert!(matches!(result, Err(DelegationError::DelegationDepthExceeded { .. })));
    }
} 