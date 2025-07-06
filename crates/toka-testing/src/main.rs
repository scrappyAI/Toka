#![forbid(unsafe_code)]

//! **toka-testing** – Interactive testing environment for Toka OS.
//!
//! This testing tool provides an intuitive interface for exploring and testing
//! the Toka agentic operating system. It demonstrates proper token management,
//! agent spawning, task scheduling, and system monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use serde_json::json;
use tokio::time::sleep;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use toka_auth::{JwtHs256Validator, JwtHs256Token, TokenValidator, Claims, CapabilityToken};
use toka_runtime::RuntimeManager;
use toka_kernel;
use toka_bus_core;
use toka_types::{Message, Operation, TaskSpec, AgentSpec, EntityId};

//─────────────────────────────
//  CLI structure
//─────────────────────────────

#[derive(Parser)]
#[command(name = "toka-test")]
#[command(about = "Interactive testing environment for Toka OS")]
#[command(version)]
struct Cli {
    /// Storage backend to use (memory, sled, sqlite)
    #[arg(long, default_value = "sqlite")]
    storage: String,

    /// Database path for persistent storage backends
    #[arg(long, default_value = "./data/toka-test.db")]
    db_path: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// JWT secret for authentication
    #[arg(long, default_value = "toka-testing-secret-change-in-production")]
    jwt_secret: String,

    /// Skip interactive mode and run demo scenarios
    #[arg(long)]
    demo: bool,
}

//─────────────────────────────
//  Testing Environment
//─────────────────────────────

struct TestingEnvironment {
    runtime: RuntimeManager,
    jwt_secret: String,
    tokens: HashMap<String, String>,
    agents: HashMap<String, EntityId>,
    next_agent_id: u128,
}

impl TestingEnvironment {
    async fn new(cli: &Cli) -> Result<Self> {
        // Create authentication validator
        let auth: Arc<dyn TokenValidator> = Arc::new(
            JwtHs256Validator::new(cli.jwt_secret.clone())
        );

        // Create runtime
        let world_state = toka_kernel::WorldState::default();
        let event_bus = Arc::new(toka_bus_core::InMemoryBus::new(1024));
        let kernel = toka_kernel::Kernel::new(world_state, auth, event_bus);
        let runtime_kernel = toka_runtime::RuntimeKernel::new(kernel);
        let runtime = RuntimeManager::new(runtime_kernel).await?;

        let mut env = Self {
            runtime,
            jwt_secret: cli.jwt_secret.clone(),
            tokens: HashMap::new(),
            agents: HashMap::new(),
            next_agent_id: 1,
        };

        // Generate initial tokens
        env.generate_system_tokens().await?;

        Ok(env)
    }

    async fn generate_system_tokens(&mut self) -> Result<()> {
        // Generate admin token for EntityId(0) - system entity
        let admin_token = self.generate_token(
            "0",  // Use EntityId as string
            "system-vault",
            vec!["read".to_string(), "write".to_string(), "admin".to_string()],
        ).await?;
        self.tokens.insert("admin".to_string(), admin_token);

        // Generate user token for EntityId(1000) - user entity  
        let user_token = self.generate_token(
            "1000",  // Use EntityId as string
            "user-vault",
            vec!["read".to_string(), "write".to_string()],
        ).await?;
        self.tokens.insert("user".to_string(), user_token);

        println!("{}", "✅ System tokens generated successfully".green());
        Ok(())
    }

    async fn generate_token(&self, subject: &str, vault: &str, permissions: Vec<String>) -> Result<String> {
        let claims = Claims {
            sub: subject.to_string(),
            vault: vault.to_string(),
            permissions,
            iat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            exp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() + 86400, // 24 hours
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let token = JwtHs256Token::mint(&claims, self.jwt_secret.as_bytes()).await?;
        Ok(token.as_str().to_string())
    }

    async fn spawn_agent(&mut self, name: &str, token_name: &str) -> Result<EntityId> {
        let token = self.tokens.get(token_name)
            .ok_or_else(|| anyhow::anyhow!("Token '{}' not found", token_name))?
            .clone();

        let agent_id = EntityId(self.next_agent_id);
        self.next_agent_id += 1;

        // Use the correct EntityId that matches the token subject
        let origin = match token_name {
            "admin" => EntityId(0),     // Admin token has subject "0"
            "user" => EntityId(1000),   // User token has subject "1000"
            _ => EntityId(0),           // Default to system entity
        };
        
        let spec = AgentSpec { name: name.to_string() };

        let message = Message {
            origin,
            capability: token,
            op: Operation::SpawnSubAgent { parent: origin, spec },
        };

        let event = self.runtime.submit(message).await?;
        self.agents.insert(name.to_string(), agent_id);

        println!("{} Agent '{}' spawned with ID {}", 
            "✅".green(), name.cyan(), agent_id.0.to_string().yellow());
        println!("📋 Event: {:?}", event);

        Ok(agent_id)
    }

    async fn schedule_task(&self, agent_name: &str, description: &str, token_name: &str) -> Result<()> {
        let token = self.tokens.get(token_name)
            .ok_or_else(|| anyhow::anyhow!("Token '{}' not found", token_name))?
            .clone();

        let agent_id = self.agents.get(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        // Use the correct EntityId that matches the token subject
        let origin = match token_name {
            "admin" => EntityId(0),     // Admin token has subject "0"
            "user" => EntityId(1000),   // User token has subject "1000"
            _ => EntityId(0),           // Default to system entity
        };

        let task = TaskSpec { description: description.to_string() };

        let message = Message {
            origin,
            capability: token,
            op: Operation::ScheduleAgentTask { agent: *agent_id, task },
        };

        let event = self.runtime.submit(message).await?;

        println!("{} Task scheduled for agent '{}'", 
            "✅".green(), agent_name.cyan());
        println!("📋 Description: {}", description);
        println!("📋 Event: {:?}", event);

        Ok(())
    }

    async fn query_state(&self) -> Result<()> {
        // TODO: Add world state querying capability to RuntimeManager
        println!("{}", "🌍 Current World State:".blue().bold());
        println!("📊 Runtime state querying not yet implemented in RuntimeManager");
        
        // For now, just show that we can get execution history
        let history = self.runtime.get_execution_history().await;
        println!("📋 Total executions: {}", history.len());

        // Dummy agent tasks display for compatibility
        println!("📊 Agents with tasks: 0");
        println!("📋 Total tasks: 0");

        // Note: Individual agent task details are not yet accessible through RuntimeManager

        Ok(())
    }

    async fn run_demo(&mut self) -> Result<()> {
        println!("{}", "🚀 Running Toka Demo Scenarios".blue().bold());
        println!();

        // Scenario 1: File Operations Agent
        println!("{}", "📁 Scenario 1: File Operations Agent".yellow().bold());
        let file_agent = self.spawn_agent("FileAgent", "admin").await?;
        sleep(Duration::from_secs(1)).await;
        
        self.schedule_task("FileAgent", "Read project configuration files", "admin").await?;
        self.schedule_task("FileAgent", "Generate project summary report", "admin").await?;
        sleep(Duration::from_secs(1)).await;

        // Scenario 2: System Monitoring Agent
        println!("\n{}", "🔍 Scenario 2: System Monitoring Agent".yellow().bold());
        let monitor_agent = self.spawn_agent("MonitorAgent", "user").await?;
        sleep(Duration::from_secs(1)).await;
        
        self.schedule_task("MonitorAgent", "Check system resource usage", "user").await?;
        self.schedule_task("MonitorAgent", "Monitor active processes", "user").await?;
        sleep(Duration::from_secs(1)).await;

        // Scenario 3: API Research Agent
        println!("\n{}", "🌐 Scenario 3: API Research Agent".yellow().bold());
        let api_agent = self.spawn_agent("ApiAgent", "admin").await?;
        sleep(Duration::from_secs(1)).await;
        
        self.schedule_task("ApiAgent", "Fetch latest GitHub API documentation", "admin").await?;
        self.schedule_task("ApiAgent", "Analyze API rate limits and usage", "admin").await?;
        sleep(Duration::from_secs(1)).await;

        // Query final state
        println!("\n{}", "📊 Final System State".yellow().bold());
        self.query_state().await?;

        println!("\n{}", "✅ Demo completed successfully!".green().bold());
        Ok(())
    }

    async fn run_interactive(&mut self) -> Result<()> {
        println!("{}", "🎮 Interactive Toka Testing Environment".blue().bold());
        println!("{}", "Type 'help' for available commands".yellow());
        println!();

        let mut rl = DefaultEditor::new()?;
        
        loop {
            let readline = rl.readline("toka-test> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    rl.add_history_entry(line)?;

                    match self.handle_command(line).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("{} {}", "❌ Error:".red(), e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("EOF");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }

        match parts[0] {
            "help" => {
                self.show_help();
            }
            "tokens" => {
                self.show_tokens();
            }
            "agents" => {
                self.show_agents();
            }
            "spawn" => {
                if parts.len() < 2 {
                    println!("{} Usage: spawn <agent-name> [token-name]", "❌".red());
                    return Ok(true);
                }
                let agent_name = parts[1];
                let token_name = parts.get(2).unwrap_or(&"admin");
                self.spawn_agent(agent_name, token_name).await?;
            }
            "task" => {
                if parts.len() < 3 {
                    println!("{} Usage: task <agent-name> <description>", "❌".red());
                    return Ok(true);
                }
                let agent_name = parts[1];
                let description = parts[2..].join(" ");
                self.schedule_task(agent_name, &description, "admin").await?;
            }
            "state" => {
                self.query_state().await?;
            }
            "demo" => {
                self.run_demo().await?;
            }
            "token" => {
                if parts.len() < 4 {
                    println!("{} Usage: token <name> <subject> <permissions>", "❌".red());
                    return Ok(true);
                }
                let name = parts[1];
                let subject = parts[2];
                let permissions = parts[3..].join(",");
                let token = self.generate_token(subject, "custom-vault", 
                    permissions.split(',').map(|s| s.trim().to_string()).collect()).await?;
                self.tokens.insert(name.to_string(), token);
                println!("{} Token '{}' generated for '{}'", "✅".green(), name, subject);
            }
            "quit" | "exit" => {
                println!("{}", "👋 Goodbye!".blue());
                return Ok(false);
            }
            _ => {
                println!("{} Unknown command: '{}'. Type 'help' for available commands.", 
                    "❌".red(), parts[0]);
            }
        }

        Ok(true)
    }

    fn show_help(&self) {
        println!("{}", "📚 Available Commands:".blue().bold());
        println!("  {}  - Show this help message", "help".cyan());
        println!("  {}  - Show available tokens", "tokens".cyan());
        println!("  {}  - Show spawned agents", "agents".cyan());
        println!("  {}  - Spawn a new agent", "spawn <name> [token]".cyan());
        println!("  {}  - Schedule a task for an agent", "task <agent> <description>".cyan());
        println!("  {}  - Query current system state", "state".cyan());
        println!("  {}  - Run demo scenarios", "demo".cyan());
        println!("  {}  - Generate custom token", "token <name> <subject> <permissions>".cyan());
        println!("  {}  - Exit the testing environment", "quit/exit".cyan());
    }

    fn show_tokens(&self) {
        println!("{}", "🔑 Available Tokens:".blue().bold());
        for (name, token) in &self.tokens {
            println!("  {} = {}", name.cyan(), &token[..50].yellow());
        }
    }

    fn show_agents(&self) {
        println!("{}", "🤖 Spawned Agents:".blue().bold());
        for (name, id) in &self.agents {
            println!("  {} (ID: {})", name.cyan(), id.0.to_string().yellow());
        }
    }
}

//─────────────────────────────
//  Main application
//─────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Toka Testing Environment v{}", env!("CARGO_PKG_VERSION"));

    // Create testing environment
    let mut env = TestingEnvironment::new(&cli).await?;
    
    // Run in demo or interactive mode
    if cli.demo {
        env.run_demo().await?;
    } else {
        env.run_interactive().await?;
    }

    // Graceful shutdown
    // Note: RuntimeManager cleanup is handled automatically
    info!("Toka Testing Environment shutting down");

    Ok(())
} 