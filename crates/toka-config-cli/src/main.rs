#![forbid(unsafe_code)]

//! **toka-config-cli** – Configuration file management CLI tool.
//!
//! A robust CLI tool for managing configuration files in YAML, JSON, and TOML formats.
//! Provides create, read, update, and delete operations with comprehensive validation
//! and error handling following Rust best practices.

mod cli;
mod config;
mod error;
mod validation;

use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};
use config::ConfigManager;
use error::ConfigError;

/// Main entry point for the configuration management CLI.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with the specified log level
    init_tracing(&cli.log_level.to_string())?;

    info!("Starting Toka Config CLI v{}", env!("CARGO_PKG_VERSION"));

    // Create configuration manager
    let config_manager = ConfigManager::new();

    // Execute the requested command
    let result: anyhow::Result<()> = match cli.command {
        Commands::Create { file, format, content } => {
            config_manager.create_config(&file, format, content).await.map_err(anyhow::Error::from)
        }
        Commands::Read { file } => {
            config_manager.read_config(&file).await.map_err(anyhow::Error::from)
        }
        Commands::Update { file, key, value } => {
            config_manager.update_config(&file, &key, &value).await.map_err(anyhow::Error::from)
        }
        Commands::Delete { file, key } => {
            config_manager.delete_key(&file, &key).await.map_err(anyhow::Error::from)
        }
        Commands::Validate { file } => {
            config_manager.validate_config(&file).await.map_err(anyhow::Error::from)
        }
        Commands::List { directory } => {
            config_manager.list_configs(&directory).await.map_err(anyhow::Error::from)
        }
    };

    // Handle results with appropriate logging and user feedback
    match result {
        Ok(()) => {
            info!("Command executed successfully");
        }
        Err(e) => {
            error!("Command failed: {}", e);
            eprintln!("❌ Error: {}", e);
            
            // Provide additional context for common errors
            if let Some(config_error) = e.downcast_ref::<ConfigError>() {
                match config_error {
                    ConfigError::FileNotFound(path) => {
                        eprintln!("💡 Tip: Check if the file path '{}' is correct", path.display());
                    }
                    ConfigError::InvalidFormat { .. } => {
                        eprintln!("💡 Tip: Supported formats are YAML (.yml, .yaml), JSON (.json), and TOML (.toml)");
                    }
                    ConfigError::ValidationError { .. } => {
                        eprintln!("💡 Tip: Check the configuration syntax and structure");
                    }
                    _ => {}
                }
            }
            
            std::process::exit(1);
        }
    }

    info!("Toka Config CLI shutting down");
    Ok(())
}

/// Initialize tracing with the specified log level.
///
/// Sets up structured logging with environment variable support and
/// appropriate filtering based on the requested verbosity level.
fn init_tracing(log_level: &str) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}