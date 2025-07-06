#![forbid(unsafe_code)]

//! **toka-orchestration-service** – Main orchestration service for Toka OS.
//!
//! This service coordinates agent spawning, lifecycle management, and provides
//! a comprehensive orchestration platform for the Toka agentic operating system.
//!
//! ## Features
//!
//! - **Agent Orchestration**: Manages agent lifecycle and dependencies
//! - **Configuration Management**: Loads and validates agent configurations
//! - **Health Monitoring**: Provides health checks and monitoring endpoints
//! - **LLM Integration**: Intelligent coordination using LLM providers
//! - **Environment Management**: Secure environment variable handling
//!
//! ## Usage
//!
//! ```bash
//! # Start orchestration service
//! toka-orchestration --config config/agents.toml
//!
//! # Start with specific agent configuration
//! toka-orchestration --config config/cursor-agents.toml --cursor-mode
//!
//! # Run in development mode
//! toka-orchestration --config config/agents.toml --dev
//! ```

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use toka_auth::JwtHs256Validator;
use toka_llm_gateway::{Config as LlmConfig, LlmGateway};
use toka_orchestration::{OrchestrationConfig, OrchestrationEngine};
use toka_runtime::{Runtime, RuntimeConfig, StorageConfig};

//─────────────────────────────
//  CLI structure
//─────────────────────────────

#[derive(Parser)]
#[command(name = "toka-orchestration")]
#[command(about = "Toka OS Orchestration Service - Agent coordination and lifecycle management")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(long, default_value = "config/agents.toml")]
    config: String,

    /// Storage backend (memory, sled, sqlite)
    #[arg(long, default_value = "sqlite")]
    storage: String,

    /// Database path for persistent storage
    #[arg(long, default_value = "data/orchestration.db")]
    db_path: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// HTTP server port for health checks and API
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Enable Cursor mode for background agents
    #[arg(long)]
    cursor_mode: bool,

    /// Enable development mode
    #[arg(long)]
    dev: bool,

    /// JWT secret for authentication
    #[arg(long, env = "JWT_SECRET")]
    jwt_secret: Option<String>,
}

//─────────────────────────────
//  Service state
//─────────────────────────────

#[derive(Clone)]
struct ServiceState {
    orchestration_engine: Arc<OrchestrationEngine>,
    runtime: Arc<Runtime>,
    llm_gateway: Option<Arc<LlmGateway>>,
    config: OrchestrationConfig,
}

//─────────────────────────────
//  API types
//─────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    orchestration_status: String,
    agent_count: usize,
    uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    session_id: String,
    current_phase: String,
    progress: f64,
    completed: bool,
    error: Option<String>,
    spawned_agents: usize,
}

//─────────────────────────────
//  Main application
//─────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level)?;

    info!("Starting Toka Orchestration Service v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_orchestration_config(&cli.config)
        .with_context(|| format!("Failed to load configuration from {}", cli.config))?;

    info!("Loaded configuration with {} agents", config.agents.len());

    // Initialize storage
    let storage_config = parse_storage_config(&cli.storage, &cli.db_path)?;
    let runtime_config = RuntimeConfig {
        bus_capacity: 1024,
        storage: storage_config,
        spawn_kernel: false,
        persistence_buffer_size: 256,
    };

    // Initialize authentication
    let jwt_secret = cli.jwt_secret
        .or_else(|| std::env::var("JWT_SECRET").ok())
        .unwrap_or_else(|| "toka-orchestration-secret-change-in-production".to_string());

    let auth = Arc::new(JwtHs256Validator::new(jwt_secret));

    // Initialize runtime
    let runtime = Arc::new(Runtime::new(runtime_config, auth).await?);
    info!("Toka runtime initialized");

    // Initialize LLM gateway
    let llm_gateway = match LlmConfig::from_env() {
        Ok(llm_config) => {
            info!("Initializing LLM gateway with provider: {}", llm_config.provider_name());
            Some(Arc::new(LlmGateway::new(llm_config).await?))
        }
        Err(e) => {
            warn!("Failed to initialize LLM gateway: {}. Continuing without LLM integration.", e);
            None
        }
    };

    // Initialize orchestration engine
    let mut engine = OrchestrationEngine::new(config.clone(), runtime.clone()).await?;

    // Add LLM gateway if available
    if let Some(llm_gateway) = llm_gateway.as_ref() {
        engine = engine.with_llm_gateway(llm_gateway.clone());
        info!("LLM gateway integrated with orchestration engine");
    }

    let engine = Arc::new(engine);

    // Create service state
    let state = ServiceState {
        orchestration_engine: engine.clone(),
        runtime: runtime.clone(),
        llm_gateway: llm_gateway.clone(),
        config: config.clone(),
    };

    // Start orchestration session
    let session = engine.start_orchestration().await?;
    info!("Orchestration session started: {}", session.session_id());

    // Start HTTP server for health checks and API
    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli.port))
        .await
        .with_context(|| format!("Failed to bind to port {}", cli.port))?;

    info!("HTTP server listening on port {}", cli.port);
    info!("Health check endpoint: http://localhost:{}/health", cli.port);
    info!("Status endpoint: http://localhost:{}/status", cli.port);

    // Start the server
    let server = axum::serve(listener, app);

    // Handle graceful shutdown
    let shutdown_signal = shutdown_signal();

    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!("HTTP server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal");
        }
        result = session.wait_for_completion() => {
            match result {
                Ok(()) => info!("Orchestration completed successfully"),
                Err(e) => error!("Orchestration failed: {}", e),
            }
        }
    }

    // Graceful shutdown
    info!("Shutting down orchestration service");
    runtime.shutdown().await?;
    info!("Toka Orchestration Service stopped");

    Ok(())
}

//─────────────────────────────
//  HTTP application
//─────────────────────────────

fn create_app(state: ServiceState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(orchestration_status))
        .route("/agents", get(list_agents))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
}

async fn health_check(State(state): State<ServiceState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Calculate uptime (simplified)
    let uptime_seconds = 0; // TODO: Implement actual uptime tracking

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        orchestration_status: "running".to_string(),
        agent_count: state.config.agents.len(),
        uptime_seconds,
    };

    Ok(Json(response))
}

async fn orchestration_status(State(state): State<ServiceState>) -> Result<Json<StatusResponse>, StatusCode> {
    let session_state = state.orchestration_engine.get_session_state().await;
    let spawned_agents = state.orchestration_engine.get_spawned_agents();

    let response = StatusResponse {
        session_id: session_state.session_id,
        current_phase: format!("{:?}", session_state.current_phase),
        progress: session_state.progress,
        completed: session_state.completed,
        error: session_state.error,
        spawned_agents: spawned_agents.len(),
    };

    Ok(Json(response))
}

async fn list_agents(State(state): State<ServiceState>) -> Result<Json<Vec<String>>, StatusCode> {
    let agent_names: Vec<String> = state.config.agents
        .iter()
        .map(|agent| agent.metadata.name.clone())
        .collect();

    Ok(Json(agent_names))
}

//─────────────────────────────
//  Utility functions
//─────────────────────────────

fn init_logging(log_level: &str) -> Result<()> {
    let log_filter = format!("toka_orchestration_service={},toka_orchestration={},toka_runtime={}", 
                            log_level, log_level, log_level);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

fn load_orchestration_config(config_path: &str) -> Result<OrchestrationConfig> {
    OrchestrationConfig::from_file(config_path)
        .with_context(|| format!("Failed to load orchestration configuration from {}", config_path))
}

fn parse_storage_config(storage_type: &str, db_path: &str) -> Result<StorageConfig> {
    match storage_type {
        "memory" => Ok(StorageConfig::Memory),
        "sled" => Ok(StorageConfig::Sled { path: db_path.to_string() }),
        "sqlite" => Ok(StorageConfig::Sqlite { path: db_path.to_string() }),
        _ => Err(anyhow::anyhow!("Unsupported storage type: {}", storage_type)),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
} 