//! Delegation management implementation

use crate::{
    DelegatedClaims, DelegationMetadata, DelegationEntry, DelegationRestrictions,
    DelegationManager, DelegationError, DelegationAuditEvent, DelegationEventType,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// Simple in-memory delegation manager
/// 
/// This implementation stores delegation metadata in memory. For production
/// use, this should be backed by a persistent store like a database.
pub struct SimpleDelegationManager {
    /// Storage for delegation metadata
    delegations: Arc<RwLock<HashMap<Uuid, DelegationMetadata>>>,
    /// Audit trail for delegation events
    audit_trail: Arc<RwLock<Vec<DelegationAuditEvent>>>,
    /// Subject to delegations mapping for efficient lookup
    subject_delegations: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl SimpleDelegationManager {
    /// Create a new delegation manager
    pub fn new() -> Self {
        Self {
            delegations: Arc::new(RwLock::new(HashMap::new())),
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            subject_delegations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record an audit event
    async fn record_audit_event(
        &self,
        event_type: DelegationEventType,
        subject: String,
        delegation_id: Uuid,
        metadata: HashMap<String, String>,
    ) {
        let event = DelegationAuditEvent {
            event_id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            subject,
            delegation_id,
            metadata,
        };

        let event_id = event.event_id;
        let event_type = event.event_type.clone();
        
        let mut audit_trail = self.audit_trail.write().await;
        audit_trail.push(event);

        debug!(
            event_id = %event_id,
            event_type = ?event_type,
            delegation_id = %delegation_id,
            "Recorded delegation audit event"
        );
    }

    /// Update subject to delegations mapping
    async fn update_subject_mapping(&self, subject: &str, delegation_id: Uuid) {
        let mut mappings = self.subject_delegations.write().await;
        mappings.entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push(delegation_id);
    }

    /// Validate delegation restrictions
    fn validate_restrictions(
        delegator: &DelegatedClaims,
        permissions: &[String],
        restrictions: &DelegationRestrictions,
    ) -> Result<(), DelegationError> {
        // Check if delegator has the permissions they're trying to delegate
        let delegator_perms = delegator.effective_permissions();
        for permission in permissions {
            if !delegator_perms.contains(permission) {
                return Err(DelegationError::InsufficientPermissions(
                    format!("Delegator does not have permission: {}", permission)
                ));
            }
        }

        // Check if further delegation is allowed
        if let Some(delegation) = &delegator.delegation {
            if !delegation.restrictions.allow_further_delegation {
                return Err(DelegationError::InvalidScope(
                    "Further delegation not allowed by original delegation".to_string()
                ));
            }

            // Check depth limits
            let current_depth = delegation.chain.len();
            if current_depth >= restrictions.max_delegation_depth {
                return Err(DelegationError::DelegationDepthExceeded {
                    current: current_depth,
                    max: restrictions.max_delegation_depth,
                });
            }
        }

        Ok(())
    }

    /// Create delegation metadata from delegator claims
    fn create_delegation_metadata(
        delegator: &DelegatedClaims,
        delegatee: &str,
        permissions: Vec<String>,
        restrictions: DelegationRestrictions,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<DelegationMetadata, DelegationError> {
        let metadata = if let Some(existing_delegation) = &delegator.delegation {
            // Extend existing delegation chain
            let mut extended_metadata = existing_delegation.clone();
            
            extended_metadata.extend_chain(
                delegator.base.sub.clone(),
                delegatee.to_string(),
                permissions,
            )?;
            
            // Apply more restrictive expiry time
            if let Some(new_expires) = expires_at {
                if let Some(existing_expires) = extended_metadata.expires_at {
                    extended_metadata.expires_at = Some(new_expires.min(existing_expires));
                } else {
                    extended_metadata.expires_at = Some(new_expires);
                }
            }
            
            extended_metadata
        } else {
            // Create new delegation
            let mut new_metadata = DelegationMetadata::new(
                permissions,
                delegator.base.sub.clone(),
                delegatee.to_string(),
                restrictions,
            );
            
            new_metadata.expires_at = expires_at;
            new_metadata
        };

        Ok(metadata)
    }

    /// Get delegation statistics for monitoring
    pub async fn get_stats(&self) -> DelegationStats {
        let delegations = self.delegations.read().await;
        let audit_trail = self.audit_trail.read().await;
        let subject_mappings = self.subject_delegations.read().await;

        let total_delegations = delegations.len();
        let active_delegations = delegations.values()
            .filter(|d| d.is_valid())
            .count();
        let revoked_delegations = delegations.values()
            .filter(|d| d.revoked)
            .count();
        let expired_delegations = delegations.values()
            .filter(|d| {
                if let Some(expires_at) = d.expires_at {
                    Utc::now() > expires_at && !d.revoked
                } else {
                    false
                }
            })
            .count();

        DelegationStats {
            total_delegations,
            active_delegations,
            revoked_delegations,
            expired_delegations,
            total_audit_events: audit_trail.len(),
            subjects_with_delegations: subject_mappings.len(),
        }
    }

    /// Get audit trail
    pub async fn get_audit_trail(&self) -> Vec<DelegationAuditEvent> {
        let audit_trail = self.audit_trail.read().await;
        audit_trail.clone()
    }

    /// Cleanup expired delegations
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut delegations = self.delegations.write().await;
        let now = Utc::now();
        let initial_count = delegations.len();

        delegations.retain(|_, delegation| {
            if let Some(expires_at) = delegation.expires_at {
                if now > expires_at && !delegation.revoked {
                    // Mark as expired rather than removing for audit purposes
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        let removed_count = initial_count - delegations.len();
        
        if removed_count > 0 {
            info!(
                removed_count = removed_count,
                "Cleaned up expired delegations"
            );
        }

        Ok(removed_count)
    }
}

impl Default for SimpleDelegationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DelegationManager for SimpleDelegationManager {
    async fn create_delegation(
        &self,
        delegator: &DelegatedClaims,
        delegatee: &str,
        permissions: Vec<String>,
        restrictions: DelegationRestrictions,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<DelegatedClaims, DelegationError> {
        // Validate the delegation request
        Self::validate_restrictions(delegator, &permissions, &restrictions)?;

        // Create delegation metadata
        let delegation_metadata = Self::create_delegation_metadata(
            delegator,
            delegatee,
            permissions.clone(),
            restrictions,
            expires_at,
        )?;

        let delegation_id = delegation_metadata.delegation_id;

        // Store the delegation
        {
            let mut delegations = self.delegations.write().await;
            delegations.insert(delegation_id, delegation_metadata.clone());
        }

        // Update subject mapping
        self.update_subject_mapping(delegatee, delegation_id).await;

        // Record audit event
        let mut audit_metadata = HashMap::new();
        audit_metadata.insert("delegatee".to_string(), delegatee.to_string());
        audit_metadata.insert("permissions".to_string(), permissions.join(","));
        if let Some(expires) = expires_at {
            audit_metadata.insert("expires_at".to_string(), expires.to_rfc3339());
        }

        self.record_audit_event(
            DelegationEventType::DelegationCreated,
            delegator.base.sub.clone(),
            delegation_id,
            audit_metadata,
        ).await;

        // Create new delegated claims for the delegatee
        let permissions_clone = permissions.clone();
        let delegatee_claims = toka_capability_core::Claims {
            sub: delegatee.to_string(),
            vault: delegator.base.vault.clone(),
            permissions: permissions,
            iat: delegator.base.iat,
            exp: delegator.base.exp,
            jti: Uuid::new_v4().to_string(),
        };

        info!(
            delegator = %delegator.base.sub,
            delegatee = %delegatee,
            delegation_id = %delegation_id,
            permissions = ?permissions_clone,
            "Created delegation"
        );

        Ok(DelegatedClaims::with_delegation(delegatee_claims, delegation_metadata))
    }

    async fn revoke_delegation(
        &self,
        delegation_id: &Uuid,
        reason: String,
    ) -> Result<(), DelegationError> {
        let mut delegations = self.delegations.write().await;
        
        if let Some(delegation) = delegations.get_mut(delegation_id) {
            let reason_clone = reason.clone();
            delegation.revoke(reason.clone());
            
            // Record audit event
            let mut audit_metadata = HashMap::new();
            audit_metadata.insert("reason".to_string(), reason);
            
            self.record_audit_event(
                DelegationEventType::DelegationRevoked,
                "system".to_string(), // Could be made configurable
                *delegation_id,
                audit_metadata,
            ).await;

            warn!(
                delegation_id = %delegation_id,
                reason = %reason_clone,
                "Revoked delegation"
            );

            Ok(())
        } else {
            Err(DelegationError::InvalidChain(
                format!("Delegation not found: {}", delegation_id)
            ))
        }
    }

    async fn validate_delegation(
        &self,
        claims: &DelegatedClaims,
    ) -> Result<bool, DelegationError> {
        if let Some(delegation) = &claims.delegation {
            let delegations = self.delegations.read().await;
            
            if let Some(stored_delegation) = delegations.get(&delegation.delegation_id) {
                // Check if delegation is still valid
                if !stored_delegation.is_valid() {
                    if stored_delegation.revoked {
                        return Err(DelegationError::DelegationRevoked {
                            reason: stored_delegation.revocation_reason
                                .clone()
                                .unwrap_or_else(|| "No reason provided".to_string()),
                        });
                    }
                    
                    if let Some(expires_at) = stored_delegation.expires_at {
                        if Utc::now() > expires_at {
                            return Err(DelegationError::DelegationExpired { expires_at });
                        }
                    }
                }

                // Record usage
                self.record_audit_event(
                    DelegationEventType::DelegationUsed,
                    claims.base.sub.clone(),
                    delegation.delegation_id,
                    HashMap::new(),
                ).await;

                Ok(true)
            } else {
                Err(DelegationError::InvalidChain(
                    "Delegation not found in storage".to_string()
                ))
            }
        } else {
            // Non-delegated claims are always valid (handled by base validation)
            Ok(true)
        }
    }

    async fn get_delegation_chain(
        &self,
        delegation_id: &Uuid,
    ) -> Result<Vec<DelegationEntry>, DelegationError> {
        let delegations = self.delegations.read().await;
        
        if let Some(delegation) = delegations.get(delegation_id) {
            Ok(delegation.chain.clone())
        } else {
            Err(DelegationError::InvalidChain(
                format!("Delegation not found: {}", delegation_id)
            ))
        }
    }

    async fn list_delegations(
        &self,
        subject: &str,
    ) -> Result<Vec<DelegationMetadata>, DelegationError> {
        let subject_mappings = self.subject_delegations.read().await;
        let delegations = self.delegations.read().await;
        
        if let Some(delegation_ids) = subject_mappings.get(subject) {
            let mut result = Vec::new();
            
            for delegation_id in delegation_ids {
                if let Some(delegation) = delegations.get(delegation_id) {
                    result.push(delegation.clone());
                }
            }
            
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Statistics about delegation usage
#[derive(Debug, Clone)]
pub struct DelegationStats {
    /// Total number of delegations ever created
    pub total_delegations: usize,
    /// Currently active delegations
    pub active_delegations: usize,
    /// Delegations that have been revoked
    pub revoked_delegations: usize,
    /// Delegations that have expired
    pub expired_delegations: usize,
    /// Total audit events recorded
    pub total_audit_events: usize,
    /// Number of subjects with delegations
    pub subjects_with_delegations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DelegatedClaims, DelegationRestrictions};
    use toka_capability_core::Claims;

    #[tokio::test]
    async fn test_simple_delegation() {
        let manager = SimpleDelegationManager::new();
        
        let base_claims = Claims {
            sub: "alice".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let delegator = DelegatedClaims::new(base_claims);
        
        let result = manager.create_delegation(
            &delegator,
            "bob",
            vec!["read".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await;

        assert!(result.is_ok());
        let delegated_claims = result.unwrap();
        assert!(delegated_claims.is_delegated());
        assert_eq!(delegated_claims.base.sub, "bob");
        assert!(delegated_claims.has_permission("read"));
        assert!(!delegated_claims.has_permission("write"));
    }

    #[tokio::test]
    async fn test_delegation_validation() {
        let manager = SimpleDelegationManager::new();
        
        let base_claims = Claims {
            sub: "alice".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let delegator = DelegatedClaims::new(base_claims);
        
        let delegated_claims = manager.create_delegation(
            &delegator,
            "bob",
            vec!["read".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await.unwrap();

        // Validation should succeed
        let is_valid = manager.validate_delegation(&delegated_claims).await.unwrap();
        assert!(is_valid);

        // Revoke the delegation
        let delegation_id = delegated_claims.delegation.as_ref().unwrap().delegation_id;
        manager.revoke_delegation(&delegation_id, "Test revocation".to_string()).await.unwrap();

        // Validation should now fail
        let result = manager.validate_delegation(&delegated_claims).await;
        assert!(matches!(result, Err(DelegationError::DelegationRevoked { .. })));
    }

    #[tokio::test]
    async fn test_insufficient_permissions() {
        let manager = SimpleDelegationManager::new();
        
        let base_claims = Claims {
            sub: "alice".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let delegator = DelegatedClaims::new(base_claims);
        
        // Try to delegate a permission the delegator doesn't have
        let result = manager.create_delegation(
            &delegator,
            "bob",
            vec!["write".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await;

        assert!(matches!(result, Err(DelegationError::InsufficientPermissions(_))));
    }

    #[tokio::test]
    async fn test_delegation_chain() {
        let manager = SimpleDelegationManager::new();
        
        // Alice delegates to Bob
        let alice_claims = Claims {
            sub: "alice".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let alice_delegated = DelegatedClaims::new(alice_claims);
        
        let bob_claims = manager.create_delegation(
            &alice_delegated,
            "bob",
            vec!["read".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await.unwrap();

        // Bob delegates to Charlie
        let charlie_claims = manager.create_delegation(
            &bob_claims,
            "charlie",
            vec!["read".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await.unwrap();

        assert_eq!(charlie_claims.delegation_depth(), 2);
        assert_eq!(charlie_claims.original_delegator(), Some(&"alice".to_string()));
        assert_eq!(charlie_claims.immediate_delegator(), Some(&"bob".to_string()));
    }

    #[tokio::test]
    async fn test_stats_collection() {
        let manager = SimpleDelegationManager::new();
        
        let base_claims = Claims {
            sub: "alice".to_string(),
            vault: "vault1".to_string(),
            permissions: vec!["read".to_string()],
            iat: 1640995200,
            exp: 1640998800,
            jti: Uuid::new_v4().to_string(),
        };

        let delegator = DelegatedClaims::new(base_claims);
        
        // Create delegation
        let delegated_claims = manager.create_delegation(
            &delegator,
            "bob",
            vec!["read".to_string()],
            DelegationRestrictions::default(),
            None,
        ).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_delegations, 1);
        assert_eq!(stats.active_delegations, 1);
        assert_eq!(stats.revoked_delegations, 0);

        // Revoke delegation
        let delegation_id = delegated_claims.delegation.as_ref().unwrap().delegation_id;
        manager.revoke_delegation(&delegation_id, "Test".to_string()).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_delegations, 1);
        assert_eq!(stats.active_delegations, 0);
        assert_eq!(stats.revoked_delegations, 1);
    }
} 