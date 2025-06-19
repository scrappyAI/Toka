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

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(
    name = "toka",
    version = env!("CARGO_PKG_VERSION"),
    author = "Toka Contributors <opensource@toka.io>",
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Agent { sub } => handle_agent(sub).await?,
        Commands::Tool { sub } => handle_tool(sub).await?,
        Commands::Vault { sub } => handle_vault(sub).await?,
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
            println!("[TODO] Running tool '{}' with payload: {}", tool_id, payload);
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