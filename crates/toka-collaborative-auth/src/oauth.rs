//! OAuth2 implementation for GitHub authentication with PKCE support

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{AuthError, AuthResult, AuthStorage, OAuthState};

/// GitHub OAuth2 URLs
const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// OAuth client for GitHub authentication
#[derive(Clone)]
pub struct OAuthClient {
    /// OAuth2 client
    client: BasicClient,
    /// Required scopes
    scopes: Vec<String>,
    /// Enable PKCE
    enable_pkce: bool,
    /// Storage backend
    storage: Arc<dyn AuthStorage>,
}

/// OAuth flow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    /// Authorization URL for user to visit
    pub auth_url: String,
    /// CSRF state token
    pub state: String,
    /// PKCE code verifier (if PKCE is enabled)
    pub code_verifier: Option<String>,
}

/// OAuth callback parameters
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    /// Authorization code from GitHub
    pub code: String,
    /// State parameter for CSRF protection
    pub state: String,
    /// Error code if authorization failed
    pub error: Option<String>,
    /// Error description if authorization failed
    pub error_description: Option<String>,
}

/// GitHub access token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubTokens {
    /// Access token
    pub access_token: String,
    /// Token type (usually "bearer")
    pub token_type: String,
    /// Granted scopes
    pub scope: String,
}

impl OAuthClient {
    /// Create a new OAuth client
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        scopes: Vec<String>,
        enable_pkce: bool,
        storage: Arc<dyn AuthStorage>,
    ) -> AuthResult<Self> {
        let auth_url = AuthUrl::new(GITHUB_AUTH_URL.to_string())
            .map_err(|e| AuthError::Config(format!("Invalid auth URL: {}", e)))?;
        
        let token_url = TokenUrl::new(GITHUB_TOKEN_URL.to_string())
            .map_err(|e| AuthError::Config(format!("Invalid token URL: {}", e)))?;
        
        let redirect_url = RedirectUrl::new(redirect_uri)
            .map_err(|e| AuthError::Config(format!("Invalid redirect URI: {}", e)))?;
        
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);
        
        Ok(Self {
            client,
            scopes,
            enable_pkce,
            storage,
        })
    }
    
    /// Start OAuth flow and return authorization URL
    pub async fn start_flow(&self, redirect_url: Option<String>) -> AuthResult<OAuthFlow> {
        debug!("Starting OAuth flow with PKCE: {}", self.enable_pkce);
        
        let mut auth_request = self.client.authorize_url(CsrfToken::new_random);
        
        // Add required scopes
        for scope in &self.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }
        
        let code_verifier = if self.enable_pkce {
            let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
            auth_request = auth_request.set_pkce_challenge(challenge);
            Some(verifier)
        } else {
            None
        };
        
        let (auth_url, csrf_token) = auth_request.url();
        
        // Store OAuth state for verification
        let oauth_state = OAuthState {
            state: csrf_token.secret().clone(),
            code_verifier: code_verifier.as_ref().map(|v| v.secret().clone()).unwrap_or_default(),
            expires_at: Utc::now() + chrono::Duration::minutes(10),
            redirect_url,
        };
        
        self.storage.store_oauth_state(&oauth_state).await?;
        
        info!("OAuth flow started with state: {}", csrf_token.secret());
        
        Ok(OAuthFlow {
            auth_url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
            code_verifier: code_verifier.map(|v| v.secret().clone()),
        })
    }
    
    /// Handle OAuth callback and exchange code for token
    pub async fn handle_callback(&self, callback: OAuthCallback) -> AuthResult<GitHubTokens> {
        debug!("Handling OAuth callback for state: {}", callback.state);
        
        // Check for OAuth errors
        if let Some(error) = callback.error {
            let description = callback.error_description.unwrap_or_default();
            error!("OAuth error: {} - {}", error, description);
            return Err(AuthError::OAuth(format!("OAuth error: {} - {}", error, description)));
        }
        
        // Retrieve and validate OAuth state
        let oauth_state = self.storage.consume_oauth_state(&callback.state).await?
            .ok_or_else(|| {
                warn!("Invalid or expired OAuth state: {}", callback.state);
                AuthError::OAuth("Invalid or expired OAuth state".to_string())
            })?;
        
        // Check if state has expired
        if oauth_state.expires_at < Utc::now() {
            warn!("OAuth state expired: {}", callback.state);
            return Err(AuthError::OAuth("OAuth state expired".to_string()));
        }
        
        // Prepare token exchange request
        let mut token_request = self.client
            .exchange_code(AuthorizationCode::new(callback.code));
        
        // Add PKCE verifier if enabled
        if self.enable_pkce && !oauth_state.code_verifier.is_empty() {
            token_request = token_request
                .set_pkce_verifier(PkceCodeVerifier::new(oauth_state.code_verifier));
        }
        
        // Exchange authorization code for access token
        let token_response = token_request
            .request_async(async_http_client)
            .await?;
        
        let tokens = GitHubTokens {
            access_token: token_response.access_token().secret().clone(),
            token_type: "bearer".to_string(),
            scope: token_response.scopes()
                .map(|scopes| {
                    scopes.iter()
                        .map(|scope| scope.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default(),
        };
        
        info!("OAuth flow completed successfully for state: {}", callback.state);
        debug!("Granted scopes: {}", tokens.scope);
        
        Ok(tokens)
    }
    
    /// Validate that required scopes were granted
    pub fn validate_scopes(&self, granted_scopes: &str) -> AuthResult<()> {
        let granted: std::collections::HashSet<&str> = granted_scopes.split_whitespace().collect();
        let required: std::collections::HashSet<&str> = self.scopes.iter().map(|s| s.as_str()).collect();
        
        let missing: Vec<&str> = required.difference(&granted).cloned().collect();
        
        if !missing.is_empty() {
            warn!("Missing required scopes: {:?}", missing);
            return Err(AuthError::OAuth(format!("Missing required scopes: {:?}", missing)));
        }
        
        debug!("All required scopes granted");
        Ok(())
    }
    
    /// Generate a secure random state string
    pub fn generate_state() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
    
    /// Validate OAuth state format
    pub fn validate_state_format(state: &str) -> bool {
        state.len() >= 16 && state.chars().all(|c| c.is_alphanumeric())
    }
}

/// OAuth endpoints for the web service
pub mod endpoints {
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::{Html, Redirect},
        Json,
    };
    use serde_json::json;
    use tracing::{error, info};
    
    use super::*;
    use crate::service::AppState;
    
    /// Start OAuth flow endpoint
    pub async fn start_oauth(
        State(state): State<Arc<AppState>>,
        Query(params): Query<StartOAuthParams>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        match state.oauth_client.start_flow(params.redirect_url).await {
            Ok(flow) => {
                info!("OAuth flow started");
                Ok(Json(json!({
                    "auth_url": flow.auth_url,
                    "state": flow.state
                })))
            }
            Err(e) => {
                error!("Failed to start OAuth flow: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    
    /// OAuth callback endpoint
    pub async fn oauth_callback(
        State(state): State<Arc<AppState>>,
        Query(callback): Query<OAuthCallback>,
    ) -> Result<Html<String>, StatusCode> {
        match state.oauth_client.handle_callback(callback).await {
            Ok(tokens) => {
                // Validate scopes
                if let Err(e) = state.oauth_client.validate_scopes(&tokens.scope) {
                    error!("Scope validation failed: {}", e);
                    return Ok(Html(format!(
                        r#"
                        <!DOCTYPE html>
                        <html>
                        <head><title>Authentication Failed</title></head>
                        <body>
                            <h1>Authentication Failed</h1>
                            <p>Required permissions were not granted: {}</p>
                            <a href="/">Try again</a>
                        </body>
                        </html>
                        "#,
                        e
                    )));
                }
                
                info!("OAuth callback successful");
                
                // TODO: Create user session and redirect to success page
                Ok(Html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Authentication Successful</title></head>
                    <body>
                        <h1>Authentication Successful</h1>
                        <p>You have been successfully authenticated!</p>
                        <script>
                            // Redirect to main app or close popup
                            if (window.opener) {
                                window.opener.postMessage({type: 'auth_success'}, '*');
                                window.close();
                            } else {
                                window.location.href = '/';
                            }
                        </script>
                    </body>
                    </html>
                    "#.to_string()
                ))
            }
            Err(e) => {
                error!("OAuth callback failed: {}", e);
                Ok(Html(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Authentication Failed</title></head>
                    <body>
                        <h1>Authentication Failed</h1>
                        <p>Error: {}</p>
                        <a href="/">Try again</a>
                    </body>
                    </html>
                    "#,
                    e
                )))
            }
        }
    }
    
    #[derive(Debug, Deserialize)]
    pub struct StartOAuthParams {
        pub redirect_url: Option<String>,
    }
} 