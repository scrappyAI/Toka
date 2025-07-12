//! Tool Catalogue - Unified metadata catalogue for tool discovery and filtering
//!
//! This module provides a comprehensive tool catalogue system that enables efficient
//! tool discovery, filtering, and metadata management. It supports multiple formats
//! (JSON, YAML, MD, MDC) and is designed for future .proto schema compatibility.
//!
//! ## Features
//!
//! - **Unified Metadata**: Single source of truth for all tool metadata
//! - **Multi-format Support**: JSON, YAML, MD, MDC with .proto roadmap
//! - **Dynamic Filtering**: Efficient filtering by capabilities, categories, security levels
//! - **Hot-swappable**: Runtime tool registration and updates
//! - **LLM Integration**: Optimized for LLM-to-LLM and human UI consumption
//!
//! ## Usage
//!
//! ```rust
//! use toka_tools::catalogue::{ToolCatalogue, ToolMetadata, ToolFilter};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut catalogue = ToolCatalogue::new();
//!     
//!     // Scan workspace for tools
//!     catalogue.scan_workspace().await?;
//!     
//!     // Filter tools by capabilities
//!     let validation_tools = catalogue.filter_tools(ToolFilter::new()
//!         .with_capability("validation")
//!         .with_security_level("medium"))?;
//!     
//!     // Export for LLM consumption
//!     let json_export = catalogue.export_json()?;
//!     
//!     Ok(())
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::manifest::{ProtocolMapping, SideEffect, Transport};
use crate::wrappers::{SecurityLevel, ResourceLimits, SandboxConfig};

/// Unified tool metadata structure for catalogue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Unique tool identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Tool category for classification
    pub category: ToolCategory,
    /// Required capabilities for execution
    pub required_capabilities: Vec<String>,
    /// Optional capabilities
    pub optional_capabilities: Vec<String>,
    /// Security classification
    pub security_level: SecurityLevel,
    /// Resource limits for execution
    pub resource_limits: ResourceLimits,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
    /// Side effects classification
    pub side_effects: SideEffect,
    /// Transport options
    pub transports: Vec<Transport>,
    /// Protocol mappings for external integration
    pub protocol_mappings: Vec<ProtocolMapping>,
    /// Input parameters schema (JSON Schema)
    pub input_schema: Option<String>,
    /// Output schema (JSON Schema)
    pub output_schema: Option<String>,
    /// Execution metadata
    pub execution_metadata: ExecutionMetadata,
    /// Discovery metadata
    pub discovery_metadata: DiscoveryMetadata,
    /// Arbitrary extension metadata
    pub extensions: HashMap<String, serde_json::Value>,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// File path relative to workspace
    pub file_path: PathBuf,
    /// Content checksum for change detection
    pub checksum: String,
}

/// Tool categories for classification and filtering
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// File system operations
    FileSystem,
    /// Validation and testing
    Validation,
    /// Build and compilation
    Build,
    /// Security and authentication
    Security,
    /// Network operations
    Network,
    /// Database operations
    Database,
    /// Code analysis and processing
    Analysis,
    /// System administration
    System,
    /// Development utilities
    Development,
    /// Monitoring and observability
    Monitoring,
    /// Documentation and reporting
    Documentation,
    /// Workflow and orchestration
    Workflow,
    /// External integrations
    Integration,
    /// Other/uncategorized
    Other,
}

/// Execution metadata for tool performance and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Whether tool is hot-swappable
    pub hot_swappable: bool,
    /// Whether tool can run in parallel
    pub parallel_safe: bool,
    /// Whether tool is resource intensive
    pub resource_intensive: bool,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: Option<u64>,
    /// Success rate percentage
    pub success_rate: Option<f64>,
    /// Last execution timestamp
    pub last_executed: Option<DateTime<Utc>>,
    /// Execution count
    pub execution_count: u64,
}

/// Discovery metadata for tool registration and updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata {
    /// Whether tool is auto-discovered
    pub auto_discover: bool,
    /// Discovery patterns for file matching
    pub discovery_patterns: Vec<String>,
    /// Discovery priority (higher = discovered first)
    pub discovery_priority: u8,
    /// Whether tool is deprecated
    pub deprecated: bool,
    /// Replacement tool ID if deprecated
    pub replacement_tool: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Tool filter for efficient querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFilter {
    /// Filter by tool categories
    pub categories: Option<Vec<ToolCategory>>,
    /// Filter by required capabilities
    pub required_capabilities: Option<Vec<String>>,
    /// Filter by security levels
    pub security_levels: Option<Vec<SecurityLevel>>,
    /// Filter by transport types
    pub transport_types: Option<Vec<String>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by execution metadata
    pub execution_metadata: Option<ExecutionFilter>,
    /// Search in name and description
    pub search_query: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Whether to include deprecated tools
    pub include_deprecated: bool,
}

/// Execution-specific filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFilter {
    /// Filter by hot-swappable tools
    pub hot_swappable: Option<bool>,
    /// Filter by parallel-safe tools
    pub parallel_safe: Option<bool>,
    /// Filter by resource-intensive tools
    pub resource_intensive: Option<bool>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
    /// Minimum success rate
    pub min_success_rate: Option<f64>,
}

impl ToolFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            categories: None,
            required_capabilities: None,
            security_levels: None,
            transport_types: None,
            tags: None,
            execution_metadata: None,
            search_query: None,
            limit: None,
            include_deprecated: false,
        }
    }

    /// Filter by categories
    pub fn with_categories(mut self, categories: Vec<ToolCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Filter by category
    pub fn with_category(mut self, category: ToolCategory) -> Self {
        self.categories = Some(vec![category]);
        self
    }

    /// Filter by required capabilities
    pub fn with_required_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.required_capabilities = Some(capabilities);
        self
    }

    /// Filter by capability
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.required_capabilities = Some(vec![capability.to_string()]);
        self
    }

    /// Filter by security levels
    pub fn with_security_levels(mut self, levels: Vec<SecurityLevel>) -> Self {
        self.security_levels = Some(levels);
        self
    }

    /// Filter by security level
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_levels = Some(vec![level]);
        self
    }

    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Filter by tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags = Some(vec![tag.to_string()]);
        self
    }

    /// Search in name and description
    pub fn with_search(mut self, query: &str) -> Self {
        self.search_query = Some(query.to_string());
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include deprecated tools
    pub fn include_deprecated(mut self) -> Self {
        self.include_deprecated = true;
        self
    }
}

impl Default for ToolFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Central tool catalogue for unified tool management
#[derive(Debug)]
pub struct ToolCatalogue {
    /// All tools indexed by ID
    tools: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    /// Tools indexed by category
    categories: Arc<RwLock<HashMap<ToolCategory, HashSet<String>>>>,
    /// Tools indexed by capability
    capabilities: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Tools indexed by tag
    tags: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Workspace root path
    workspace_root: PathBuf,
    /// Last scan timestamp
    last_scan: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl ToolCatalogue {
    /// Create a new tool catalogue
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            last_scan: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new tool catalogue with specified workspace root
    pub fn with_workspace_root(workspace_root: PathBuf) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            workspace_root,
            last_scan: Arc::new(RwLock::new(None)),
        }
    }

    /// Scan the workspace for tools and build the catalogue
    pub async fn scan_workspace(&self) -> Result<usize> {
        info!("Scanning workspace for tools: {}", self.workspace_root.display());
        
        // Clear existing data
        {
            let mut tools = self.tools.write().await;
            let mut categories = self.categories.write().await;
            let mut capabilities = self.capabilities.write().await;
            let mut tags = self.tags.write().await;
            
            tools.clear();
            categories.clear();
            capabilities.clear();
            tags.clear();
        }
        
        // Discover tool manifests
        let manifest_files = self.discover_manifest_files().await?;
        info!("Found {} manifest files to process", manifest_files.len());
        
        let mut processed_count = 0;
        
        // Process each manifest file
        for file_path in manifest_files {
            match self.process_manifest_file(&file_path).await {
                Ok(metadata) => {
                    self.add_tool(metadata).await;
                    processed_count += 1;
                }
                Err(e) => {
                    warn!("Failed to process manifest file {}: {}", file_path.display(), e);
                }
            }
        }
        
        // Update last scan timestamp
        {
            let mut last_scan = self.last_scan.write().await;
            *last_scan = Some(Utc::now());
        }
        
        info!("Scan complete: {} tools catalogued", processed_count);
        Ok(processed_count)
    }

    /// Discover all tool manifest files in the workspace
    async fn discover_manifest_files(&self) -> Result<Vec<PathBuf>> {
        let mut manifest_files = Vec::new();
        
        // Scan toka-tools manifests directory
        let manifests_path = self.workspace_root.join("crates/toka-tools/manifests");
        if manifests_path.exists() {
            manifest_files.extend(self.scan_directory(&manifests_path, &["*.yaml", "*.yml", "*.json"])?);
        }
        
        // Scan scripts directory for tool definitions
        let scripts_path = self.workspace_root.join("scripts");
        if scripts_path.exists() {
            manifest_files.extend(self.scan_directory(&scripts_path, &["*.yaml", "*.yml", "*.json"])?);
        }
        
        // Scan config directory for tool configurations
        let config_path = self.workspace_root.join("config");
        if config_path.exists() {
            manifest_files.extend(self.scan_directory(&config_path, &["*.yaml", "*.yml", "*.json"])?);
        }
        
        Ok(manifest_files)
    }

    /// Scan directory for files matching patterns
    fn scan_directory(&self, dir: &Path, patterns: &[&str]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if !dir.exists() || !dir.is_dir() {
            return Ok(files);
        }
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                for pattern in patterns {
                    if file_name.ends_with(&pattern[1..]) { // Remove * from pattern
                        files.push(path);
                        break;
                    }
                }
            }
        }
        
        Ok(files)
    }

    /// Process a single manifest file and extract metadata
    async fn process_manifest_file(&self, file_path: &Path) -> Result<ToolMetadata> {
        let content = std::fs::read(file_path)?;
        let checksum = self.calculate_checksum(&content);
        
        // Parse manifest based on file extension
        let metadata = if file_path.extension().map(|s| s.to_str().unwrap()) == Some("json") {
            self.parse_json_manifest(&content, file_path)?
        } else {
            self.parse_yaml_manifest(&content, file_path)?
        };
        
        let file_metadata = std::fs::metadata(file_path)?;
        let last_modified = DateTime::from_timestamp(
            file_metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64,
            0
        ).unwrap_or_else(|| Utc::now());
        
        Ok(ToolMetadata {
            checksum,
            last_modified,
            file_path: file_path.strip_prefix(&self.workspace_root)
                .unwrap_or(file_path)
                .to_path_buf(),
            ..metadata
        })
    }

    /// Parse YAML manifest format
    fn parse_yaml_manifest(&self, content: &[u8], file_path: &Path) -> Result<ToolMetadata> {
        let yaml_content: serde_yaml::Value = serde_yaml::from_slice(content)?;
        
        let metadata = yaml_content.get("metadata")
            .ok_or_else(|| anyhow::anyhow!("Missing metadata section"))?;
        
        let spec = yaml_content.get("spec")
            .ok_or_else(|| anyhow::anyhow!("Missing spec section"))?;
        
        let interface = yaml_content.get("interface")
            .ok_or_else(|| anyhow::anyhow!("Missing interface section"))?;
        
        let protocols = yaml_content.get("protocols")
            .map(|p| serde_yaml::from_value(p.clone()))
            .transpose()?
            .unwrap_or_default();
        
        let id = metadata.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| file_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        
        let name = metadata.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        
        let version = metadata.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        
        let description = metadata.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let category = metadata.get("category")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "filesystem" => ToolCategory::FileSystem,
                "validation" => ToolCategory::Validation,
                "build" => ToolCategory::Build,
                "security" => ToolCategory::Security,
                "network" => ToolCategory::Network,
                "database" => ToolCategory::Database,
                "analysis" => ToolCategory::Analysis,
                "system" => ToolCategory::System,
                "development" => ToolCategory::Development,
                "monitoring" => ToolCategory::Monitoring,
                "documentation" => ToolCategory::Documentation,
                "workflow" => ToolCategory::Workflow,
                "integration" => ToolCategory::Integration,
                _ => ToolCategory::Other,
            })
            .unwrap_or(ToolCategory::Other);
        
        let capabilities = spec.get("capabilities")
            .and_then(|c| c.get("required"))
            .and_then(|r| r.as_sequence())
            .map(|seq| seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
        
        let optional_capabilities = spec.get("capabilities")
            .and_then(|c| c.get("optional"))
            .and_then(|o| o.as_sequence())
            .map(|seq| seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
        
        let security_level = spec.get("security")
            .and_then(|s| s.get("level"))
            .and_then(|l| l.as_str())
            .map(|s| match s {
                "basic" => SecurityLevel::Basic,
                "medium" => SecurityLevel::Medium,
                "high" => SecurityLevel::High,
                _ => SecurityLevel::Medium,
            })
            .unwrap_or(SecurityLevel::Medium);
        
        let sandbox = spec.get("security")
            .and_then(|s| s.get("sandbox"))
            .unwrap_or(&serde_yaml::Value::Null);
        
        let resource_limits = ResourceLimits {
            max_memory_mb: sandbox.get("memory_limit")
                .and_then(|v| v.as_str())
                .and_then(|s| s.replace("MB", "").parse::<u64>().ok())
                .unwrap_or(128),
            max_cpu_percent: sandbox.get("cpu_limit")
                .and_then(|v| v.as_str())
                .and_then(|s| s.replace("%", "").parse::<f64>().ok())
                .unwrap_or(25.0),
            max_execution_time: std::time::Duration::from_secs(
                sandbox.get("timeout")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.replace("s", "").parse::<u64>().ok())
                    .unwrap_or(30)
            ),
            max_output_size: 1024 * 1024, // 1MB default
            max_output_files: 10,
            max_disk_mb: 64,
        };
        
        let sandbox_config = SandboxConfig {
            use_namespaces: false,
            allow_network: sandbox.get("allow_network")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            readonly_paths: sandbox.get("readonly_paths")
                .and_then(|v| v.as_sequence())
                .map(|seq| seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| std::path::PathBuf::from(s))
                    .collect())
                .unwrap_or_default(),
            writable_paths: sandbox.get("writable_paths")
                .and_then(|v| v.as_sequence())
                .map(|seq| seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| std::path::PathBuf::from(s))
                    .collect())
                .unwrap_or_default(),
            forbidden_paths: vec![],
            allowed_syscalls: vec![],
            env_whitelist: vec!["PATH".to_string()],
            disable_ptrace: false,
            disable_core_dumps: false,
        };
        
        let execution = interface.get("execution")
            .unwrap_or(&serde_yaml::Value::Null);
        
        let execution_metadata = ExecutionMetadata {
            hot_swappable: execution.get("hot_swappable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            parallel_safe: execution.get("parallel_safe")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            resource_intensive: execution.get("resource_intensive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            avg_execution_time_ms: None,
            success_rate: None,
            last_executed: None,
            execution_count: 0,
        };
        
        let discovery = interface.get("discovery")
            .unwrap_or(&serde_yaml::Value::Null);
        
        let discovery_metadata = DiscoveryMetadata {
            auto_discover: discovery.get("auto_discover")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            discovery_patterns: discovery.get("patterns")
                .and_then(|v| v.as_sequence())
                .map(|seq| seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            discovery_priority: 50,
            deprecated: false,
            replacement_tool: None,
            tags: vec![],
        };
        
        Ok(ToolMetadata {
            id,
            name,
            version,
            description,
            category,
            required_capabilities: capabilities,
            optional_capabilities,
            security_level,
            resource_limits,
            sandbox_config,
            side_effects: SideEffect::ReadOnly, // Default to read-only
            transports: vec![Transport::InProcess], // Default to in-process
            protocol_mappings: protocols,
            input_schema: None,
            output_schema: None,
            execution_metadata,
            discovery_metadata,
            extensions: HashMap::new(),
            last_modified: Utc::now(),
            file_path: PathBuf::new(),
            checksum: String::new(),
        })
    }

    /// Parse JSON manifest format
    fn parse_json_manifest(&self, _content: &[u8], _file_path: &Path) -> Result<ToolMetadata> {
        // TODO: Implement JSON manifest parsing
        // For now, return a placeholder
        Err(anyhow::anyhow!("JSON manifest parsing not yet implemented"))
    }

    /// Calculate SHA256 checksum of content
    fn calculate_checksum(&self, content: &[u8]) -> String {
        // Simple hash for now - in production, use proper crypto library
        let mut hash = 0u64;
        for (i, &byte) in content.iter().enumerate() {
            hash = hash.wrapping_add((byte as u64).wrapping_mul(i as u64 + 1));
        }
        format!("{:x}", hash)
    }

    /// Add a tool to the catalogue
    async fn add_tool(&self, metadata: ToolMetadata) {
        let id = metadata.id.clone();
        let category = metadata.category.clone();
        let capabilities = metadata.required_capabilities.clone();
        let tags = metadata.discovery_metadata.tags.clone();
        
        // Add to main tools map
        {
            let mut tools = self.tools.write().await;
            tools.insert(id.clone(), metadata);
        }
        
        // Add to category index
        {
            let mut categories = self.categories.write().await;
            categories.entry(category).or_insert_with(HashSet::new).insert(id.clone());
        }
        
        // Add to capability index
        {
            let mut capabilities_map = self.capabilities.write().await;
            for capability in capabilities {
                capabilities_map.entry(capability).or_insert_with(HashSet::new).insert(id.clone());
            }
        }
        
        // Add to tag index
        {
            let mut tags_map = self.tags.write().await;
            for tag in tags {
                tags_map.entry(tag).or_insert_with(HashSet::new).insert(id.clone());
            }
        }
    }

    /// Get a tool by ID
    pub async fn get_tool(&self, id: &str) -> Option<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.get(id).cloned()
    }

    /// Get all tools
    pub async fn get_all_tools(&self) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Filter tools based on criteria
    pub async fn filter_tools(&self, filter: ToolFilter) -> Result<Vec<ToolMetadata>> {
        let tools = self.tools.read().await;
        let mut filtered_tools = Vec::new();
        
        for tool in tools.values() {
            if self.matches_filter(tool, &filter).await {
                filtered_tools.push(tool.clone());
            }
        }
        
        // Apply limit if specified
        if let Some(limit) = filter.limit {
            filtered_tools.truncate(limit);
        }
        
        Ok(filtered_tools)
    }

    /// Check if a tool matches the filter criteria
    async fn matches_filter(&self, tool: &ToolMetadata, filter: &ToolFilter) -> bool {
        // Check categories
        if let Some(ref categories) = filter.categories {
            if !categories.contains(&tool.category) {
                return false;
            }
        }
        
        // Check required capabilities
        if let Some(ref capabilities) = filter.required_capabilities {
            for capability in capabilities {
                if !tool.required_capabilities.contains(capability) {
                    return false;
                }
            }
        }
        
        // Check security levels
        if let Some(ref levels) = filter.security_levels {
            if !levels.contains(&tool.security_level) {
                return false;
            }
        }
        
        // Check tags
        if let Some(ref tags) = filter.tags {
            for tag in tags {
                if !tool.discovery_metadata.tags.contains(tag) {
                    return false;
                }
            }
        }
        
        // Check search query
        if let Some(ref query) = filter.search_query {
            let search_text = format!("{} {}", tool.name, tool.description).to_lowercase();
            if !search_text.contains(&query.to_lowercase()) {
                return false;
            }
        }
        
        // Check deprecated status
        if !filter.include_deprecated && tool.discovery_metadata.deprecated {
            return false;
        }
        
        // Check execution metadata
        if let Some(ref exec_filter) = filter.execution_metadata {
            if let Some(hot_swappable) = exec_filter.hot_swappable {
                if tool.execution_metadata.hot_swappable != hot_swappable {
                    return false;
                }
            }
            
            if let Some(parallel_safe) = exec_filter.parallel_safe {
                if tool.execution_metadata.parallel_safe != parallel_safe {
                    return false;
                }
            }
            
            if let Some(resource_intensive) = exec_filter.resource_intensive {
                if tool.execution_metadata.resource_intensive != resource_intensive {
                    return false;
                }
            }
            
            if let Some(max_time) = exec_filter.max_execution_time_ms {
                if let Some(avg_time) = tool.execution_metadata.avg_execution_time_ms {
                    if avg_time > max_time {
                        return false;
                    }
                }
            }
            
            if let Some(min_rate) = exec_filter.min_success_rate {
                if let Some(success_rate) = tool.execution_metadata.success_rate {
                    if success_rate < min_rate {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: &ToolCategory) -> Vec<ToolMetadata> {
        let categories = self.categories.read().await;
        let tools = self.tools.read().await;
        
        categories.get(category)
            .map(|ids| ids.iter()
                .filter_map(|id| tools.get(id))
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Get tools by capability
    pub async fn get_tools_by_capability(&self, capability: &str) -> Vec<ToolMetadata> {
        let capabilities = self.capabilities.read().await;
        let tools = self.tools.read().await;
        
        capabilities.get(capability)
            .map(|ids| ids.iter()
                .filter_map(|id| tools.get(id))
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Get tools by tag
    pub async fn get_tools_by_tag(&self, tag: &str) -> Vec<ToolMetadata> {
        let tags = self.tags.read().await;
        let tools = self.tools.read().await;
        
        tags.get(tag)
            .map(|ids| ids.iter()
                .filter_map(|id| tools.get(id))
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        self.tools.read().await.len()
    }

    /// Export catalogue to JSON for LLM consumption
    pub async fn export_json(&self) -> Result<String> {
        let tools = self.get_all_tools().await;
        let export_data = CatalogueExport {
            version: "1.0.0".to_string(),
            generated_at: Utc::now(),
            tool_count: tools.len(),
            tools,
        };
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Export catalogue to YAML
    pub async fn export_yaml(&self) -> Result<String> {
        let tools = self.get_all_tools().await;
        let export_data = CatalogueExport {
            version: "1.0.0".to_string(),
            generated_at: Utc::now(),
            tool_count: tools.len(),
            tools,
        };
        
        Ok(serde_yaml::to_string(&export_data)?)
    }

    /// Get catalogue statistics
    pub async fn get_statistics(&self) -> CatalogueStatistics {
        let tools = self.tools.read().await;
        let categories = self.categories.read().await;
        let capabilities = self.capabilities.read().await;
        let tags = self.tags.read().await;
        
        let mut category_distribution = HashMap::new();
        for (category, ids) in categories.iter() {
            category_distribution.insert(category.clone(), ids.len());
        }
        
        let mut capability_distribution = HashMap::new();
        for (capability, ids) in capabilities.iter() {
            capability_distribution.insert(capability.clone(), ids.len());
        }
        
        let mut tag_distribution = HashMap::new();
        for (tag, ids) in tags.iter() {
            tag_distribution.insert(tag.clone(), ids.len());
        }
        
        CatalogueStatistics {
            total_tools: tools.len(),
            category_distribution,
            capability_distribution,
            tag_distribution,
            last_scan: *self.last_scan.read().await,
        }
    }
}

/// Export format for catalogue data
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogueExport {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub tool_count: usize,
    pub tools: Vec<ToolMetadata>,
}

/// Catalogue statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogueStatistics {
    pub total_tools: usize,
    pub category_distribution: HashMap<ToolCategory, usize>,
    pub capability_distribution: HashMap<String, usize>,
    pub tag_distribution: HashMap<String, usize>,
    pub last_scan: Option<DateTime<Utc>>,
}

impl Default for ToolCatalogue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_catalogue_creation() {
        let catalogue = ToolCatalogue::new();
        assert_eq!(catalogue.tool_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_tool_filtering() {
        let catalogue = ToolCatalogue::new();
        
        // Create a test tool
        let test_tool = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool for validation".to_string(),
            category: ToolCategory::Validation,
            required_capabilities: vec!["validation".to_string()],
            optional_capabilities: vec![],
            security_level: SecurityLevel::Medium,
            resource_limits: ResourceLimits {
                memory_limit: "128MB".to_string(),
                cpu_limit: "25%".to_string(),
                timeout: "30s".to_string(),
            },
            sandbox_config: SandboxConfig {
                allow_network: false,
                readonly_paths: vec![],
                writable_paths: vec![],
            },
            side_effects: SideEffect::ReadOnly,
            transports: vec![Transport::InProcess],
            protocol_mappings: vec![],
            input_schema: None,
            output_schema: None,
            execution_metadata: ExecutionMetadata {
                hot_swappable: true,
                parallel_safe: true,
                resource_intensive: false,
                avg_execution_time_ms: None,
                success_rate: None,
                last_executed: None,
                execution_count: 0,
            },
            discovery_metadata: DiscoveryMetadata {
                auto_discover: true,
                discovery_patterns: vec![],
                discovery_priority: 50,
                deprecated: false,
                replacement_tool: None,
                tags: vec!["test".to_string()],
            },
            extensions: HashMap::new(),
            last_modified: Utc::now(),
            file_path: PathBuf::from("test.yaml"),
            checksum: "test".to_string(),
        };
        
        catalogue.add_tool(test_tool).await;
        
        // Test filtering by category
        let validation_tools = catalogue.filter_tools(
            ToolFilter::new().with_category(ToolCategory::Validation)
        ).await.unwrap();
        assert_eq!(validation_tools.len(), 1);
        
        // Test filtering by capability
        let validation_tools = catalogue.filter_tools(
            ToolFilter::new().with_capability("validation")
        ).await.unwrap();
        assert_eq!(validation_tools.len(), 1);
        
        // Test filtering by tag
        let test_tools = catalogue.filter_tools(
            ToolFilter::new().with_tag("test")
        ).await.unwrap();
        assert_eq!(test_tools.len(), 1);
    }
} 