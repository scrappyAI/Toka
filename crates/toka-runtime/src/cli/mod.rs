// CLI module 

mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the runtime daemon
    Start {
        /// Path to the runtime configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// Run a tool
    Run {
        /// Name of the tool to run
        tool: String,
        
        /// Tool arguments as key=value pairs
        #[arg(short, long, value_delimiter = ',')]
        args: Option<Vec<String>>,
    },
    
    /// List available tools
    Tools,
    
    /// Manage agents
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    
    /// Emit events into the system
    Event {
        /// Type of event to emit
        #[arg(short, long)]
        event_type: String,
        
        /// Event data (JSON format)
        #[arg(short, long)]
        data: String,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// Create a new agent
    Create {
        /// Type of agent to create
        #[arg(short, long)]
        agent_type: String,
        
        /// Configuration for the agent (JSON format)
        #[arg(short, long)]
        config: Option<String>,
    },
    
    /// List all registered agents
    List,
    
    /// Control an agent's execution
    Control {
        /// ID of the agent to control
        #[arg(short, long)]
        agent_id: String,
        
        /// Command to send to the agent (pause/resume/set-goals)
        #[arg(short, long)]
        command: String,
        
        /// Additional command data (JSON format)
        #[arg(short, long)]
        data: Option<String>,
    },
}

/// Parse command line arguments
pub fn parse_args() -> Result<Cli> {
    Ok(Cli::parse())
}

/// Execute the CLI command
pub async fn execute_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Start { config } => {
            commands::start_runtime(config).await
        }
        Commands::Run { tool, args } => {
            commands::run_tool(tool, args).await
        }
        Commands::Tools => {
            commands::list_tools().await
        }
        Commands::Agent { command } => {
            match command {
                AgentCommands::Create { agent_type, config } => {
                    commands::create_agent(agent_type, config).await
                }
                AgentCommands::List => {
                    commands::list_agents().await
                }
                AgentCommands::Control { agent_id, command, data } => {
                    commands::control_agent(agent_id, command, data).await
                }
            }
        }
        Commands::Event { event_type, data } => {
            commands::emit_event(event_type, data).await
        }
    }
} 