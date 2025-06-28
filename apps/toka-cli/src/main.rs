//! Toka CLI – command-line interface skeleton.
//!
//! Phase-4 deliverable: provides a minimal CLI frontend to interact with
//! agents, tools, and the vault.  Each command prints a placeholder for now
//! and will be wired up to `toka-runtime` functionality in later phases.
//!
//! Usage examples (once installed):
//!   $ toka agent list
//!   $ toka agent new "super-agent"
//!   $ toka tool run echo --payload "Hello"
//!   $ toka vault get user/session
//!
//! Build & run in debug mode:
//!   cargo run -p toka-cli -- agent list
//!
//! The CLI uses `clap` derive macros and an async `main` entry-point so we can
//! later await runtime operations.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "toka",
    version = env!("CARGO_PKG_VERSION"),
    author = "Toka Contributors <opensource@toka.sh>",
    about = "Toka – unified CLI for agents, tools & vaults",
    propagate_version = true,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Agent management operations
    Agent {
        #[command(subcommand)]
        sub: AgentCmd,
    },
    /// Tool registry operations
    Tool {
        #[command(subcommand)]
        sub: ToolCmd,
    },
    /// Secure vault interactions
    Vault {
        #[command(subcommand)]
        sub: VaultCmd,
    },
    /// Authentication & security utilities
    Auth {
        #[command(subcommand)]
        sub: AuthCmd,
    },
    /// Launch an interactive playground (REPL) wired to a temporary runtime
    Playground,
    /// Manifest utilities
    Manifest {
        #[command(subcommand)]
        sub: ManifestCmd,
    },
}

#[derive(Subcommand)]
enum AgentCmd {
    /// Create a new agent with the given name
    New {
        /// Human-readable agent name
        name: String,
    },
    /// List all registered agents
    List,
    /// Observe an agent's event stream (follow mode)
    Observe {
        /// Agent identifier (UUID or name)
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum ToolCmd {
    /// List all available tools
    List,
    /// Run a tool with a JSON payload
    Run {
        /// Tool identifier
        tool_id: String,
        /// JSON-encoded parameters passed verbatim to the tool
        #[arg(long)]
        payload: String,
    },
}

#[derive(Subcommand)]
enum VaultCmd {
    /// Get a value from the vault
    Get {
        /// Key path to retrieve (e.g. "user/session")
        key: String,
    },
    /// Put a value into the vault
    Put {
        /// Key path to store data at
        key: String,
        /// JSON-encoded value to store under the key
        #[arg(long)]
        value: String,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Rotate the active capability-token secret immediately.
    RotateSecret,
}

#[derive(Subcommand)]
enum ManifestCmd {
    /// Validate a Tool manifest JSON/YAML file
    Lint {
        /// Path to manifest file (JSON)
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Agent { sub } => handle_agent(sub).await?,
        Commands::Tool { sub } => handle_tool(sub).await?,
        Commands::Vault { sub } => handle_vault(sub).await?,
        Commands::Auth { sub } => handle_auth(sub).await?,
        Commands::Playground => run_playground().await?,
        Commands::Manifest { sub } => handle_manifest(sub).await?,
    }

    Ok(())
}

async fn handle_agent(cmd: AgentCmd) -> Result<()> {
    match cmd {
        AgentCmd::New { name } => {
            println!("[TODO] Creating agent: {}", name);
        }
        AgentCmd::List => {
            println!("[TODO] Listing agents…");
        }
        AgentCmd::Observe { agent_id } => {
            println!("[TODO] Observing events for agent: {}", agent_id);
        }
    }
    Ok(())
}

async fn handle_tool(cmd: ToolCmd) -> Result<()> {
    match cmd {
        ToolCmd::List => {
            println!("[TODO] Listing tools…");
        }
        ToolCmd::Run { tool_id, payload } => {
            println!(
                "[TODO] Running tool '{}' with payload: {}",
                tool_id, payload
            );
        }
    }
    Ok(())
}

async fn handle_vault(cmd: VaultCmd) -> Result<()> {
    match cmd {
        VaultCmd::Get { key } => {
            println!("[TODO] Vault GET for key: {}", key);
        }
        VaultCmd::Put { key, value } => {
            println!("[TODO] Vault PUT for key: {} with value: {}", key, value);
        }
    }
    Ok(())
}

async fn handle_auth(cmd: AuthCmd) -> Result<()> {
    match cmd {
        AuthCmd::RotateSecret => {
            use toka_runtime::runtime::{Runtime, RuntimeConfig};
            let rt = Runtime::new(RuntimeConfig::default()).await?;
            rt.rotate_secrets();
            println!("✅ Secret rotated successfully");
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
//  Interactive playground implementation
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};
use tracing::Level;

use async_trait::async_trait;

use toka_runtime::runtime::{Runtime, RuntimeConfig};
use toka_runtime::tools::{Tool, ToolParams, ToolResult, ToolMetadata};

use toka_agents::Agent;

/// Simple system tool that echoes the provided "message" argument verbatim.
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes the provided 'message' argument. Useful for smoke-testing tool plumbing."
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn execute(&self, params: &ToolParams) -> anyhow::Result<ToolResult> {
        let msg = params
            .args
            .get("message")
            .cloned()
            .unwrap_or_default();

        Ok(ToolResult {
            success: true,
            output: msg.clone(),
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, _params: &ToolParams) -> anyhow::Result<()> {
        // No strict validation – echo happily accepts empty input.
        Ok(())
    }
}

/// Minimal agent that prints any `user_input` events to STDOUT.
struct PrinterAgent {
    id: String,
}

impl PrinterAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for PrinterAgent {
    fn name(&self) -> &str {
        &self.id
    }

    async fn process_event(&mut self, event_type: &str, event_data: &str) -> anyhow::Result<()> {
        if event_type == "user_input" {
            println!("📨 Agent '{}' received: {}", self.id, event_data);
        }
        Ok(())
    }

    async fn save_state(
        &self,
        _adapter: &dyn toka_agents::MemoryAdapter,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn load_state(
        &mut self,
        _adapter: &dyn toka_agents::MemoryAdapter,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Starts an interactive REPL backed by a fresh in-memory runtime.
async fn run_playground() -> anyhow::Result<()> {
    // Pretty‐print tracing events to STDOUT so the user can observe behaviour.
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .try_init();

    // 1️⃣  Boot the runtime with default config.
    let runtime = Runtime::new(RuntimeConfig::default()).await?;
    runtime.start().await?;

    // 2️⃣  Register a couple of basic system tools.
    runtime
        .tool_registry()
        .register_tool(Arc::new(EchoTool))
        .await?;

    // 3️⃣  Create & register a trivial agent that prints user input.
    runtime
        .register_agent(Box::new(PrinterAgent::new("printer")))
        .await?;

    println!("🟢 Playground ready. Type messages, or 'exit' / 'quit' to leave.");

    let mut lines = io::BufReader::new(io::stdin()).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            break;
        }

        // Emit as a runtime event so agents can react.
        runtime
            .emit_event("user_input".to_string(), line.to_string())
            .await?;

        // Example: also demonstrate the echo tool by invoking it directly.
        let params = ToolParams {
            name: "echo".to_string(),
            args: std::iter::once(("message".to_string(), line.to_string()))
                .collect(),
        };

        let result = runtime
            .tool_registry()
            .execute_tool("echo", &params)
            .await?;

        println!("🛠️  Tool result: {}", result.output);
    }

    runtime.stop().await?;
    println!("👋 Playground terminated.");

    Ok(())
}

async fn handle_manifest(cmd: ManifestCmd) -> Result<()> {
    match cmd {
        ManifestCmd::Lint { path } => {
            use std::fs;
            use toka_toolkit_core::manifest::ToolManifest;

            let data = fs::read_to_string(&path)?;
            let manifest: ToolManifest = serde_json::from_str(&data)?;
            manifest.validate()?;
            println!("✅ Manifest valid: {} (capability: {})", manifest.name, manifest.capability);
        }
    }
    Ok(())
}
