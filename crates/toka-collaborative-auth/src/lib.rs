#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! **toka-collaborative-auth** – GitHub OAuth abstraction and role assignment for collaborative Toka workspace
//!
//! This crate provides a secure, web-based authentication system that enables humans and AI agents
//! to collaborate seamlessly in the Toka workspace. It builds upon GitHub's OAuth2 flow to provide
//! automatic role assignment and capability token generation.
//!
//! ## Architecture
//!
//! The collaborative authentication system consists of:
//!
//! - **OAuth Gateway**: Web-based GitHub OAuth2 flow with automatic user onboarding
//! - **Role Assignment Engine**: Maps GitHub permissions to Toka capability tokens
//! - **Permission Matrix**: Fine-grained access control for collaborative features
//! - **Session Management**: Secure session handling with automatic token refresh
//!
//! ## Usage
//!
//! ```rust,no_run
//! use toka_collaborative_auth::{AuthService, AuthConfig, GitHubRole};
//! use std::collections::HashMap;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! // Configure the authentication service
//! let config = AuthConfig {
//!     github_client_id: "your-github-app-id".to_string(),
//!     github_client_secret: "your-github-app-secret".to_string(),
//!     redirect_uri: "http://localhost:3000/auth/callback".to_string(),
//!     jwt_secret: "your-jwt-secret".to_string(),
//!     organization: Some("your-org".to_string()),
//!     ..Default::default()
//! };
//!
//! // Start the authentication service
//! let auth_service = AuthService::new(config).await?;
//! auth_service.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! The authentication system follows security-first principles:
//!
//! - **OAuth2 Security**: Uses GitHub's secure OAuth2 flow with PKCE
//! - **Capability Tokens**: Integrates with Toka's capability-based security
//! - **Role Isolation**: Clear boundaries between read/write/admin/experiment permissions
//! - **Session Security**: Secure JWT tokens with automatic rotation
//! - **Audit Logging**: Comprehensive logging of all authentication events

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod config;
pub mod github;
pub mod oauth;
pub mod permissions;
pub mod service;
pub mod session;
pub mod storage;

pub use config::AuthConfig;
pub use permissions::{Action, PermissionMatrix, PermissionChecker};
pub use service::AuthService;
pub use session::{SessionManager, SessionClaims};
pub use storage::{MemoryStorage, StorageFactory};

/// GitHub user roles that can be automatically detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitHubRole {
    /// Organization owner
    Owner,
    /// Repository maintainer
    Maintainer,
    /// Repository collaborator with write access
    Collaborator,
    /// External contributor with read access
    Contributor,
    /// Public user with no special access
    Public,
}

/// Toka workspace capabilities mapped from GitHub roles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokaCapability {
    /// Read-only access to public workspace content
    Read,
    /// Fork and experiment in isolated environments
    Fork,
    /// Create experiments and proposals
    Experiment,
    /// Review and comment on proposals
    Review,
    /// Write access to the workspace
    Write,
    /// Merge proposals and manage branches
    Merge,
    /// Administrative access to workspace settings
    Admin,
    /// Deploy and manage production systems
    Deploy,
}

/// User authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    /// Unique session identifier
    pub session_id: Uuid,
    /// GitHub user information
    pub github_user: GitHubUser,
    /// Assigned Toka capabilities
    pub capabilities: Vec<TokaCapability>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// JWT token for this session
    pub jwt_token: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    /// GitHub user ID
    pub id: u64,
    /// GitHub username
    pub login: String,
    /// Display name
    pub name: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// Avatar URL
    pub avatar_url: String,
    /// User's role in the organization/repository
    pub role: GitHubRole,
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    /// Unique state identifier
    pub state: String,
    /// PKCE code verifier
    pub code_verifier: String,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Original redirect URL after authentication
    pub redirect_url: Option<String>,
}

/// Authentication event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuthEventType,
    /// GitHub user involved
    pub github_user: Option<GitHubUser>,
    /// Session ID if applicable
    pub session_id: Option<Uuid>,
    /// Additional event details
    pub details: HashMap<String, String>,
}

/// Types of authentication events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthEventType {
    /// User started OAuth flow
    OAuthStarted,
    /// OAuth flow completed successfully
    OAuthCompleted,
    /// OAuth flow failed
    OAuthFailed,
    /// Session created
    SessionCreated,
    /// Session refreshed
    SessionRefreshed,
    /// Session expired
    SessionExpired,
    /// Session revoked
    SessionRevoked,
    /// Capability check performed
    CapabilityChecked,
    /// Access denied due to insufficient capabilities
    AccessDenied,
}

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// OAuth-related errors
    #[error("OAuth error: {0}")]
    OAuth(String),
    
    /// GitHub API errors
    #[error("GitHub API error: {0}")]
    GitHub(String),
    
    /// JWT token errors
    #[error("JWT error: {0}")]
    Jwt(String),
    
    /// Session-related errors
    #[error("Session error: {0}")]
    Session(String),
    
    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(String),
}

impl From<oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>> for AuthError {
    fn from(err: oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>) -> Self {
        AuthError::OAuth(err.to_string())
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::Network(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AuthError::Jwt(err.to_string())
    }
}

#[cfg(feature = "sqlite-storage")]
impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Storage(err.to_string())
    }
}

/// Trait for storage backends  
#[async_trait::async_trait]
pub trait AuthStorage: Send + Sync + std::fmt::Debug {
    /// Store an OAuth state
    async fn store_oauth_state(&self, state: &OAuthState) -> AuthResult<()>;
    
    /// Retrieve and remove an OAuth state
    async fn consume_oauth_state(&self, state_id: &str) -> AuthResult<Option<OAuthState>>;
    
    /// Store an authentication session
    async fn store_session(&self, session: &AuthSession) -> AuthResult<()>;
    
    /// Retrieve a session by ID
    async fn get_session(&self, session_id: &Uuid) -> AuthResult<Option<AuthSession>>;
    
    /// Update a session
    async fn update_session(&self, session: &AuthSession) -> AuthResult<()>;
    
    /// Remove a session
    async fn remove_session(&self, session_id: &Uuid) -> AuthResult<()>;
    
    /// Clean up expired sessions and states
    async fn cleanup_expired(&self) -> AuthResult<u64>;
    
    /// Store an authentication event
    async fn store_auth_event(&self, event: &AuthEvent) -> AuthResult<()>;
}

/// Default role mapping from GitHub roles to Toka capabilities
pub fn default_role_mapping() -> HashMap<GitHubRole, Vec<TokaCapability>> {
    use GitHubRole::*;
    use TokaCapability as Cap;
    
    let mut mapping = HashMap::new();
    
    // Organization owner gets full access
    mapping.insert(Owner, vec![Cap::Read, Cap::Fork, Cap::Experiment, Cap::Review, Cap::Write, Cap::Merge, Cap::Admin, Cap::Deploy]);
    
    // Repository maintainer gets all permissions except deploy
    mapping.insert(Maintainer, vec![Cap::Read, Cap::Fork, Cap::Experiment, Cap::Review, Cap::Write, Cap::Merge]);
    
    // Collaborator gets write access and can review
    mapping.insert(Collaborator, vec![Cap::Read, Cap::Fork, Cap::Experiment, Cap::Review, Cap::Write]);
    
    // Contributor can experiment and fork
    mapping.insert(Contributor, vec![Cap::Read, Cap::Fork, Cap::Experiment]);
    
    // Public user gets basic read access
    mapping.insert(Public, vec![Cap::Read]);
    
    mapping
} 