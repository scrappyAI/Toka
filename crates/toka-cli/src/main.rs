#![forbid(unsafe_code)]

//! **toka-cli** – Command-line interface for Toka OS.
//!
//! This CLI provides a convenient way to interact with the Toka agentic operating
//! system, allowing users to submit operations, query system state, and manage
//! the runtime configuration.

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use toka_auth::{JwtHs256Validator, TokenValidator, Claims};
use toka_runtime::{Runtime, RuntimeConfig, StorageConfig};
use toka_types::{Message, Operation, TaskSpec, AgentSpec, EntityId};

//─────────────────────────────
//  CLI structure
//─────────────────────────────

#[derive(Parser)]
#[command(name = "toka")]
#[command(about = "Toka OS - Agentic Operating System CLI")]
#[command(version)]
struct Cli {
    /// Storage backend to use (memory, sled, sqlite)
    #[arg(long, default_value = "memory")]
    storage: String,

    /// Database path for persistent storage backends
    #[arg(long, default_value = "toka.db")]
    db_path: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// JWT secret for authentication (use a secure secret in production)
    #[arg(long, default_value = "toka-development-secret-change-in-production")]
    jwt_secret: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Schedule a task for an agent
    ScheduleTask {
        /// Agent entity ID
        #[arg(long)]
        agent: u128,
        /// Task description
        #[arg(long)]
        description: String,
    },
    /// Spawn a new agent
    SpawnAgent {
        /// Agent name
        #[arg(long)]
        name: String,
    },
    /// Query the current world state
    QueryState,
    /// Run the runtime in daemon mode (listen for events)
    Daemon,
    /// Generate a development JWT token
    GenerateToken {
        /// Subject (user/agent identifier)
        #[arg(long, default_value = "dev-user")]
        subject: String,
        /// Vault identifier
        #[arg(long, default_value = "dev-vault")]
        vault: String,
        /// Permissions (comma-separated)
        #[arg(long, default_value = "read,write")]
        permissions: String,
    },
}

//─────────────────────────────
//  Main application
//─────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    init_tracing(&cli.log_level)?;

    info!("Starting Toka CLI v{}", env!("CARGO_PKG_VERSION"));

    // Parse storage configuration
    let storage_config = parse_storage_config(&cli.storage, &cli.db_path)?;
    debug!("Storage config: {:?}", storage_config);

    // Create runtime configuration
    let runtime_config = RuntimeConfig {
        bus_capacity: 1024,
        storage: storage_config,
        spawn_kernel: false,
        persistence_buffer_size: 256,
    };

    // Create authentication validator
    let auth: Arc<dyn TokenValidator> = Arc::new(
        JwtHs256Validator::new(cli.jwt_secret.as_bytes())
    );

    // Create and run the runtime
    let runtime = Runtime::new(runtime_config, auth).await?;
    info!("Toka runtime initialized successfully");

    // Execute the command
    match cli.command {
        Commands::ScheduleTask { agent, description } => {
            handle_schedule_task(&runtime, agent, description).await?;
        }
        Commands::SpawnAgent { name } => {
            handle_spawn_agent(&runtime, name).await?;
        }
        Commands::QueryState => {
            handle_query_state(&runtime).await?;
        }
        Commands::Daemon => {
            handle_daemon(&runtime).await?;
        }
        Commands::GenerateToken { subject, vault, permissions } => {
            handle_generate_token(&cli.jwt_secret, subject, vault, permissions)?;
        }
    }

    // Graceful shutdown
    runtime.shutdown().await?;
    info!("Toka CLI shutting down");

    Ok(())
}

//─────────────────────────────
//  Command handlers
//─────────────────────────────

async fn handle_schedule_task(runtime: &Runtime, agent_id: u128, description: String) -> Result<()> {
    let agent = EntityId(agent_id);
    let task = TaskSpec { description: description.clone() };

    // For demo purposes, use a simple token. In production, this would be provided by the user.
    let message = Message {
        origin: agent,
        capability: "demo-token".to_string(),
        op: Operation::ScheduleAgentTask { agent, task },
    };

    info!("Scheduling task for agent {}: {}", agent_id, description);
    let event = runtime.submit(message).await?;
    
    println!("✅ Task scheduled successfully!");
    println!("📋 Event: {:?}", event);

    Ok(())
}

async fn handle_spawn_agent(runtime: &Runtime, name: String) -> Result<()> {
    let parent = EntityId(0); // Use entity 0 as the system parent
    let spec = AgentSpec { name: name.clone() };

    let message = Message {
        origin: parent,
        capability: "demo-token".to_string(),
        op: Operation::SpawnAgent { parent, spec },
    };

    info!("Spawning agent: {}", name);
    let event = runtime.submit(message).await?;
    
    println!("✅ Agent spawned successfully!");
    println!("🤖 Event: {:?}", event);

    Ok(())
}

async fn handle_query_state(runtime: &Runtime) -> Result<()> {
    let state = runtime.world_state();
    let state_guard = state.read().await;

    println!("🌍 Current World State:");
    println!("📊 Agents: {}", state_guard.agents.len());
    
    for (id, agent) in &state_guard.agents {
        println!("  🤖 Agent {}: {}", id.0, agent.name);
    }

    println!("📋 Total tasks across all agents: {}", 
        state_guard.agent_tasks.values().map(|tasks| tasks.len()).sum::<usize>());

    for (agent_id, tasks) in &state_guard.agent_tasks {
        if !tasks.is_empty() {
            println!("  📝 Agent {} has {} tasks:", agent_id.0, tasks.len());
            for (i, task) in tasks.iter().enumerate() {
                println!("    {}. {}", i + 1, task.description);
            }
        }
    }

    Ok(())
}

async fn handle_daemon(runtime: &Runtime) -> Result<()> {
    println!("🚀 Starting Toka daemon mode...");
    println!("📡 Listening for events (Press Ctrl+C to stop)");

    let mut rx = runtime.subscribe();

    // Handle Ctrl+C gracefully
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            // Handle incoming events
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        println!("📨 Event received: {:?}", event);
                        info!("Event received: {:?}", event);
                    }
                    Err(e) => {
                        error!("Error receiving event: {}", e);
                        break;
                    }
                }
            }
            // Handle Ctrl+C
            _ = &mut ctrl_c => {
                println!("\n⚡ Shutdown signal received");
                break;
            }
        }
    }

    println!("👋 Daemon mode stopped");
    Ok(())
}

fn handle_generate_token(secret: &str, subject: String, vault: String, permissions: String) -> Result<()> {
    use toka_auth::{JwtHs256Token, CapabilityToken};
    
    let perms: Vec<String> = permissions.split(',').map(|s| s.trim().to_string()).collect();
    
    let claims = Claims {
        sub: subject.clone(),
        vault: vault.clone(),
        permissions: perms.clone(),
        iat: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        exp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() + 86400, // 24 hours
        jti: uuid::Uuid::new_v4().to_string(),
    };

    // This is a blocking operation but should be fine for CLI usage
    let token = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            JwtHs256Token::mint(&claims, secret.as_bytes()).await
        })
    })?;

    println!("🔑 Generated JWT token:");
    println!("👤 Subject: {}", subject);
    println!("🏪 Vault: {}", vault);
    println!("🔐 Permissions: {}", permissions);
    println!("🎫 Token: {}", token.as_str());
    println!("\n💡 Use this token as the 'capability' field in messages");

    Ok(())
}

//─────────────────────────────
//  Utility functions
//─────────────────────────────

fn init_tracing(log_level: &str) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

fn parse_storage_config(storage_type: &str, db_path: &str) -> Result<StorageConfig> {
    match storage_type.to_lowercase().as_str() {
        "memory" => Ok(StorageConfig::Memory),
        #[cfg(feature = "sled-storage")]
        "sled" => Ok(StorageConfig::Sled {
            path: db_path.to_string(),
        }),
        #[cfg(feature = "sqlite-storage")]
        "sqlite" => Ok(StorageConfig::Sqlite {
            path: db_path.to_string(),
        }),
        _ => Err(anyhow::anyhow!(
            "Unsupported storage type: {}. Supported types: memory{}{}",
            storage_type,
            #[cfg(feature = "sled-storage")] ", sled" #[cfg(not(feature = "sled-storage"))] "",
            #[cfg(feature = "sqlite-storage")] ", sqlite" #[cfg(not(feature = "sqlite-storage"))] ""
        )),
    }
}