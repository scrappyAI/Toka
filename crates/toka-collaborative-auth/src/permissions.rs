//! Permission mapping and capability checking

use std::collections::HashMap;

use tracing::{debug, warn};

use crate::{AuthError, AuthResult, AuthSession, GitHubRole, TokaCapability};

/// Permission matrix for mapping GitHub roles to Toka capabilities
#[derive(Debug, Clone)]
pub struct PermissionMatrix {
    /// Role to capability mapping
    role_mapping: HashMap<GitHubRole, Vec<TokaCapability>>,
}

impl PermissionMatrix {
    /// Create a new permission matrix with the given role mapping
    pub fn new(role_mapping: HashMap<GitHubRole, Vec<TokaCapability>>) -> Self {
        Self { role_mapping }
    }
    
    /// Get capabilities for a given role
    pub fn get_capabilities(&self, role: &GitHubRole) -> Vec<TokaCapability> {
        self.role_mapping
            .get(role)
            .cloned()
            .unwrap_or_else(|| {
                warn!("No capabilities defined for role: {:?}", role);
                vec![TokaCapability::Read] // Default to read-only
            })
    }
    
    /// Check if a role has a specific capability
    pub fn has_capability(&self, role: &GitHubRole, capability: &TokaCapability) -> bool {
        self.get_capabilities(role).contains(capability)
    }
    
    /// Check if a session has a specific capability
    pub fn session_has_capability(&self, session: &AuthSession, capability: &TokaCapability) -> bool {
        session.capabilities.contains(capability)
    }
    
    /// Get all roles that have a specific capability
    pub fn roles_with_capability(&self, capability: &TokaCapability) -> Vec<GitHubRole> {
        self.role_mapping
            .iter()
            .filter(|(_, capabilities)| capabilities.contains(capability))
            .map(|(role, _)| role.clone())
            .collect()
    }
    
    /// Update capabilities for a role
    pub fn update_role_capabilities(&mut self, role: GitHubRole, capabilities: Vec<TokaCapability>) {
        debug!("Updating capabilities for role {:?}: {:?}", role, capabilities);
        self.role_mapping.insert(role, capabilities);
    }
    
    /// Add a capability to a role
    pub fn add_capability_to_role(&mut self, role: &GitHubRole, capability: TokaCapability) {
        let capabilities = self.role_mapping.entry(role.clone()).or_insert_with(Vec::new);
        if !capabilities.contains(&capability) {
            debug!("Added capability {:?} to role {:?}", capability, role);
            capabilities.push(capability);
        }
    }
    
    /// Remove a capability from a role
    pub fn remove_capability_from_role(&mut self, role: &GitHubRole, capability: &TokaCapability) {
        if let Some(capabilities) = self.role_mapping.get_mut(role) {
            capabilities.retain(|c| c != capability);
            debug!("Removed capability {:?} from role {:?}", capability, role);
        }
    }
    
    /// Get the minimum role required for a capability
    pub fn min_role_for_capability(&self, capability: &TokaCapability) -> Option<GitHubRole> {
        let roles_with_cap = self.roles_with_capability(capability);
        
        // Define role hierarchy (lowest to highest privilege)
        let role_hierarchy = [
            GitHubRole::Public,
            GitHubRole::Contributor,
            GitHubRole::Collaborator,
            GitHubRole::Maintainer,
            GitHubRole::Owner,
        ];
        
        for role in &role_hierarchy {
            if roles_with_cap.contains(role) {
                return Some(role.clone());
            }
        }
        
        None
    }
}

/// Permission checker for validating access to resources
#[derive(Debug)]
pub struct PermissionChecker {
    /// Permission matrix
    matrix: PermissionMatrix,
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new(matrix: PermissionMatrix) -> Self {
        Self { matrix }
    }
    
    /// Check if a session can perform an action
    pub fn can_perform_action(&self, session: &AuthSession, action: &Action) -> AuthResult<bool> {
        let required_capabilities = action.required_capabilities();
        
        for capability in required_capabilities {
            if !self.matrix.session_has_capability(session, &capability) {
                debug!(
                    "Session {} lacks required capability {:?} for action {:?}",
                    session.session_id, capability, action
                );
                return Ok(false);
            }
        }
        
        debug!(
            "Session {} authorized for action {:?}",
            session.session_id, action
        );
        Ok(true)
    }
    
    /// Require that a session has specific capabilities
    pub fn require_capabilities(
        &self,
        session: &AuthSession,
        capabilities: &[TokaCapability],
    ) -> AuthResult<()> {
        for capability in capabilities {
            if !self.matrix.session_has_capability(session, capability) {
                return Err(AuthError::PermissionDenied {
                    message: format!(
                        "Session {} lacks required capability: {:?}",
                        session.session_id, capability
                    ),
                });
            }
        }
        Ok(())
    }
    
    /// Require that a session can perform an action
    pub fn require_action(&self, session: &AuthSession, action: &Action) -> AuthResult<()> {
        if !self.can_perform_action(session, action)? {
            return Err(AuthError::PermissionDenied {
                message: format!(
                    "Session {} not authorized for action: {:?}",
                    session.session_id, action
                ),
            });
        }
        Ok(())
    }
    
    /// Get the permission matrix
    pub fn matrix(&self) -> &PermissionMatrix {
        &self.matrix
    }
    
    /// Update the permission matrix
    pub fn update_matrix(&mut self, matrix: PermissionMatrix) {
        self.matrix = matrix;
    }
}

/// Actions that can be performed in the collaborative workspace
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Read workspace content
    ReadWorkspace,
    
    /// Fork the workspace to create an experiment
    ForkWorkspace,
    
    /// Create a new experiment
    CreateExperiment,
    
    /// Update an existing experiment
    UpdateExperiment,
    
    /// Delete an experiment
    DeleteExperiment,
    
    /// Create a proposal
    CreateProposal,
    
    /// Update a proposal
    UpdateProposal,
    
    /// Review a proposal
    ReviewProposal,
    
    /// Approve a proposal
    ApproveProposal,
    
    /// Merge a proposal
    MergeProposal,
    
    /// Write to the main workspace
    WriteWorkspace,
    
    /// Manage workspace settings
    ManageWorkspace,
    
    /// Deploy to production
    Deploy,
    
    /// Access admin functions
    AdminAccess,
}

impl Action {
    /// Get the capabilities required for this action
    pub fn required_capabilities(&self) -> Vec<TokaCapability> {
        use Action::*;
        use TokaCapability as Cap;
        
        match self {
            ReadWorkspace => vec![Cap::Read],
            ForkWorkspace => vec![Cap::Read, Cap::Fork],
            CreateExperiment => vec![Cap::Read, Cap::Fork, Cap::Experiment],
            UpdateExperiment => vec![Cap::Read, Cap::Fork, Cap::Experiment],
            DeleteExperiment => vec![Cap::Read, Cap::Fork, Cap::Experiment],
            CreateProposal => vec![Cap::Read, Cap::Fork, Cap::Experiment],
            UpdateProposal => vec![Cap::Read, Cap::Fork, Cap::Experiment],
            ReviewProposal => vec![Cap::Read, Cap::Review],
            ApproveProposal => vec![Cap::Read, Cap::Review, Cap::Write],
            MergeProposal => vec![Cap::Read, Cap::Review, Cap::Write, Cap::Merge],
            WriteWorkspace => vec![Cap::Read, Cap::Write],
            ManageWorkspace => vec![Cap::Read, Cap::Write, Cap::Admin],
            Deploy => vec![Cap::Read, Cap::Write, Cap::Deploy],
            AdminAccess => vec![Cap::Admin],
        }
    }
    
    /// Get all available actions
    pub fn all_actions() -> Vec<Action> {
        vec![
            Action::ReadWorkspace,
            Action::ForkWorkspace,
            Action::CreateExperiment,
            Action::UpdateExperiment,
            Action::DeleteExperiment,
            Action::CreateProposal,
            Action::UpdateProposal,
            Action::ReviewProposal,
            Action::ApproveProposal,
            Action::MergeProposal,
            Action::WriteWorkspace,
            Action::ManageWorkspace,
            Action::Deploy,
            Action::AdminAccess,
        ]
    }
}

/// Middleware for checking permissions on API endpoints
pub mod middleware {
    use std::sync::Arc;
    
    use axum::{
        extract::{Request, State},
        http::StatusCode,
        middleware::Next,
        response::Response,
    };
    use tracing::{error, warn};
    
    use super::*;
    use crate::service::AppState;
    
    /// Middleware to require specific capabilities
    pub async fn require_capabilities(
        capabilities: Vec<TokaCapability>,
    ) -> impl Fn(State<Arc<AppState>>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
        move |State(state): State<Arc<AppState>>, request: Request, next: Next| {
            let caps = capabilities.clone();
            Box::pin(async move {
                // TODO: Extract session from request headers/cookies
                // For now, return unauthorized
                warn!("Permission check not implemented yet");
                Err(StatusCode::UNAUTHORIZED)
            })
        }
    }
    
    /// Middleware to require ability to perform an action
    pub async fn require_action(
        action: Action,
    ) -> impl Fn(State<Arc<AppState>>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
        move |State(state): State<Arc<AppState>>, request: Request, next: Next| {
            let action = action.clone();
            Box::pin(async move {
                // TODO: Extract session from request and check action
                // For now, return unauthorized
                warn!("Action check not implemented yet for action: {:?}", action);
                Err(StatusCode::UNAUTHORIZED)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_role_mapping;
    
    #[test]
    fn test_permission_matrix_creation() {
        let mapping = default_role_mapping();
        let matrix = PermissionMatrix::new(mapping);
        
        // Test that owner has all capabilities
        let owner_caps = matrix.get_capabilities(&GitHubRole::Owner);
        assert!(owner_caps.contains(&TokaCapability::Read));
        assert!(owner_caps.contains(&TokaCapability::Deploy));
        
        // Test that public user only has read access
        let public_caps = matrix.get_capabilities(&GitHubRole::Public);
        assert!(public_caps.contains(&TokaCapability::Read));
        assert!(!public_caps.contains(&TokaCapability::Deploy));
    }
    
    #[test]
    fn test_has_capability() {
        let mapping = default_role_mapping();
        let matrix = PermissionMatrix::new(mapping);
        
        assert!(matrix.has_capability(&GitHubRole::Owner, &TokaCapability::Deploy));
        assert!(!matrix.has_capability(&GitHubRole::Public, &TokaCapability::Deploy));
        assert!(matrix.has_capability(&GitHubRole::Public, &TokaCapability::Read));
    }
    
    #[test]
    fn test_action_capabilities() {
        let read_caps = Action::ReadWorkspace.required_capabilities();
        assert_eq!(read_caps, vec![TokaCapability::Read]);
        
        let merge_caps = Action::MergeProposal.required_capabilities();
        assert!(merge_caps.contains(&TokaCapability::Read));
        assert!(merge_caps.contains(&TokaCapability::Review));
        assert!(merge_caps.contains(&TokaCapability::Write));
        assert!(merge_caps.contains(&TokaCapability::Merge));
    }
    
    #[test]
    fn test_min_role_for_capability() {
        let mapping = default_role_mapping();
        let matrix = PermissionMatrix::new(mapping);
        
        let min_role = matrix.min_role_for_capability(&TokaCapability::Read);
        assert_eq!(min_role, Some(GitHubRole::Public));
        
        let min_role = matrix.min_role_for_capability(&TokaCapability::Deploy);
        assert_eq!(min_role, Some(GitHubRole::Owner));
    }
} 