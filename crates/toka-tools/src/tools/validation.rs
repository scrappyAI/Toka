//! Validation tools for ensuring workspace integrity

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

use crate::core::{Tool, ToolResult, ToolParams};

/// Date validation tool that checks for future dates in workspace files
#[derive(Debug, Clone)]
pub struct DateValidator {
    /// Regex patterns for finding dates
    date_patterns: Vec<Regex>,
    /// File extensions to check
    target_extensions: Vec<String>,
    /// Maximum future days allowed
    max_future_days: i64,
}

impl DateValidator {
    /// Create a new date validator with default settings
    pub fn new() -> Result<Self> {
        let date_patterns = vec![
            Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b")?, // YYYY-MM-DD
            Regex::new(r"\b(\d{2})/(\d{2})/(\d{4})\b")?, // MM/DD/YYYY
            Regex::new(r"\b(\d{4})/(\d{2})/(\d{2})\b")?, // YYYY/MM/DD
            Regex::new(r"Generated on: (\d{4}-\d{2}-\d{2})")?, // Generated on: YYYY-MM-DD
            Regex::new(r"Created: (\d{4}-\d{2}-\d{2})")?, // Created: YYYY-MM-DD
            Regex::new(r"Date: (\d{4}-\d{2}-\d{2})")?, // Date: YYYY-MM-DD
        ];
        
        let target_extensions = vec![
            "rs".to_string(),
            "toml".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "md".to_string(),
            "txt".to_string(),
            "py".to_string(),
            "sh".to_string(),
        ];
        
        Ok(Self {
            date_patterns,
            target_extensions,
            max_future_days: 0, // No future dates allowed by default
        })
    }
    
    /// Validate dates in a file or directory
    pub async fn validate_path(&self, path: &Path, fix_violations: bool) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        if path.is_file() {
            self.validate_file(path, fix_violations, &mut result).await?;
        } else if path.is_dir() {
            Box::pin(self.validate_directory(path, fix_violations, &mut result)).await?;
        }
        
        Ok(result)
    }
    
    /// Validate dates in a single file
    async fn validate_file(&self, path: &Path, fix_violations: bool, result: &mut ValidationResult) -> Result<()> {
        // Check if file extension is in target list
        if let Some(ext) = path.extension() {
            if !self.target_extensions.contains(&ext.to_string_lossy().to_lowercase()) {
                return Ok(());
            }
        } else {
            return Ok(());
        }
        
        result.files_checked += 1;
        
        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let mut violations = Vec::new();
        let mut fixed_content = content.clone();
        let now = Utc::now();
        
        // Check each date pattern
        for pattern in &self.date_patterns {
            for capture in pattern.captures_iter(&content) {
                if let Some(date_str) = capture.get(0) {
                    let date_text = date_str.as_str();
                    
                    // Try to parse the date
                    if let Some(parsed_date) = self.parse_date(date_text) {
                        // Check if date is in the future
                        let days_diff = (parsed_date - now).num_days();
                        
                        if days_diff > self.max_future_days {
                            violations.push(DateViolation {
                                file_path: path.to_path_buf(),
                                line_number: self.find_line_number(&content, date_str.start()),
                                date_text: date_text.to_string(),
                                parsed_date,
                                days_in_future: days_diff,
                            });
                            
                            // Fix violation if requested
                            if fix_violations {
                                let current_date = now.format("%Y-%m-%d").to_string();
                                fixed_content = fixed_content.replace(date_text, &current_date);
                            }
                        }
                    }
                }
            }
        }
        
        // Write fixed content back if there were violations and fixes were requested
        if fix_violations && !violations.is_empty() {
            fs::write(path, fixed_content).await
                .with_context(|| format!("Failed to write fixed content to: {}", path.display()))?;
            
            result.fixes_applied += violations.len();
            info!("Fixed {} date violations in {}", violations.len(), path.display());
        }
        
        result.violations.extend(violations);
        
        Ok(())
    }
    
    /// Validate dates in a directory recursively
    async fn validate_directory(&self, dir: &Path, fix_violations: bool, result: &mut ValidationResult) -> Result<()> {
        let mut entries = fs::read_dir(dir).await
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Skip hidden files and directories
            if path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with('.'))
                .unwrap_or(false) {
                continue;
            }
            
            // Skip target and build directories
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(name, "target" | "build" | "dist" | "node_modules" | ".git") {
                        continue;
                    }
                }
            }
            
            if path.is_file() {
                self.validate_file(&path, fix_violations, result).await?;
            } else if path.is_dir() {
                Box::pin(self.validate_directory(&path, fix_violations, result)).await?;
            }
        }
        
        Ok(())
    }
    
    /// Parse a date string into a DateTime<Utc>
    fn parse_date(&self, date_str: &str) -> Option<DateTime<Utc>> {
        // Try different date formats
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }
        
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }
        
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }
        
        None
    }
    
    /// Find the line number for a character position in the content
    fn find_line_number(&self, content: &str, char_pos: usize) -> usize {
        content[..char_pos].chars().filter(|&c| c == '\n').count() + 1
    }
}

#[async_trait]
impl Tool for DateValidator {
    fn name(&self) -> &str {
        "date-validator"
    }
    
    fn description(&self) -> &str {
        "Validates dates in workspace files to prevent future dates"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let path = params.args.get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        
        let fix_violations = params.args.get("fix_violations")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);
        
        let path = PathBuf::from(path);
        
        let result = self.validate_path(&path, fix_violations).await?;
        
        Ok(ToolResult {
            success: result.violations.is_empty(),
            output: serde_json::to_string(&result)?,
            metadata: crate::core::ToolMetadata {
                execution_time_ms: 0, // Will be set by registry
                tool_version: self.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("path") {
            return Err(anyhow::anyhow!("Missing required parameter: path"));
        }
        
        if let Some(path) = params.args.get("path") {
            if path.trim().is_empty() {
                return Err(anyhow::anyhow!("Path parameter cannot be empty"));
            }
        }
        
        Ok(())
    }
}

/// Result of date validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub files_checked: usize,
    pub violations: Vec<DateViolation>,
    pub fixes_applied: usize,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            files_checked: 0,
            violations: Vec::new(),
            fixes_applied: 0,
        }
    }
}

/// A date violation found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateViolation {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub date_text: String,
    pub parsed_date: DateTime<Utc>,
    pub days_in_future: i64,
}

/// Build validator that checks workspace build integrity
#[derive(Debug, Clone)]
pub struct BuildValidator;

impl BuildValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate the build system
    pub async fn validate_build(&self, workspace_path: &Path) -> Result<BuildValidationResult> {
        let mut result = BuildValidationResult::new();
        
        // Check Cargo.toml files
        self.check_cargo_files(workspace_path, &mut result).await?;
        
        // Check dependency consistency
        self.check_dependencies(workspace_path, &mut result).await?;
        
        // Check for common build issues
        self.check_build_issues(workspace_path, &mut result).await?;
        
        Ok(result)
    }
    
    async fn check_cargo_files(&self, workspace_path: &Path, result: &mut BuildValidationResult) -> Result<()> {
        let workspace_cargo = workspace_path.join("Cargo.toml");
        if !workspace_cargo.exists() {
            result.issues.push("Missing workspace Cargo.toml".to_string());
            return Ok(());
        }
        
        // Check workspace members
        let content = fs::read_to_string(&workspace_cargo).await?;
        if !content.contains("[workspace]") {
            result.issues.push("Workspace Cargo.toml missing [workspace] section".to_string());
        }
        
        result.files_checked += 1;
        Ok(())
    }
    
    async fn check_dependencies(&self, workspace_path: &Path, result: &mut BuildValidationResult) -> Result<()> {
        // This is a simplified check - in practice, you'd want to parse TOML and check versions
        let crates_dir = workspace_path.join("crates");
        if !crates_dir.exists() {
            return Ok(());
        }
        
        let mut entries = fs::read_dir(&crates_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.exists() {
                    result.files_checked += 1;
                    // Add dependency checking logic here
                }
            }
        }
        
        Ok(())
    }
    
    async fn check_build_issues(&self, workspace_path: &Path, result: &mut BuildValidationResult) -> Result<()> {
        // Check for target directory in git
        let target_dir = workspace_path.join("target");
        let gitignore = workspace_path.join(".gitignore");
        
        if target_dir.exists() && gitignore.exists() {
            let gitignore_content = fs::read_to_string(&gitignore).await?;
            if !gitignore_content.contains("target/") {
                result.issues.push("target/ directory not in .gitignore".to_string());
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Tool for BuildValidator {
    fn name(&self) -> &str {
        "build-validator"
    }
    
    fn description(&self) -> &str {
        "Validates build system integrity and dependencies"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let workspace_path = params.args.get("workspace_path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'workspace_path' parameter"))?;
        
        let workspace_path = PathBuf::from(workspace_path);
        
        let result = self.validate_build(&workspace_path).await?;
        
        Ok(ToolResult {
            success: result.issues.is_empty(),
            output: serde_json::to_string(&result)?,
            metadata: crate::core::ToolMetadata {
                execution_time_ms: 0, // Will be set by registry
                tool_version: self.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("workspace_path") {
            return Err(anyhow::anyhow!("Missing required parameter: workspace_path"));
        }
        
        if let Some(path) = params.args.get("workspace_path") {
            if path.trim().is_empty() {
                return Err(anyhow::anyhow!("Workspace path parameter cannot be empty"));
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildValidationResult {
    pub files_checked: usize,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

impl BuildValidationResult {
    fn new() -> Self {
        Self {
            files_checked: 0,
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_date_validator_future_date() {
        let validator = DateValidator::new().unwrap();
        
        // Create a temporary file with a future date
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        
        let content = format!("# Test File\n\nGenerated on: {}\n", "2030-01-01");
        fs::write(&test_file, content).await.unwrap();
        
        let result = validator.validate_path(&test_file, false).await.unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(!result.violations.is_empty());
        assert_eq!(result.violations[0].date_text, "2030-01-01");
    }
    
    #[tokio::test]
    async fn test_date_validator_fix_violations() {
        let validator = DateValidator::new().unwrap();
        
        // Create a temporary file with a future date
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        
        let content = "# Test File\n\nGenerated on: 2030-01-01\n";
        fs::write(&test_file, content).await.unwrap();
        
        let result = validator.validate_path(&test_file, true).await.unwrap();
        
        assert_eq!(result.fixes_applied, 1);
        
        // Check that the file was actually fixed
        let fixed_content = fs::read_to_string(&test_file).await.unwrap();
        assert!(!fixed_content.contains("2030-01-01"));
    }
    
    #[tokio::test]
    async fn test_build_validator() {
        let validator = BuildValidator::new();
        
        // Create a temporary workspace
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();
        
        // Create a basic Cargo.toml
        let cargo_toml = workspace_path.join("Cargo.toml");
        fs::write(&cargo_toml, "[workspace]\nmembers = []\n").await.unwrap();
        
        let result = validator.validate_build(workspace_path).await.unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(result.issues.is_empty());
    }
} 