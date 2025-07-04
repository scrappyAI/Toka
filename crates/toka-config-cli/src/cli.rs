//! CLI command definitions and argument parsing.
//!
//! This module defines the command-line interface structure using Clap derive macros,
//! providing a clean and type-safe way to handle user input and command routing.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Configuration file management CLI tool.
#[derive(Parser)]
#[command(name = "toka-config")]
#[command(about = "Configuration file management CLI - supports YAML, JSON, and TOML")]
#[command(version)]
pub struct Cli {
    /// Log level for the application
    #[arg(long, default_value = "info")]
    #[arg(value_enum)]
    pub log_level: LogLevel,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available log levels for the application.
#[derive(Clone, ValueEnum)]
pub enum LogLevel {
    /// Trace level logging (most verbose)
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging (default)
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging (least verbose)
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// Supported configuration file formats.
#[derive(Clone, ValueEnum)]
pub enum ConfigFormat {
    /// YAML format (.yml, .yaml)
    Yaml,
    /// JSON format (.json)
    Json,
    /// TOML format (.toml)
    Toml,
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigFormat::Yaml => write!(f, "yaml"),
            ConfigFormat::Json => write!(f, "json"),
            ConfigFormat::Toml => write!(f, "toml"),
        }
    }
}

/// Available commands for configuration management.
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new configuration file
    Create {
        /// Path to the configuration file to create
        #[arg(short, long)]
        file: PathBuf,

        /// Format of the configuration file
        #[arg(short = 't', long)]
        #[arg(value_enum)]
        format: ConfigFormat,

        /// Initial content for the configuration file (JSON string)
        #[arg(short, long, default_value = "{}")]
        content: String,
    },
    /// Read and display a configuration file
    Read {
        /// Path to the configuration file to read
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Update a key-value pair in a configuration file
    Update {
        /// Path to the configuration file to update
        #[arg(short, long)]
        file: PathBuf,

        /// Key to update (supports dot notation for nested keys)
        #[arg(short, long)]
        key: String,

        /// New value for the key (JSON format)
        #[arg(short, long)]
        value: String,
    },
    /// Delete a key from a configuration file
    Delete {
        /// Path to the configuration file
        #[arg(short, long)]
        file: PathBuf,

        /// Key to delete (supports dot notation for nested keys)
        #[arg(short, long)]
        key: String,
    },
    /// Validate the syntax and structure of a configuration file
    Validate {
        /// Path to the configuration file to validate
        #[arg(short, long)]
        file: PathBuf,
    },
    /// List all configuration files in a directory
    List {
        /// Directory to search for configuration files
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify that the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_config_format_display() {
        assert_eq!(ConfigFormat::Yaml.to_string(), "yaml");
        assert_eq!(ConfigFormat::Json.to_string(), "json");
        assert_eq!(ConfigFormat::Toml.to_string(), "toml");
    }
}