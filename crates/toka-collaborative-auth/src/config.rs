//! Configuration for the collaborative authentication service

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{GitHubRole, TokaCapability, default_role_mapping};

/// Configuration for the collaborative authentication service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// GitHub OAuth app client ID
    pub github_client_id: String,
    
    /// GitHub OAuth app client secret
    pub github_client_secret: String,
    
    /// OAuth redirect URI
    pub redirect_uri: String,
    
    /// JWT secret for signing tokens
    pub jwt_secret: String,
    
    /// GitHub organization to check membership (optional)
    pub organization: Option<String>,
    
    /// GitHub repository to check access (optional)
    pub repository: Option<String>,
    
    /// Custom role mapping (uses default if None)
    pub role_mapping: Option<HashMap<GitHubRole, Vec<TokaCapability>>>,
    
    /// Session configuration
    pub session: SessionConfig,
    
    /// Web server configuration
    pub server: ServerConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session duration before expiration
    pub duration: Duration,
    
    /// Refresh threshold (refresh when this much time is left)
    pub refresh_threshold: Duration,
    
    /// Maximum number of concurrent sessions per user
    pub max_sessions_per_user: u32,
    
    /// Session cleanup interval
    pub cleanup_interval: Duration,
}

/// Web server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Base URL for the service
    pub base_url: String,
    
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    
    /// Request timeout
    pub request_timeout: Duration,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    
    /// Database URL (for SQL backends)
    pub database_url: Option<String>,
    
    /// Connection pool size
    pub pool_size: u32,
    
    /// Connection timeout
    pub connection_timeout: Duration,
}

/// Storage backend types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage (for testing)
    Memory,
    /// SQLite database
    SQLite,
    /// PostgreSQL database
    PostgreSQL,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Required GitHub scopes
    pub required_scopes: Vec<String>,
    
    /// Enable PKCE for OAuth flow
    pub enable_pkce: bool,
    
    /// OAuth state timeout
    pub oauth_state_timeout: Duration,
    
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    
    /// JWT configuration
    pub jwt: JwtConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    
    /// Requests per minute per IP
    pub requests_per_minute: u32,
    
    /// Requests per minute per user
    pub requests_per_minute_per_user: u32,
    
    /// Burst limit
    pub burst_limit: u32,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT algorithm
    pub algorithm: String,
    
    /// Token issuer
    pub issuer: String,
    
    /// Token audience
    pub audience: String,
    
    /// Key rotation interval
    pub key_rotation_interval: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            github_client_id: String::new(),
            github_client_secret: String::new(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            jwt_secret: String::new(),
            organization: None,
            repository: None,
            role_mapping: None,
            session: SessionConfig::default(),
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(24 * 60 * 60), // 24 hours
            refresh_threshold: Duration::from_secs(2 * 60 * 60), // 2 hours
            max_sessions_per_user: 5,
            cleanup_interval: Duration::from_secs(60 * 60), // 1 hour
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 3000,
            base_url: "http://localhost:3000".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
            request_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::SQLite,
            database_url: Some("sqlite:./data/collaborative-auth.db".to_string()),
            pool_size: 10,
            connection_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            required_scopes: vec![
                "read:user".to_string(),
                "user:email".to_string(),
                "read:org".to_string(),
            ],
            enable_pkce: true,
            oauth_state_timeout: Duration::from_secs(10 * 60), // 10 minutes
            rate_limiting: RateLimitConfig::default(),
            jwt: JwtConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            requests_per_minute_per_user: 100,
            burst_limit: 10,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            algorithm: "HS256".to_string(),
            issuer: "toka-collaborative-auth".to_string(),
            audience: "toka-workspace".to_string(),
            key_rotation_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
}

impl AuthConfig {
    /// Get the role mapping, using default if not specified
    pub fn get_role_mapping(&self) -> HashMap<GitHubRole, Vec<TokaCapability>> {
        self.role_mapping.clone().unwrap_or_else(default_role_mapping)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.github_client_id.is_empty() {
            return Err("github_client_id is required".to_string());
        }
        
        if self.github_client_secret.is_empty() {
            return Err("github_client_secret is required".to_string());
        }
        
        if self.jwt_secret.is_empty() {
            return Err("jwt_secret is required".to_string());
        }
        
        if self.jwt_secret.len() < 32 {
            return Err("jwt_secret must be at least 32 characters".to_string());
        }
        
        if self.redirect_uri.is_empty() {
            return Err("redirect_uri is required".to_string());
        }
        
        if !self.redirect_uri.starts_with("http://") && !self.redirect_uri.starts_with("https://") {
            return Err("redirect_uri must be a valid HTTP(S) URL".to_string());
        }
        
        if self.server.port == 0 {
            return Err("server port must be specified".to_string());
        }
        
        if matches!(self.storage.backend, StorageBackend::SQLite | StorageBackend::PostgreSQL) {
            if self.storage.database_url.is_none() {
                return Err("database_url is required for SQL storage backends".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let mut config = Self::default();
        
        if let Ok(client_id) = std::env::var("GITHUB_CLIENT_ID") {
            config.github_client_id = client_id;
        }
        
        if let Ok(client_secret) = std::env::var("GITHUB_CLIENT_SECRET") {
            config.github_client_secret = client_secret;
        }
        
        if let Ok(redirect_uri) = std::env::var("GITHUB_REDIRECT_URI") {
            config.redirect_uri = redirect_uri;
        }
        
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        }
        
        if let Ok(organization) = std::env::var("GITHUB_ORGANIZATION") {
            config.organization = Some(organization);
        }
        
        if let Ok(repository) = std::env::var("GITHUB_REPOSITORY") {
            config.repository = Some(repository);
        }
        
        if let Ok(base_url) = std::env::var("AUTH_BASE_URL") {
            config.server.base_url = base_url;
        }
        
        if let Ok(port) = std::env::var("AUTH_PORT") {
            config.server.port = port.parse().map_err(|_| "Invalid AUTH_PORT")?;
        }
        
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.storage.database_url = Some(database_url);
        }
        
        config.validate()?;
        Ok(config)
    }
} 