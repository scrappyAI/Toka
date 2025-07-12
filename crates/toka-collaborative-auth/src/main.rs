use std::env;
use tracing::{info, error};
use toka_collaborative_auth::{AuthService, AuthConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Toka Collaborative Auth Service");
    
    // Load configuration from environment
    let config = AuthConfig {
        github_client_id: env::var("GITHUB_CLIENT_ID")
            .expect("GITHUB_CLIENT_ID environment variable is required"),
        github_client_secret: env::var("GITHUB_CLIENT_SECRET")
            .expect("GITHUB_CLIENT_SECRET environment variable is required"),
        redirect_uri: env::var("REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
        jwt_secret: env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
        organization: env::var("GITHUB_ORGANIZATION").ok(),
        repository: env::var("GITHUB_REPOSITORY").ok(),
        role_mapping: None,
        session: toka_collaborative_auth::config::SessionConfig {
            duration: std::time::Duration::from_secs(
                env::var("SESSION_DURATION_HOURS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(24) * 60 * 60
            ),
            ..Default::default()
        },
        server: toka_collaborative_auth::config::ServerConfig {
            bind_address: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            ..Default::default()
        },
        storage: Default::default(),
        security: toka_collaborative_auth::config::SecurityConfig {
            enable_pkce: env::var("ENABLE_PKCE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            ..Default::default()
        },
    };
    
    info!("📋 Configuration loaded:");
    info!("  - GitHub Client ID: {}", config.github_client_id);
    info!("  - Redirect URI: {}", config.redirect_uri);
    info!("  - Organization: {:?}", config.organization);
    info!("  - Repository: {:?}", config.repository);
    info!("  - Session Duration: {} hours", config.session.duration.as_secs() / 3600);
    info!("  - PKCE Enabled: {}", config.security.enable_pkce);
    
    // Create and start the auth service
    match AuthService::new(config).await {
        Ok(service) => {
            info!("✅ Auth service initialized successfully");
            info!("🌐 Starting web server...");
            info!("📱 Visit http://localhost:3000/auth/login to test the OAuth flow");
            
            if let Err(e) = service.start().await {
                error!("❌ Failed to start auth service: {}", e);
                return Err(e.into());
            }
        }
        Err(e) => {
            error!("❌ Failed to initialize auth service: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
} 