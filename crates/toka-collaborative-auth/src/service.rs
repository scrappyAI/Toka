//! Main authentication service implementation

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::{HeaderValue, StatusCode},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::{
    config::AuthConfig,
    github::GitHubClient,
    oauth::{endpoints::*, OAuthClient},
    permissions::{PermissionChecker, PermissionMatrix},
    session::{extraction::AuthenticatedSession, SessionManager},
    storage::StorageFactory,
    AuthError, AuthEvent, AuthEventType, AuthResult, AuthStorage, GitHubUser,
};

/// Main authentication service  
pub struct AuthService {
    /// Service configuration
    config: AuthConfig,
    /// Application state
    state: Arc<AppState>,
}

/// Application state shared across handlers
pub struct AppState {
    /// OAuth client for GitHub authentication
    pub oauth_client: OAuthClient,
    /// GitHub API client
    pub github_client: GitHubClient,
    /// Session manager
    pub session_manager: SessionManager,
    /// Permission checker
    pub permission_checker: PermissionChecker,
    /// Storage backend
    pub storage: Arc<dyn AuthStorage>,
    /// Service configuration
    pub config: AuthConfig,
}

impl AuthService {
    /// Create a new authentication service
    pub async fn new(config: AuthConfig) -> AuthResult<Self> {
        // Validate configuration
        config.validate().map_err(AuthError::Config)?;
        
        // Create storage backend
        let storage = StorageFactory::create_storage(
            &config.storage.backend,
            config.storage.database_url.as_deref(),
        ).await?;
        
        // Create OAuth client
        let oauth_client = OAuthClient::new(
            config.github_client_id.clone(),
            config.github_client_secret.clone(),
            config.redirect_uri.clone(),
            config.security.required_scopes.clone(),
            config.security.enable_pkce,
            storage.clone(),
        )?;
        
        // Create GitHub client
        let github_client = GitHubClient::new(
            config.organization.clone(),
            config.repository.clone(),
        );
        
        // Create permission matrix
        let permission_matrix = PermissionMatrix::new(config.get_role_mapping());
        
        // Create session manager
        let session_manager = SessionManager::new(
            config.jwt_secret.clone(),
            config.security.jwt.issuer.clone(),
            config.security.jwt.audience.clone(),
            config.session.duration,
            config.session.refresh_threshold,
            config.session.max_sessions_per_user,
            storage.clone(),
            permission_matrix.clone(),
        )?;
        
        // Create permission checker
        let permission_checker = PermissionChecker::new(permission_matrix);
        
        let state = Arc::new(AppState {
            oauth_client,
            github_client,
            session_manager,
            permission_checker,
            storage,
            config: config.clone(),
        });
        
        Ok(Self { config, state })
    }
    
    /// Start the authentication service
    pub async fn start(self) -> AuthResult<()> {
        info!("Starting Toka Collaborative Authentication Service");
        info!("Server: {}:{}", self.config.server.bind_address, self.config.server.port);
        
        // Start background tasks
        let cleanup_state = self.state.clone();
        let cleanup_interval = self.config.session.cleanup_interval;
        tokio::spawn(async move {
            Self::cleanup_task(cleanup_state, cleanup_interval).await;
        });
        
        // Create router
        let app = self.create_router();
        
        // Start server
        let bind_addr = format!("{}:{}", self.config.server.bind_address, self.config.server.port);
        let listener = tokio::net::TcpListener::bind(&bind_addr).await
            .map_err(|e| AuthError::Network(format!("Failed to bind to {}: {}", bind_addr, e)))?;
        
        info!("Authentication service listening on {}", bind_addr);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| AuthError::Network(format!("Server error: {}", e)))?;
        
        Ok(())
    }
    
    /// Create the router with all endpoints
    fn create_router(&self) -> Router {
        // Create CORS layer
        let cors = CorsLayer::new()
            .allow_origin(
                self.config.server.cors_origins
                    .iter()
                    .map(|origin| origin.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>()
            )
            .allow_methods(Any)
            .allow_headers(Any);
        
        Router::new()
            // OAuth endpoints
            .route("/auth/start", get(start_oauth))
            .route("/auth/callback", get(oauth_callback))
            
            // Session endpoints
            .route("/auth/session", get(get_session_info))
            .route("/auth/refresh", post(refresh_session))
            .route("/auth/logout", post(logout))
            
            // Health and info endpoints
            .route("/health", get(health_check))
            .route("/auth/info", get(service_info))
            
            // User endpoints
            .route("/auth/user", get(get_user_info))
            .route("/auth/user/sessions", get(get_user_sessions))
            
            // Admin endpoints (require admin capabilities)
            .route("/auth/admin/stats", get(get_service_stats))
            .route("/auth/admin/cleanup", post(force_cleanup))
            
            // Static files for login page
            .route("/", get(login_page))
            
            .layer(
                ServiceBuilder::new()
                    .layer(cors)
                    .into_inner()
            )
            .with_state(self.state.clone())
    }
    
    /// Background cleanup task
    async fn cleanup_task(state: Arc<AppState>, interval: Duration) {
        let mut cleanup_interval = time::interval(interval);
        
        loop {
            cleanup_interval.tick().await;
            
            match state.session_manager.cleanup_expired_sessions().await {
                Ok(cleaned) => {
                    if cleaned > 0 {
                        info!("Cleanup: removed {} expired items", cleaned);
                    }
                }
                Err(e) => {
                    error!("Cleanup task failed: {}", e);
                }
            }
        }
    }
    
    /// Get the application state
    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }
    
    /// Get the service configuration
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

/// Complete OAuth flow and create session
pub async fn complete_oauth_flow(
    state: &AppState,
    callback: crate::oauth::OAuthCallback,
) -> AuthResult<crate::AuthSession> {
    // Handle OAuth callback
    let tokens = state.oauth_client.handle_callback(callback).await?;
    
    // Validate scopes
    state.oauth_client.validate_scopes(&tokens.scope)?;
    
    // Get GitHub user information
    let github_user_response = state.github_client.get_user(&tokens.access_token).await?;
    let github_user = state.github_client.create_github_user(&tokens.access_token, github_user_response).await?;
    
    // Create authentication session
    let session = state.session_manager.create_session(github_user.clone()).await?;
    
    // Log authentication event
    let auth_event = AuthEvent {
        timestamp: chrono::Utc::now(),
        event_type: AuthEventType::SessionCreated,
        github_user: Some(github_user),
        session_id: Some(session.session_id),
        details: std::collections::HashMap::new(),
    };
    
    if let Err(e) = state.storage.store_auth_event(&auth_event).await {
        warn!("Failed to log auth event: {}", e);
    }
    
    Ok(session)
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Get current session information
async fn get_session_info(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "session_id": session.0.session_id,
        "user": {
            "id": session.0.github_user.id,
            "login": session.0.github_user.login,
            "name": session.0.github_user.name,
            "avatar_url": session.0.github_user.avatar_url,
            "role": session.0.github_user.role
        },
        "capabilities": session.0.capabilities,
        "created_at": session.0.created_at,
        "expires_at": session.0.expires_at,
        "last_activity": session.0.last_activity
    })))
}

/// Refresh current session
async fn refresh_session(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.session_manager.refresh_session(&session.0).await {
        Ok(Some(refreshed_session)) => {
            Ok(Json(json!({
                "refreshed": true,
                "new_token": refreshed_session.jwt_token,
                "expires_at": refreshed_session.expires_at
            })))
        }
        Ok(None) => {
            Ok(Json(json!({
                "refreshed": false,
                "message": "Session does not need refresh yet"
            })))
        }
        Err(e) => {
            error!("Failed to refresh session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Logout (revoke session)
async fn logout(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.session_manager.revoke_session(&session.0.session_id).await {
        Ok(()) => {
            Ok(Json(json!({
                "success": true,
                "message": "Session revoked successfully"
            })))
        }
        Err(e) => {
            error!("Failed to revoke session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user information
async fn get_user_info(
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "id": session.0.github_user.id,
        "login": session.0.github_user.login,
        "name": session.0.github_user.name,
        "email": session.0.github_user.email,
        "avatar_url": session.0.github_user.avatar_url,
        "role": session.0.github_user.role,
        "capabilities": session.0.capabilities
    })))
}

/// Get user's sessions (requires the user to be authenticated)
async fn get_user_sessions(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.session_manager.get_user_sessions(session.0.github_user.id).await {
        Ok(sessions) => {
            let session_list: Vec<serde_json::Value> = sessions.into_iter().map(|s| json!({
                "session_id": s.session_id,
                "created_at": s.created_at,
                "expires_at": s.expires_at,
                "last_activity": s.last_activity
            })).collect();
            
            Ok(Json(json!({
                "sessions": session_list
            })))
        }
        Err(e) => {
            error!("Failed to get user sessions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Health check endpoint
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "toka-collaborative-auth",
        "timestamp": chrono::Utc::now()
    })))
}

/// Service information endpoint
async fn service_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "service": "toka-collaborative-auth",
        "version": env!("CARGO_PKG_VERSION"),
        "github_organization": state.config.organization,
        "github_repository": state.config.repository,
        "session_duration": state.config.session.duration.as_secs(),
        "max_sessions_per_user": state.config.session.max_sessions_per_user,
        "storage_backend": state.config.storage.backend
    })))
}

/// Service statistics (admin only)
async fn get_service_stats(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check admin capability
    use crate::TokaCapability;
    match state.permission_checker.require_capabilities(&session.0, &[TokaCapability::Admin]) {
        Ok(()) => {},
        Err(_) => return Err(StatusCode::FORBIDDEN),
    }
    
    // Get statistics (this would need to be implemented in storage)
    Ok(Json(json!({
        "active_sessions": 0, // TODO: implement
        "total_users": 0,     // TODO: implement
        "recent_logins": 0    // TODO: implement
    })))
}

/// Force cleanup (admin only)
async fn force_cleanup(
    State(state): State<Arc<AppState>>,
    session: AuthenticatedSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check admin capability
    use crate::TokaCapability;
    match state.permission_checker.require_capabilities(&session.0, &[TokaCapability::Admin]) {
        Ok(()) => {},
        Err(_) => return Err(StatusCode::FORBIDDEN),
    }
    
    match state.session_manager.cleanup_expired_sessions().await {
        Ok(cleaned) => {
            Ok(Json(json!({
                "success": true,
                "cleaned_items": cleaned
            })))
        }
        Err(e) => {
            error!("Failed to force cleanup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Login page
async fn login_page(
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let page = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Toka Collaborative Authentication</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            background: white;
            padding: 3rem;
            border-radius: 1rem;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            text-align: center;
            max-width: 400px;
            width: 90%;
        }}
        .logo {{
            font-size: 2rem;
            font-weight: bold;
            color: #333;
            margin-bottom: 0.5rem;
        }}
        .subtitle {{
            color: #666;
            margin-bottom: 2rem;
        }}
        .btn {{
            background: #24292e;
            color: white;
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: 0.5rem;
            font-size: 1rem;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            transition: background 0.2s;
        }}
        .btn:hover {{
            background: #0366d6;
        }}
        .info {{
            margin-top: 2rem;
            font-size: 0.875rem;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">🔬 Toka</div>
        <div class="subtitle">Collaborative AI Development</div>
        
        <p>Authenticate with GitHub to access the collaborative workspace</p>
        
        <button class="btn" onclick="startAuth()">
            Sign in with GitHub
        </button>
        
        <div class="info">
            <p><strong>Organization:</strong> {}</p>
            <p><strong>Repository:</strong> {}</p>
        </div>
    </div>

    <script>
        async function startAuth() {{
            try {{
                const response = await fetch('/auth/start');
                const data = await response.json();
                window.location.href = data.auth_url;
            }} catch (error) {{
                console.error('Failed to start authentication:', error);
                alert('Failed to start authentication. Please try again.');
            }}
        }}
    </script>
</body>
</html>
    "#, 
        state.config.organization.as_deref().unwrap_or("Not specified"),
        state.config.repository.as_deref().unwrap_or("Not specified")
    );
    
    Ok(Html(page))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, StorageBackend};
    
    fn create_test_config() -> AuthConfig {
        AuthConfig {
            github_client_id: "test-client-id".to_string(),
            github_client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            jwt_secret: "test-jwt-secret-that-is-long-enough-for-security".to_string(),
            organization: Some("test-org".to_string()),
            repository: Some("test-repo".to_string()),
            storage: crate::config::StorageConfig {
                backend: StorageBackend::Memory,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    #[tokio::test]
    async fn test_auth_service_creation() {
        let config = create_test_config();
        let service = AuthService::new(config).await.unwrap();
        
        assert_eq!(service.config.github_client_id, "test-client-id");
        assert_eq!(service.config.organization, Some("test-org".to_string()));
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = create_test_config();
        assert!(config.validate().is_ok());
        
        config.jwt_secret = "too_short".to_string();
        assert!(config.validate().is_err());
    }
} 