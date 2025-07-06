#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! # Toka Analysis Tools
//!
//! This crate provides secure, sandboxed execution of Python-based code analysis tools
//! for the Toka OS ecosystem. It bridges the gap between the Rust-based Toka system
//! and the Python analysis tools, ensuring security, capability validation, and
//! comprehensive error handling.
//!
//! ## Features
//!
//! - **Control Flow Analysis**: Analyzes function control flow patterns and complexity
//! - **Dependency Analysis**: Visualizes and analyzes crate dependencies and architecture
//! - **Combined Analysis**: Runs multiple analysis tools together for comprehensive reports
//! - **Secure Execution**: All Python tools run in sandboxed environments with resource limits
//! - **Multiple Output Formats**: Supports Mermaid, JSON, HTML, and Markdown outputs
//!
//! ## Security Model
//!
//! All Python tools are executed in secure sandboxes with:
//! - Process isolation using Linux namespaces
//! - Resource limits (CPU, memory, disk, network)
//! - Filesystem access restrictions
//! - Capability-based permission checking
//! - Comprehensive audit logging
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────────┐
//! │                           Toka Analysis Tools                                    │
//! ├─────────────────────────────────────────────────────────────────────────────────┤
//! │  Tool Registry  │  Security Layer  │  Executor  │  Output Processing  │  Cache  │
//! ├─────────────────────────────────────────────────────────────────────────────────┤
//! │                              Secure Python Sandbox                              │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────────┐  │
//! │  │ Control Flow    │  │ Dependency      │  │ Combined Analysis              │  │
//! │  │ Analysis        │  │ Analysis        │  │ (Multiple Tools)               │  │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use toka_analysis_tools::{AnalysisToolRegistry, ControlFlowAnalysisTool};
//! use toka_tools::ToolRegistry;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create tool registry
//!     let registry = ToolRegistry::new().await?;
//!     
//!     // Register analysis tools
//!     let analysis_registry = AnalysisToolRegistry::new().await?;
//!     analysis_registry.register_all_tools(&registry).await?;
//!     
//!     // Use tools through the standard Toka tool interface
//!     let mut args = std::collections::HashMap::new();
//!     args.insert("target_function".into(), "main".into());
//!     args.insert("output_format".into(), "mermaid".into());
//!     
//!     let params = toka_tools::ToolParams {
//!         name: "control-flow-analysis".into(),
//!         args,
//!     };
//!     
//!     let result = registry.execute_tool("control-flow-analysis", &params).await?;
//!     println!("Analysis result: {}", result.output);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Tool Development
//!
//! New analysis tools should implement the `AnalysisTool` trait and provide:
//! - Input validation and sanitization
//! - Secure execution with resource limits
//! - Structured output in multiple formats
//! - Comprehensive error handling
//! - Audit logging for security compliance

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use toka_tools::{Tool, ToolParams, ToolResult, ToolRegistry};
use toka_types::SecurityConfig;

pub mod executor;
pub mod security;
pub mod tools;
pub mod sandbox;
pub mod validation;
pub mod output;
pub mod cache;
pub mod metrics;

// Re-export commonly used types
pub use executor::{PythonExecutor, ExecutionConfig, ExecutionResult};
pub use security::{SecurityValidator, ResourceLimits, SandboxConfig};
pub use tools::{ControlFlowAnalysisTool, DependencyAnalysisTool, CombinedAnalysisTool};
pub use sandbox::{PythonSandbox, SandboxBuilder};
pub use validation::{InputValidator, OutputValidator};
pub use output::{OutputProcessor, OutputFormat};
pub use cache::{ResultCache, CacheConfig};
pub use metrics::{AnalysisMetrics, MetricsCollector};

/// Error types for analysis tools
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    /// Python execution failed
    #[error("Python execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Security validation failed
    #[error("Security validation failed: {0}")]
    SecurityViolation(String),
    
    /// Input validation failed
    #[error("Input validation failed: {0}")]
    InvalidInput(String),
    
    /// Output processing failed
    #[error("Output processing failed: {0}")]
    OutputProcessingFailed(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    /// Sandbox creation failed
    #[error("Sandbox creation failed: {0}")]
    SandboxCreationFailed(String),
    
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Configuration for analysis tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Python interpreter path
    pub python_path: PathBuf,
    /// Analysis tools directory
    pub tools_directory: PathBuf,
    /// Output directory for results
    pub output_directory: PathBuf,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Security configuration
    pub security: SecurityConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Execution timeout
    pub timeout: Duration,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            python_path: PathBuf::from("python3"),
            tools_directory: PathBuf::from("toka_analysis_tools"),
            output_directory: PathBuf::from("target/analysis"),
            workspace_root: PathBuf::from("."),
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec![
                    "filesystem-read".to_string(),
                    "filesystem-write".to_string(),
                    "process-spawn".to_string(),
                ],
                resource_limits: toka_types::ResourceLimits {
                    max_memory: "512MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "10m".to_string(),
                },
            },
            resource_limits: ResourceLimits::default(),
            timeout: Duration::from_secs(600), // 10 minutes
            enable_cache: true,
            cache_config: CacheConfig::default(),
            enable_metrics: true,
        }
    }
}

/// Registry for analysis tools
pub struct AnalysisToolRegistry {
    config: AnalysisConfig,
    executor: Arc<PythonExecutor>,
    cache: Arc<RwLock<ResultCache>>,
    metrics: Arc<MetricsCollector>,
    tools: HashMap<String, Box<dyn AnalysisTool>>,
}

impl AnalysisToolRegistry {
    /// Create a new analysis tool registry
    pub async fn new() -> Result<Self> {
        Self::with_config(AnalysisConfig::default()).await
    }
    
    /// Create a new analysis tool registry with custom configuration
    pub async fn with_config(config: AnalysisConfig) -> Result<Self> {
        info!("Initializing analysis tool registry with config: {:?}", config);
        
        // Create executor
        let executor = Arc::new(PythonExecutor::new(config.clone()).await?);
        
        // Create cache
        let cache = Arc::new(RwLock::new(ResultCache::new(config.cache_config.clone())?));
        
        // Create metrics collector
        let metrics = Arc::new(MetricsCollector::new(config.enable_metrics));
        
        // Create tools
        let mut tools: HashMap<String, Box<dyn AnalysisTool>> = HashMap::new();
        
        // Register built-in tools
        #[cfg(feature = "control-flow-analysis")]
        {
            let tool = ControlFlowAnalysisTool::new(executor.clone(), cache.clone(), metrics.clone());
            tools.insert("control-flow-analysis".to_string(), Box::new(tool));
        }
        
        #[cfg(feature = "dependency-analysis")]
        {
            let tool = DependencyAnalysisTool::new(executor.clone(), cache.clone(), metrics.clone());
            tools.insert("dependency-analysis".to_string(), Box::new(tool));
        }
        
        #[cfg(feature = "combined-analysis")]
        {
            let tool = CombinedAnalysisTool::new(executor.clone(), cache.clone(), metrics.clone());
            tools.insert("combined-analysis".to_string(), Box::new(tool));
        }
        
        info!("Analysis tool registry initialized with {} tools", tools.len());
        
        Ok(Self {
            config,
            executor,
            cache,
            metrics,
            tools,
        })
    }
    
    /// Register all analysis tools with the main tool registry
    pub async fn register_all_tools(&self, registry: &ToolRegistry) -> Result<()> {
        for (name, tool) in &self.tools {
            let tool_wrapper = AnalysisToolWrapper::new(tool.as_ref(), name.clone());
            registry.register_tool(Arc::new(tool_wrapper)).await?;
            info!("Registered analysis tool: {}", name);
        }
        Ok(())
    }
    
    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn AnalysisTool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }
    
    /// List available tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    /// Get metrics
    pub fn get_metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> cache::CacheStats {
        self.cache.read().await.get_stats()
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.write().await.clear().await
    }
}

/// Trait for analysis tools
#[async_trait]
pub trait AnalysisTool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &str;
    
    /// Get tool description
    fn description(&self) -> &str;
    
    /// Get tool version
    fn version(&self) -> &str;
    
    /// Get required capabilities
    fn required_capabilities(&self) -> Vec<String>;
    
    /// Validate input parameters
    fn validate_input(&self, params: &ToolParams) -> Result<()>;
    
    /// Execute the analysis
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;
    
    /// Get output schema
    fn output_schema(&self) -> serde_json::Value;
}

/// Wrapper to adapt AnalysisTool to the standard Tool trait
pub struct AnalysisToolWrapper {
    tool: *const dyn AnalysisTool,
    name: String,
}

impl AnalysisToolWrapper {
    fn new(tool: &dyn AnalysisTool, name: String) -> Self {
        Self {
            tool: tool as *const dyn AnalysisTool,
            name,
        }
    }
}

// Safety: The tool pointer is guaranteed to be valid for the lifetime of the wrapper
unsafe impl Send for AnalysisToolWrapper {}
unsafe impl Sync for AnalysisToolWrapper {}

#[async_trait]
impl Tool for AnalysisToolWrapper {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        // Safety: The tool pointer is guaranteed to be valid
        unsafe { &*self.tool }.description()
    }
    
    fn version(&self) -> &str {
        // Safety: The tool pointer is guaranteed to be valid
        unsafe { &*self.tool }.version()
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        // Safety: The tool pointer is guaranteed to be valid
        unsafe { &*self.tool }.execute(params).await
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        // Safety: The tool pointer is guaranteed to be valid
        unsafe { &*self.tool }.validate_input(params)
    }
}

/// Analysis tool execution context
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Tool name
    pub tool_name: String,
    /// Input parameters
    pub params: ToolParams,
    /// Working directory
    pub working_directory: PathBuf,
    /// Output directory
    pub output_directory: PathBuf,
    /// Execution ID for tracking
    pub execution_id: String,
    /// Security config
    pub security_config: SecurityConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Execution timeout
    pub timeout: Duration,
}

impl AnalysisContext {
    /// Create a new analysis context
    pub fn new(
        tool_name: String,
        params: ToolParams,
        config: &AnalysisConfig,
    ) -> Self {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let output_directory = config.output_directory.join(&execution_id);
        
        Self {
            tool_name,
            params,
            working_directory: config.workspace_root.clone(),
            output_directory,
            execution_id,
            security_config: config.security.clone(),
            resource_limits: config.resource_limits.clone(),
            timeout: config.timeout,
        }
    }
}

/// Result of analysis execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Execution ID
    pub execution_id: String,
    /// Tool name
    pub tool_name: String,
    /// Success status
    pub success: bool,
    /// Output data
    pub output: String,
    /// Output format
    pub format: OutputFormat,
    /// Execution metrics
    pub metrics: AnalysisMetrics,
    /// Error message if failed
    pub error: Option<String>,
    /// Output files generated
    pub output_files: Vec<PathBuf>,
}

impl AnalysisResult {
    /// Create a successful result
    pub fn success(
        execution_id: String,
        tool_name: String,
        output: String,
        format: OutputFormat,
        metrics: AnalysisMetrics,
        output_files: Vec<PathBuf>,
    ) -> Self {
        Self {
            execution_id,
            tool_name,
            success: true,
            output,
            format,
            metrics,
            error: None,
            output_files,
        }
    }
    
    /// Create a failed result
    pub fn failure(
        execution_id: String,
        tool_name: String,
        error: String,
        metrics: AnalysisMetrics,
    ) -> Self {
        Self {
            execution_id,
            tool_name,
            success: false,
            output: String::new(),
            format: OutputFormat::Json,
            metrics,
            error: Some(error),
            output_files: Vec::new(),
        }
    }
}

/// Common constants
pub mod constants {
    use std::time::Duration;
    
    /// Default execution timeout
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);
    
    /// Maximum memory limit
    pub const MAX_MEMORY_MB: u64 = 1024;
    
    /// Maximum CPU percentage
    pub const MAX_CPU_PERCENT: f64 = 80.0;
    
    /// Maximum output size
    pub const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB
    
    /// Maximum number of output files
    pub const MAX_OUTPUT_FILES: usize = 100;
    
    /// Cache TTL
    pub const CACHE_TTL_SECONDS: u64 = 3600; // 1 hour
}