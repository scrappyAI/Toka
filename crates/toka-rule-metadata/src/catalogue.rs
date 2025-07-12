//! Rule catalogue implementation for unified rule management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    RuleMetadata, RuleFormat, RuleCategory, RuleMetadataError,
    calculate_checksum, determine_format, scan_directory
};

/// Central catalogue for managing all rule types in the Toka workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCatalogue {
    /// All rules indexed by name
    rules: HashMap<String, RuleMetadata>,
    /// Rules grouped by format
    formats: HashMap<RuleFormat, Vec<String>>,
    /// Rules grouped by category
    categories: HashMap<RuleCategory, Vec<String>>,
    /// Workspace root path
    workspace_root: PathBuf,
    /// Last scan timestamp
    last_scan: Option<DateTime<Utc>>,
}

impl RuleCatalogue {
    /// Create a new empty rule catalogue.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            formats: HashMap::new(),
            categories: HashMap::new(),
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            last_scan: None,
        }
    }

    /// Create a new rule catalogue with specified workspace root.
    pub fn with_workspace_root(workspace_root: PathBuf) -> Self {
        Self {
            rules: HashMap::new(),
            formats: HashMap::new(),
            categories: HashMap::new(),
            workspace_root,
            last_scan: None,
        }
    }

    /// Scan the entire workspace for rules.
    pub async fn scan_workspace(&mut self) -> Result<()> {
        info!("Scanning workspace for rules: {}", self.workspace_root.display());
        
        // Clear existing data
        self.rules.clear();
        self.formats.clear();
        self.categories.clear();
        
        // Scan different rule locations
        let rule_files = self.discover_rule_files().await?;
        
        info!("Found {} rule files to process", rule_files.len());
        
        // Process each rule file
        for file_path in rule_files {
            match self.process_rule_file(&file_path).await {
                Ok(metadata) => {
                    self.add_rule(metadata);
                }
                Err(e) => {
                    warn!("Failed to process rule file {}: {}", file_path.display(), e);
                }
            }
        }
        
        self.last_scan = Some(Utc::now());
        info!("Scan complete: {} rules catalogued", self.rules.len());
        
        Ok(())
    }

    /// Discover all rule files in the workspace.
    async fn discover_rule_files(&self) -> Result<Vec<PathBuf>> {
        let mut rule_files = Vec::new();
        
        // Scan .cursor/rules directory
        let cursor_rules_path = self.workspace_root.join(".cursor/rules");
        if cursor_rules_path.exists() {
            rule_files.extend(scan_directory(&cursor_rules_path)?);
        }
        
        // Scan agents-specs directory
        let agent_specs_path = self.workspace_root.join("agents-specs");
        if agent_specs_path.exists() {
            rule_files.extend(scan_directory(&agent_specs_path)?);
        }
        
        // Scan config directory
        let config_path = self.workspace_root.join("config");
        if config_path.exists() {
            rule_files.extend(scan_directory(&config_path)?);
        }
        
        Ok(rule_files)
    }

    /// Process a single rule file and extract metadata.
    async fn process_rule_file(&self, file_path: &Path) -> Result<RuleMetadata> {
        let content = fs::read(file_path)?;
        let checksum = calculate_checksum(&content);
        
        let format = determine_format(file_path)
            .ok_or_else(|| RuleMetadataError::Validation(
                format!("Unable to determine format for {}", file_path.display())
            ))?;
        
        let metadata = match format {
            RuleFormat::CursorYaml => self.parse_cursor_yaml(&content, file_path)?,
            RuleFormat::LegacyMdc => self.parse_legacy_mdc(&content, file_path)?,
            RuleFormat::AgentSpec => self.parse_agent_spec(&content, file_path)?,
            RuleFormat::AgentConfig => self.parse_agent_config(&content, file_path)?,
        };
        
        let file_metadata = fs::metadata(file_path)?;
        let last_modified = DateTime::from_timestamp(
            file_metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64,
            0
        ).unwrap_or_else(|| Utc::now());
        
        Ok(RuleMetadata {
            checksum,
            format,
            last_modified,
            file_path: file_path.strip_prefix(&self.workspace_root)
                .unwrap_or(file_path)
                .to_path_buf(),
            ..metadata
        })
    }

    /// Parse Cursor YAML rule format.
    fn parse_cursor_yaml(&self, content: &[u8], file_path: &Path) -> Result<RuleMetadata> {
        let yaml_content: serde_yaml::Value = serde_yaml::from_slice(content)?;
        
        let name = yaml_content.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| file_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        
        let version = yaml_content.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        
        let category = yaml_content.get("category")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "core" => RuleCategory::Core,
                "security" => RuleCategory::Security,
                "documentation" => RuleCategory::Documentation,
                "testing" => RuleCategory::Testing,
                "development" => RuleCategory::Development,
                "architecture" => RuleCategory::Architecture,
                _ => RuleCategory::Other,
            })
            .unwrap_or(RuleCategory::Other);
        
        let priority = yaml_content.get("priority")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as u8;
        
        let always_apply = yaml_content.get("always_apply")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let description = yaml_content.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let dependencies = yaml_content.get("extends")
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
        
        let mut metadata = HashMap::new();
        if let Some(objectives) = yaml_content.get("objectives") {
            metadata.insert("objectives".to_string(), objectives.clone());
        }
        if let Some(guidelines) = yaml_content.get("guidelines") {
            metadata.insert("guidelines".to_string(), guidelines.clone());
        }
        
        Ok(RuleMetadata {
            name,
            version,
            format: RuleFormat::CursorYaml,
            category,
            priority,
            file_path: PathBuf::new(),
            checksum: String::new(),
            dependencies,
            last_modified: Utc::now(),
            description,
            always_apply,
            metadata,
        })
    }

    /// Parse Legacy MDC rule format.
    fn parse_legacy_mdc(&self, content: &[u8], file_path: &Path) -> Result<RuleMetadata> {
        let content_str = String::from_utf8_lossy(content);
        
        // Extract frontmatter
        let mut name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut description = None;
        let mut always_apply = false;
        
        if content_str.starts_with("---") {
            if let Some(end_pos) = content_str[3..].find("---") {
                let frontmatter = &content_str[3..end_pos + 3];
                if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    if let Some(desc) = yaml.get("description").and_then(|v| v.as_str()) {
                        description = Some(desc.to_string());
                    }
                    if let Some(apply) = yaml.get("alwaysApply").and_then(|v| v.as_bool()) {
                        always_apply = apply;
                    }
                }
            }
        }
        
        // Extract ProjectRule name if present
        if let Some(start) = content_str.find("<ProjectRule name=\"") {
            let start_pos = start + 18;
            if let Some(end) = content_str[start_pos..].find("\"") {
                name = content_str[start_pos..start_pos + end].to_string();
            }
        }
        
        Ok(RuleMetadata {
            name,
            version: "1.0.0".to_string(),
            format: RuleFormat::LegacyMdc,
            category: RuleCategory::Development,
            priority: 50,
            file_path: PathBuf::new(),
            checksum: String::new(),
            dependencies: Vec::new(),
            last_modified: Utc::now(),
            description,
            always_apply,
            metadata: HashMap::new(),
        })
    }

    /// Parse Agent specification format.
    fn parse_agent_spec(&self, content: &[u8], file_path: &Path) -> Result<RuleMetadata> {
        let yaml_content: serde_yaml::Value = serde_yaml::from_slice(content)?;
        
        let name = yaml_content.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| file_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        
        let version = yaml_content.get("metadata")
            .and_then(|m| m.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        
        let description = yaml_content.get("spec")
            .and_then(|s| s.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let priority = yaml_content.get("spec")
            .and_then(|s| s.get("priority"))
            .and_then(|v| v.as_str())
            .map(|p| match p {
                "critical" => 100,
                "high" => 80,
                "medium" => 50,
                "low" => 20,
                _ => 50,
            })
            .unwrap_or(50);
        
        let mut metadata = HashMap::new();
        if let Some(capabilities) = yaml_content.get("capabilities") {
            metadata.insert("capabilities".to_string(), capabilities.clone());
        }
        if let Some(objectives) = yaml_content.get("objectives") {
            metadata.insert("objectives".to_string(), objectives.clone());
        }
        
        Ok(RuleMetadata {
            name,
            version,
            format: RuleFormat::AgentSpec,
            category: RuleCategory::Agent,
            priority,
            file_path: PathBuf::new(),
            checksum: String::new(),
            dependencies: Vec::new(),
            last_modified: Utc::now(),
            description,
            always_apply: false,
            metadata,
        })
    }

    /// Parse Agent configuration format.
    fn parse_agent_config(&self, content: &[u8], file_path: &Path) -> Result<RuleMetadata> {
        let toml_content: toml::Value = toml::from_slice(content)?;
        
        let name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
        
        let mut metadata = HashMap::new();
        if let Some(orchestration) = toml_content.get("orchestration") {
            metadata.insert("orchestration".to_string(), 
                serde_json::to_value(orchestration)?);
        }
        if let Some(agents) = toml_content.get("agents") {
            metadata.insert("agents".to_string(), 
                serde_json::to_value(agents)?);
        }
        
        Ok(RuleMetadata {
            name,
            version: "1.0.0".to_string(),
            format: RuleFormat::AgentConfig,
            category: RuleCategory::Agent,
            priority: 70,
            file_path: PathBuf::new(),
            checksum: String::new(),
            dependencies: Vec::new(),
            last_modified: Utc::now(),
            description: Some("Agent configuration".to_string()),
            always_apply: true,
            metadata,
        })
    }

    /// Add a rule to the catalogue.
    fn add_rule(&mut self, metadata: RuleMetadata) {
        let name = metadata.name.clone();
        let format = metadata.format.clone();
        let category = metadata.category.clone();
        
        // Add to main rules map
        self.rules.insert(name.clone(), metadata);
        
        // Add to format index
        self.formats.entry(format).or_insert_with(Vec::new).push(name.clone());
        
        // Add to category index
        self.categories.entry(category).or_insert_with(Vec::new).push(name);
    }

    /// Get a rule by name.
    pub fn get_rule(&self, name: &str) -> Option<&RuleMetadata> {
        self.rules.get(name)
    }

    /// Get all rules of a specific format.
    pub fn get_rules_by_format(&self, format: &RuleFormat) -> Vec<&RuleMetadata> {
        self.formats.get(format)
            .map(|names| names.iter()
                .filter_map(|name| self.rules.get(name))
                .collect())
            .unwrap_or_default()
    }

    /// Get all rules of a specific category.
    pub fn get_rules_by_category(&self, category: &RuleCategory) -> Vec<&RuleMetadata> {
        self.categories.get(category)
            .map(|names| names.iter()
                .filter_map(|name| self.rules.get(name))
                .collect())
            .unwrap_or_default()
    }

    /// Get total rule count.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get format distribution.
    pub fn format_distribution(&self) -> HashMap<RuleFormat, usize> {
        self.formats.iter()
            .map(|(format, names)| (format.clone(), names.len()))
            .collect()
    }

    /// Get category distribution.
    pub fn category_distribution(&self) -> HashMap<RuleCategory, usize> {
        self.categories.iter()
            .map(|(category, names)| (category.clone(), names.len()))
            .collect()
    }

    /// Export catalogue to JSON.
    pub fn export_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Import catalogue from JSON.
    pub fn import_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

impl Default for RuleCatalogue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_scan_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create test directory structure
        fs::create_dir_all(base_path.join(".cursor/rules")).unwrap();
        fs::create_dir_all(base_path.join("agents-specs/v0.3")).unwrap();
        fs::create_dir_all(base_path.join("config")).unwrap();
        
        // Create test files
        fs::write(
            base_path.join(".cursor/rules/test.yaml"),
            r#"
name: "TestRule"
version: "1.0.0"
category: "core"
priority: 100
always_apply: true
description: "Test rule"
            "#
        ).unwrap();
        
        fs::write(
            base_path.join("agents-specs/v0.3/test-agent.yaml"),
            r#"
metadata:
  name: "test-agent"
  version: "v0.3.0"
spec:
  name: "Test Agent"
  priority: "high"
            "#
        ).unwrap();
        
        fs::write(
            base_path.join("config/test.toml"),
            r#"
[orchestration]
max_concurrent_agents = 5

[[agents]]
name = "test-agent"
            "#
        ).unwrap();
        
        let mut catalogue = RuleCatalogue::with_workspace_root(base_path.to_path_buf());
        catalogue.scan_workspace().await.unwrap();
        
        assert_eq!(catalogue.rule_count(), 3);
        assert!(catalogue.get_rule("TestRule").is_some());
        assert!(catalogue.get_rule("test-agent").is_some());
        assert!(catalogue.get_rule("test").is_some());
    }
} 