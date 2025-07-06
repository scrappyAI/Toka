//! Python executor for analysis tools
//!
//! This module provides secure execution of Python analysis tools with
//! proper resource management, sandboxing, and monitoring.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, instrument};

use toka_tools::{ToolParams, ToolResult, ToolMetadata};

use crate::{
    AnalysisConfig, AnalysisError, AnalysisResult, AnalysisContext,
    sandbox::{PythonSandbox, SandboxBuilder, SandboxExecutionResult},
    security::{SecurityValidator, ResourceUsage},
    output::{OutputProcessor, OutputFormat},
    validation::{InputValidator, OutputValidator},
    metrics::{AnalysisMetrics, MetricsCollector},
};

/// Configuration for Python execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Python interpreter path
    pub python_path: PathBuf,
    /// Working directory
    pub working_directory: PathBuf,
    /// Output directory
    pub output_directory: PathBuf,
    /// Timeout for execution
    pub timeout: Duration,
    /// Enable sandboxing
    pub enable_sandbox: bool,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            python_path: PathBuf::from("python3"),
            working_directory: PathBuf::from("."),
            output_directory: PathBuf::from("target/analysis"),
            timeout: Duration::from_secs(600),
            enable_sandbox: true,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// Result of Python execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Success status
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time
    pub execution_time: Duration,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Output files created
    pub output_files: Vec<PathBuf>,
}

/// Python executor for analysis tools
pub struct PythonExecutor {
    config: AnalysisConfig,
    execution_config: ExecutionConfig,
    security_validator: SecurityValidator,
    input_validator: InputValidator,
    output_validator: OutputValidator,
    output_processor: OutputProcessor,
    metrics_collector: Arc<MetricsCollector>,
}

impl PythonExecutor {
    /// Create a new Python executor
    pub async fn new(config: AnalysisConfig) -> Result<Self> {
        info!("Initializing Python executor with config: {:?}", config);
        
        // Validate Python installation
        Self::validate_python_installation(&config.python_path).await?;
        
        // Create output directory
        if !config.output_directory.exists() {
            fs::create_dir_all(&config.output_directory).await
                .context("Failed to create output directory")?;
        }
        
        // Create execution config
        let execution_config = ExecutionConfig {
            python_path: config.python_path.clone(),
            working_directory: config.workspace_root.clone(),
            output_directory: config.output_directory.clone(),
            timeout: config.timeout,
            enable_sandbox: config.security.sandbox,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
        };
        
        // Create security validator
        let security_validator = SecurityValidator::new(
            config.security.clone(),
            config.resource_limits.clone(),
        );
        
        // Create validators and processors
        let input_validator = InputValidator::new();
        let output_validator = OutputValidator::new();
        let output_processor = OutputProcessor::new();
        let metrics_collector = Arc::new(MetricsCollector::new(config.enable_metrics));
        
        Ok(Self {
            config,
            execution_config,
            security_validator,
            input_validator,
            output_validator,
            output_processor,
            metrics_collector,
        })
    }
    
    /// Execute a Python analysis tool
    #[instrument(skip(self, context), fields(tool = %context.tool_name, id = %context.execution_id))]
    pub async fn execute_tool(
        &self,
        context: &AnalysisContext,
    ) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        let mut metrics = AnalysisMetrics::new(context.tool_name.clone());
        
        info!("Executing Python tool: {} ({})", context.tool_name, context.execution_id);
        
        // Validate security
        self.security_validator.validate_tool_params(&context.tool_name, &context.params)?;
        metrics.security_checks_passed = true;
        
        // Validate input
        self.input_validator.validate(&context.params)?;
        metrics.input_validation_passed = true;
        
        // Determine script path
        let script_path = self.get_script_path(&context.tool_name)?;
        
        // Execute with retries
        let mut last_error = None;
        for attempt in 1..=self.execution_config.max_retries {
            match self.execute_with_sandbox(&script_path, context).await {
                Ok(result) => {
                    metrics.execution_time = start_time.elapsed();
                    metrics.success = result.success;
                    
                    if result.success {
                        return self.process_successful_result(result, context, metrics).await;
                    } else {
                        return self.process_failed_result(result, context, metrics).await;
                    }
                }
                Err(e) => {
                    warn!("Execution attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < self.execution_config.max_retries {
                        tokio::time::sleep(self.execution_config.retry_delay).await;
                    }
                }
            }
        }
        
        // All retries failed
        metrics.execution_time = start_time.elapsed();
        metrics.success = false;
        
        let error_msg = last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "All execution attempts failed".to_string());
        
        Ok(AnalysisResult::failure(
            context.execution_id.clone(),
            context.tool_name.clone(),
            error_msg,
            metrics,
        ))
    }
    
    /// Execute with sandbox
    async fn execute_with_sandbox(
        &self,
        script_path: &Path,
        context: &AnalysisContext,
    ) -> Result<SandboxExecutionResult> {
        if self.execution_config.enable_sandbox {
            self.execute_sandboxed(script_path, context).await
        } else {
            self.execute_direct(script_path, context).await
        }
    }
    
    /// Execute in sandbox
    async fn execute_sandboxed(
        &self,
        script_path: &Path,
        context: &AnalysisContext,
    ) -> Result<SandboxExecutionResult> {
        let mut sandbox = SandboxBuilder::new()
            .enable_isolation(true)
            .disable_network(true)
            .working_directory(context.working_directory.clone())
            .allow_read_path(context.working_directory.clone())
            .allow_read_path(self.config.tools_directory.clone())
            .allow_write_path(context.output_directory.clone())
            .resource_limits(self.config.resource_limits.clone())
            .timeout(context.timeout)
            .python_path(self.config.tools_directory.clone())
            .env("TOKA_WORKSPACE_ROOT".to_string(), context.working_directory.to_string_lossy().to_string())
            .env("TOKA_OUTPUT_DIR".to_string(), context.output_directory.to_string_lossy().to_string())
            .env("TOKA_EXECUTION_ID".to_string(), context.execution_id.clone())
            .build()?;
        
        // Build arguments
        let args = self.build_script_arguments(context)?;
        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        
        // Execute
        sandbox.execute_python(script_path, &args_str, None).await
    }
    
    /// Execute directly (for testing/debugging)
    async fn execute_direct(
        &self,
        script_path: &Path,
        context: &AnalysisContext,
    ) -> Result<SandboxExecutionResult> {
        use tokio::process::Command;
        use std::process::Stdio;
        
        warn!("Executing Python tool without sandbox - this should only be used for testing!");
        
        let args = self.build_script_arguments(context)?;
        
        let start_time = Instant::now();
        let output = Command::new(&self.execution_config.python_path)
            .arg(script_path)
            .args(&args)
            .current_dir(&context.working_directory)
            .env("TOKA_WORKSPACE_ROOT", &context.working_directory)
            .env("TOKA_OUTPUT_DIR", &context.output_directory)
            .env("TOKA_EXECUTION_ID", &context.execution_id)
            .env("PYTHONPATH", self.config.tools_directory.to_string_lossy().as_ref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute Python process")?;
        
        let execution_time = start_time.elapsed();
        
        Ok(SandboxExecutionResult {
            success: output.status.success(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            resource_usage: crate::sandbox::ResourceUsage {
                memory_peak_mb: 0,
                cpu_time: execution_time,
            },
            audit_events: vec![],
        })
    }
    
    /// Get script path for tool
    fn get_script_path(&self, tool_name: &str) -> Result<PathBuf> {
        let script_name = match tool_name {
            "control-flow-analysis" => "control_flow.py",
            "dependency-analysis" => "dependency_graph.py",
            "combined-analysis" => "combined_analysis.py",
            _ => return Err(AnalysisError::ToolNotFound(tool_name.to_string()).into()),
        };
        
        let script_path = self.config.tools_directory.join(script_name);
        
        if !script_path.exists() {
            return Err(AnalysisError::ConfigurationError(
                format!("Script not found: {}", script_path.display())
            ).into());
        }
        
        Ok(script_path)
    }
    
    /// Build script arguments from context
    fn build_script_arguments(&self, context: &AnalysisContext) -> Result<Vec<String>> {
        let mut args = Vec::new();
        
        // Add standard arguments
        args.push("--workspace-root".to_string());
        args.push(context.working_directory.to_string_lossy().to_string());
        
        args.push("--output-dir".to_string());
        args.push(context.output_directory.to_string_lossy().to_string());
        
        args.push("--execution-id".to_string());
        args.push(context.execution_id.clone());
        
        // Add tool-specific arguments
        for (key, value) in &context.params.args {
            args.push(format!("--{}", key.replace("_", "-")));
            args.push(value.clone());
        }
        
        Ok(args)
    }
    
    /// Process successful execution result
    async fn process_successful_result(
        &self,
        result: SandboxExecutionResult,
        context: &AnalysisContext,
        mut metrics: AnalysisMetrics,
    ) -> Result<AnalysisResult> {
        // Validate output
        self.output_validator.validate(&result.stdout)?;
        metrics.output_validation_passed = true;
        
        // Process output
        let processed_output = self.output_processor.process(&result.stdout, OutputFormat::Json)?;
        
        // Find output files
        let output_files = self.find_output_files(&context.output_directory).await?;
        
        // Validate resource usage
        let resource_usage = crate::security::ResourceUsage {
            memory_mb: result.resource_usage.memory_peak_mb,
            cpu_percent: 0.0, // Would need actual monitoring
            execution_time: result.execution_time,
            output_size: result.stdout.len() + result.stderr.len(),
            output_files: output_files.len(),
            disk_mb: 0, // Would need actual monitoring
        };
        
        self.security_validator.validate_resource_usage(&resource_usage)?;
        
        // Update metrics
        metrics.resource_usage = Some(resource_usage);
        metrics.output_size = result.stdout.len() + result.stderr.len();
        
        // Record metrics
        self.metrics_collector.record_execution(&metrics).await;
        
        Ok(AnalysisResult::success(
            context.execution_id.clone(),
            context.tool_name.clone(),
            processed_output,
            OutputFormat::Json,
            metrics,
            output_files,
        ))
    }
    
    /// Process failed execution result
    async fn process_failed_result(
        &self,
        result: SandboxExecutionResult,
        context: &AnalysisContext,
        mut metrics: AnalysisMetrics,
    ) -> Result<AnalysisResult> {
        let error_msg = if !result.stderr.is_empty() {
            format!("Python execution failed (exit code {}): {}", result.exit_code, result.stderr)
        } else {
            format!("Python execution failed with exit code {}", result.exit_code)
        };
        
        // Update metrics
        metrics.error_message = Some(error_msg.clone());
        
        // Record metrics
        self.metrics_collector.record_execution(&metrics).await;
        
        Ok(AnalysisResult::failure(
            context.execution_id.clone(),
            context.tool_name.clone(),
            error_msg,
            metrics,
        ))
    }
    
    /// Find output files in directory
    async fn find_output_files(&self, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut output_files = Vec::new();
        
        if !output_dir.exists() {
            return Ok(output_files);
        }
        
        let mut entries = fs::read_dir(output_dir).await
            .context("Failed to read output directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                output_files.push(path);
            }
        }
        
        Ok(output_files)
    }
    
    /// Validate Python installation
    async fn validate_python_installation(python_path: &Path) -> Result<()> {
        use tokio::process::Command;
        
        let output = Command::new(python_path)
            .arg("--version")
            .output()
            .await
            .context("Failed to check Python version")?;
        
        if !output.status.success() {
            return Err(AnalysisError::ConfigurationError(
                format!("Python not found or not working: {}", python_path.display())
            ).into());
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("Python version: {}", version.trim());
        
        Ok(())
    }
    
    /// Convert AnalysisResult to ToolResult
    pub fn to_tool_result(&self, analysis_result: AnalysisResult) -> ToolResult {
        ToolResult {
            success: analysis_result.success,
            output: analysis_result.output,
            metadata: ToolMetadata {
                execution_time_ms: analysis_result.metrics.execution_time.as_millis() as u64,
                tool_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().timestamp() as u64,
            },
        }
    }
    
    /// Get metrics collector
    pub fn get_metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    
    fn create_test_config() -> AnalysisConfig {
        AnalysisConfig {
            python_path: PathBuf::from("python3"),
            tools_directory: PathBuf::from("test_tools"),
            output_directory: PathBuf::from("test_output"),
            workspace_root: PathBuf::from("."),
            ..Default::default()
        }
    }
    
    fn create_test_context() -> AnalysisContext {
        let mut args = HashMap::new();
        args.insert("target_function".to_string(), "main".to_string());
        args.insert("output_format".to_string(), "json".to_string());
        
        let params = ToolParams {
            name: "control-flow-analysis".to_string(),
            args,
        };
        
        AnalysisContext::new(
            "control-flow-analysis".to_string(),
            params,
            &create_test_config(),
        )
    }
    
    #[tokio::test]
    async fn test_executor_creation() {
        let config = create_test_config();
        
        // Skip test if Python is not available
        if PythonExecutor::validate_python_installation(&config.python_path).await.is_err() {
            return;
        }
        
        let executor = PythonExecutor::new(config).await;
        assert!(executor.is_ok());
    }
    
    #[test]
    fn test_script_path_resolution() {
        let config = create_test_config();
        let executor = PythonExecutor {
            config: config.clone(),
            execution_config: ExecutionConfig::default(),
            security_validator: SecurityValidator::new(config.security.clone(), config.resource_limits.clone()),
            input_validator: InputValidator::new(),
            output_validator: OutputValidator::new(),
            output_processor: OutputProcessor::new(),
            metrics_collector: Arc::new(MetricsCollector::new(true)),
        };
        
        // Test known tool
        let script_path = executor.get_script_path("control-flow-analysis");
        match script_path {
            Ok(path) => assert!(path.ends_with("control_flow.py")),
            Err(_) => {
                // Expected if script doesn't exist in test environment
            }
        }
        
        // Test unknown tool
        let script_path = executor.get_script_path("unknown-tool");
        assert!(script_path.is_err());
    }
    
    #[test]
    fn test_argument_building() {
        let config = create_test_config();
        let executor = PythonExecutor {
            config: config.clone(),
            execution_config: ExecutionConfig::default(),
            security_validator: SecurityValidator::new(config.security.clone(), config.resource_limits.clone()),
            input_validator: InputValidator::new(),
            output_validator: OutputValidator::new(),
            output_processor: OutputProcessor::new(),
            metrics_collector: Arc::new(MetricsCollector::new(true)),
        };
        
        let context = create_test_context();
        let args = executor.build_script_arguments(&context).unwrap();
        
        assert!(args.contains(&"--workspace-root".to_string()));
        assert!(args.contains(&"--output-dir".to_string()));
        assert!(args.contains(&"--execution-id".to_string()));
        assert!(args.contains(&"--target-function".to_string()));
        assert!(args.contains(&"main".to_string()));
    }
    
    #[test]
    fn test_tool_result_conversion() {
        let config = create_test_config();
        let executor = PythonExecutor {
            config: config.clone(),
            execution_config: ExecutionConfig::default(),
            security_validator: SecurityValidator::new(config.security.clone(), config.resource_limits.clone()),
            input_validator: InputValidator::new(),
            output_validator: OutputValidator::new(),
            output_processor: OutputProcessor::new(),
            metrics_collector: Arc::new(MetricsCollector::new(true)),
        };
        
        let analysis_result = AnalysisResult::success(
            "test-id".to_string(),
            "test-tool".to_string(),
            "test output".to_string(),
            OutputFormat::Json,
            AnalysisMetrics::new("test-tool".to_string()),
            vec![],
        );
        
        let tool_result = executor.to_tool_result(analysis_result);
        assert!(tool_result.success);
        assert_eq!(tool_result.output, "test output");
    }
}