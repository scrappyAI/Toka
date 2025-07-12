#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-rule-metadata** – Unified rule metadata catalogue and management system.
//!
//! This crate provides a centralized system for managing the different rule formats
//! used throughout the Toka workspace:
//!
//! - **Cursor Rules** (`.cursor/rules/*.yaml`)
//! - **Legacy Rules** (`.cursor/rules/legacy/*.mdc`)
//! - **Agent Specifications** (`agents-specs/**/*.yaml`)
//! - **Agent Configuration** (`config/*.toml`)
//!
//! ## Features
//!
//! - **Unified Metadata**: Single source of truth for all rule formats
//! - **Consistency Validation**: Detect conflicts and inconsistencies
//! - **Dependency Tracking**: Understand rule relationships
//! - **Version Management**: Track rule evolution over time
//! - **CLI Tools**: Command-line interface for rule management
//!
//! ## Usage
//!
//! ```rust,no_run
//! use toka_rule_metadata::{RuleCatalogue, RuleFormat};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut catalogue = RuleCatalogue::new();
//!     
//!     // Scan entire workspace for rules
//!     catalogue.scan_workspace().await?;
//!     
//!     // Validate consistency
//!     let report = catalogue.validate_consistency()?;
//!     println!("Found {} rules with {} issues", 
//!              catalogue.rule_count(), 
//!              report.issues.len());
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

pub mod catalogue;
pub mod formats;
pub mod validation;
pub mod cli;

pub use catalogue::RuleCatalogue;
pub use formats::*;
pub use validation::*;

/// Unified metadata for all rule types in the Toka workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleMetadata {
    /// Unique rule identifier
    pub name: String,
    /// Rule version
    pub version: String,
    /// Rule format type
    pub format: RuleFormat,
    /// Rule category
    pub category: RuleCategory,
    /// Priority level (higher = more important)
    pub priority: u8,
    /// File path relative to workspace root
    pub file_path: PathBuf,
    /// SHA256 checksum of rule content
    pub checksum: String,
    /// Rule dependencies
    pub dependencies: Vec<String>,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// Rule description
    pub description: Option<String>,
    /// Whether rule is always applied
    pub always_apply: bool,
    /// Rule-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Rule format types supported by the catalogue system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleFormat {
    /// Cursor YAML rules (`.cursor/rules/*.yaml`)
    CursorYaml,
    /// Legacy MDC rules (`.cursor/rules/legacy/*.mdc`)
    LegacyMdc,
    /// Agent specifications (`agents-specs/**/*.yaml`)
    AgentSpec,
    /// Agent configuration (`config/*.toml`)
    AgentConfig,
}

/// Rule category classifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Core system rules
    Core,
    /// Security-related rules
    Security,
    /// Documentation rules
    Documentation,
    /// Testing rules
    Testing,
    /// Development process rules
    Development,
    /// Architecture rules
    Architecture,
    /// Agent-specific rules
    Agent,
    /// Integration rules
    Integration,
    /// Performance rules
    Performance,
    /// Other/uncategorized
    Other,
}

/// Error types for rule metadata operations.
#[derive(Debug, thiserror::Error)]
pub enum RuleMetadataError {
    /// File system operation error
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlParsing(#[from] serde_yaml::Error),
    
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParsing(#[from] toml::de::Error),
    
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    
    /// Rule validation error
    #[error("Rule validation error: {0}")]
    Validation(String),
    
    /// Dependency resolution error
    #[error("Dependency resolution error: {0}")]
    DependencyResolution(String),
    
    /// Checksum mismatch error
    #[error("Checksum mismatch for rule {rule_name}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        rule_name: String,
        expected: String,
        actual: String,
    },
}

/// Calculate SHA256 checksum for file content.
pub fn calculate_checksum(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Determine rule format from file path.
pub fn determine_format(path: &Path) -> Option<RuleFormat> {
    let path_str = path.to_string_lossy();
    
    if path_str.contains(".cursor/rules/") && path_str.ends_with(".yaml") {
        Some(RuleFormat::CursorYaml)
    } else if path_str.contains(".cursor/rules/legacy/") && path_str.ends_with(".mdc") {
        Some(RuleFormat::LegacyMdc)
    } else if path_str.contains("agents-specs/") && path_str.ends_with(".yaml") {
        Some(RuleFormat::AgentSpec)
    } else if path_str.contains("config/") && path_str.ends_with(".toml") {
        Some(RuleFormat::AgentConfig)
    } else {
        None
    }
}

/// Scan directory for rule files.
pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut rule_files = Vec::new();
    
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        if determine_format(path).is_some() {
            rule_files.push(path.to_path_buf());
        }
    }
    
    debug!("Found {} rule files in {}", rule_files.len(), dir.display());
    Ok(rule_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_determine_format() {
        assert_eq!(
            determine_format(Path::new(".cursor/rules/core.yaml")),
            Some(RuleFormat::CursorYaml)
        );
        
        assert_eq!(
            determine_format(Path::new(".cursor/rules/legacy/arch.mdc")),
            Some(RuleFormat::LegacyMdc)
        );
        
        assert_eq!(
            determine_format(Path::new("agents-specs/v0.3/test.yaml")),
            Some(RuleFormat::AgentSpec)
        );
        
        assert_eq!(
            determine_format(Path::new("config/agents.toml")),
            Some(RuleFormat::AgentConfig)
        );
        
        assert_eq!(
            determine_format(Path::new("random/file.txt")),
            None
        );
    }

    #[test]
    fn test_calculate_checksum() {
        let content = b"test content";
        let checksum = calculate_checksum(content);
        
        // SHA256 of "test content"
        assert_eq!(checksum, "1eebdf4fdc9fc7bf283031b93f9aef3338de9052f584b10f7ea7f0b7b4c6e8c3");
    }

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create test directory structure
        fs::create_dir_all(base_path.join(".cursor/rules")).unwrap();
        fs::create_dir_all(base_path.join(".cursor/rules/legacy")).unwrap();
        fs::create_dir_all(base_path.join("agents-specs/v0.3")).unwrap();
        fs::create_dir_all(base_path.join("config")).unwrap();
        
        // Create test files
        fs::write(base_path.join(".cursor/rules/test.yaml"), "test: content").unwrap();
        fs::write(base_path.join(".cursor/rules/legacy/test.mdc"), "# Test").unwrap();
        fs::write(base_path.join("agents-specs/v0.3/test.yaml"), "spec: test").unwrap();
        fs::write(base_path.join("config/test.toml"), "[test]").unwrap();
        fs::write(base_path.join("random.txt"), "ignored").unwrap();
        
        let rule_files = scan_directory(base_path).unwrap();
        assert_eq!(rule_files.len(), 4);
    }
} 