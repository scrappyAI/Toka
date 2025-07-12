//! Toka Runtime - Unified Execution System with Kernel Enforcement
//!
//! This crate provides the unified runtime layer for all execution types in Toka OS:
//! - Dynamic code generation and execution
//! - Agent workflow execution and orchestration
//! - Tool execution and management
//! - Security enforcement through kernel integration
//!
//! # Architecture
//!
//! The unified runtime consists of:
//!
//! - **ExecutionModel**: Unified enum handling different execution types
//! - **RuntimeManager**: Central coordinator for all execution
//! - **Execution Engines**: Different runtime environments (WASM, Python, Agent, etc.)
//! - **Sandboxing**: Isolated execution environments with resource limits
//! - **Security Integration**: Full kernel enforcement for all operations
//! - **Resource Management**: Memory, CPU, and I/O tracking per execution
//!
//! # Usage
//!
//! ```rust
//! use toka_runtime::{RuntimeManager, ExecutionRequest, ExecutionModel};
//! use toka_kernel::{ToolKernel, SecurityLevel};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize kernel and runtime
//!     let kernel = ToolKernel::new().await?;
//!     let runtime = RuntimeManager::new(kernel).await?;
//!     
//!     // Execute Python code dynamically
//!     let request = ExecutionRequest {
//!         model: ExecutionModel::DynamicCode {
//!             code_type: CodeType::Python,
//!             code: "print('Hello from dynamic execution!')".to_string(),
//!         },
//!         session_id: "user_session".to_string(),
//!         security_level: SecurityLevel::Sandboxed,
//!         inputs: serde_json::json!({}),
//!     };
//!     
//!     let result = runtime.execute(request).await?;
//!     println!("Execution result: {}", result.output);
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// Re-export kernel types
pub use toka_kernel::{Kernel, KernelError};

// Import toka-types for Message handling
use toka_types::{Message, Operation, AgentConfig, TaskConfig, EntityId, TaskSpec};

// TODO: Create these module files when implementing the engines
// pub mod engines;
// pub mod sandbox;
// pub mod generation;
// pub mod validation;

// TODO: These types need to be implemented in toka-kernel or defined here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Capability {
    CodeGeneration,
    FileSystem,
    Network,
    Process,
    AgentSpawning,
    TaskExecution,
    LlmAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: Vec<Capability>,
}

impl CapabilitySet {
    pub fn with_capabilities(capabilities: Vec<Capability>) -> Self {
        Self { capabilities }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub session_id: String,
    pub security_level: SecurityLevel,
    pub capabilities: CapabilitySet,
}

/// **Phase 1 Implementation: Unified Execution Model**
/// 
/// This enum consolidates all execution types into a single model,
/// eliminating the duplication between toka-runtime and toka-agent-runtime.
/// 
/// Generated: 2025-07-12 (date-enforcement compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionModel {
    /// Dynamic code execution (Python, JS, WASM, etc.)
    DynamicCode {
        code_type: CodeType,
        code: String,
    },
    /// Agent workflow execution
    AgentWorkflow {
        agent_config: AgentConfig,
        agent_id: EntityId,
    },
    /// Tool execution
    ToolExecution {
        tool_name: String,
        tool_args: JsonValue,
    },
}

// TODO: Wrapper struct to add methods needed by runtime
pub struct RuntimeKernel {
    kernel: Kernel,
}

impl RuntimeKernel {
    pub fn new(kernel: Kernel) -> Self {
        Self { kernel }
    }
    
    /// Create execution context (placeholder implementation)
    pub async fn create_execution_context(
        &self,
        _runtime_type: &str,
        session_id: &str,
        capabilities: &CapabilitySet,
        security_level: SecurityLevel,
    ) -> Result<ExecutionContext> {
        Ok(ExecutionContext {
            session_id: session_id.to_string(),
            security_level,
            capabilities: capabilities.clone(),
        })
    }
    
    /// Enforce execution (placeholder implementation)
    pub async fn enforce_execution<F, T>(&self, _context: &ExecutionContext, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        f.await
    }
}

// Type alias for now - will need to be updated when proper types are available
pub type ToolKernel = RuntimeKernel;

/// **Phase 1: Unified Runtime Execution Request**
/// 
/// Consolidates all execution request types into a single structure
/// that can handle dynamic code, agent workflows, and tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unified execution model
    pub model: ExecutionModel,
    /// Session identifier for capability checking
    pub session_id: String,
    /// Security level for execution
    pub security_level: SecurityLevel,
    /// Input data for the execution
    pub inputs: JsonValue,
    /// Optional timeout override
    pub timeout_override: Option<Duration>,
    /// Environment variables
    pub environment: Option<HashMap<String, String>>,
}

/// Supported code execution types
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum CodeType {
    /// Python script execution
    Python,
    /// JavaScript execution (Node.js)
    JavaScript,
    /// WebAssembly module
    WebAssembly,
    /// Bash script execution
    Shell,
    /// Rust code compilation and execution
    Rust,
}

/// **Phase 1: Unified Runtime Execution Result**
/// 
/// Consolidates all execution result types to support both
/// dynamic code execution and agent workflow results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Standard output from execution
    pub output: String,
    /// Standard error from execution
    pub error: String,
    /// Exit code (for process-based execution)
    pub exit_code: Option<i32>,
    /// Execution metadata
    pub metadata: RuntimeMetadata,
    /// Generated artifacts (compiled binaries, etc.)
    pub artifacts: Vec<Artifact>,
    /// Agent-specific results (if applicable)
    pub agent_results: Option<AgentExecutionResult>,
}

/// **Phase 1: Agent Execution Results**
/// 
/// Specific results for agent workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    /// Agent ID that executed
    pub agent_id: EntityId,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
    /// Agent final state
    pub final_state: String,
    /// Execution metrics
    pub metrics: AgentMetrics,
}

/// **Phase 1: Agent Metrics**
/// 
/// Metrics collected during agent execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total tasks attempted
    pub tasks_attempted: u64,
    /// Tasks completed successfully
    pub tasks_completed: u64,
    /// Tasks that failed
    pub tasks_failed: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// LLM requests made
    pub llm_requests: u64,
    /// LLM tokens consumed
    pub llm_tokens_consumed: u64,
}

/// Runtime execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetadata {
    /// Execution model used
    pub execution_model: String,
    /// Session identifier
    pub session_id: String,
    /// Execution duration
    pub duration: Duration,
    /// Peak resource usage
    pub resource_usage: RuntimeResourceUsage,
    /// Security level used
    pub security_level: SecurityLevel,
    /// Engine version used
    pub engine_version: String,
    /// Execution timestamp
    pub executed_at: std::time::SystemTime,
}

/// Resource usage during runtime execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResourceUsage {
    /// Peak memory usage in MB
    pub peak_memory_mb: u64,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Number of system calls made
    pub syscall_count: u32,
    /// Files accessed during execution
    pub files_accessed: Vec<String>,
    /// Network connections attempted
    pub network_attempts: u32,
}

/// Generated artifact from code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact type (binary, library, etc.)
    pub artifact_type: String,
    /// File path or identifier
    pub path: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Checksum for integrity
    pub checksum: String,
}

/// **Phase 1: Unified Runtime Manager**
/// 
/// Central coordinator for all execution types, eliminating the need
/// for separate runtime managers in different crates.
pub struct RuntimeManager {
    kernel: Arc<RuntimeKernel>,
    engines: RwLock<HashMap<String, Box<dyn ExecutionEngine + Send + Sync>>>,
    execution_history: RwLock<Vec<ExecutionResult>>,
    code_cache: RwLock<HashMap<String, CachedExecution>>,
}

/// Cached execution for performance optimization
#[derive(Debug, Clone)]
struct CachedExecution {
    code_hash: String,
    compiled_artifact: Option<Artifact>,
    last_used: Instant,
    execution_count: u32,
}

/// **Phase 1: Unified Execution Engine Trait**
/// 
/// Single trait for all execution engines (code, agent, tool)
#[async_trait::async_trait]
pub trait ExecutionEngine {
    /// Get engine metadata
    fn metadata(&self) -> EngineMetadata;
    
    /// Validate execution request before processing
    async fn validate_request(&self, request: &ExecutionRequest) -> Result<()>;
    
    /// Execute with kernel enforcement
    async fn execute(
        &self,
        context: &ExecutionContext,
        request: &ExecutionRequest,
        kernel: &ToolKernel,
    ) -> Result<ExecutionResult>;
    
    /// Check if engine supports specific capabilities
    fn supports_capabilities(&self, capabilities: &CapabilitySet) -> bool;
    
    /// Get required capabilities for this engine
    fn required_capabilities(&self) -> CapabilitySet;
}

/// Engine metadata
#[derive(Debug, Clone)]
pub struct EngineMetadata {
    pub name: String,
    pub version: String,
    pub execution_types: Vec<String>,
    pub description: String,
    pub supported_features: Vec<String>,
}

impl RuntimeManager {
    /// Create new unified runtime manager with kernel
    pub async fn new(kernel: ToolKernel) -> Result<Self> {
        let engines: HashMap<String, Box<dyn ExecutionEngine + Send + Sync>> = HashMap::new();
        
        // TODO: Register unified engines when engine modules are implemented
        // engines.insert("python".to_string(), Box::new(engines::PythonEngine::new()));
        // engines.insert("javascript".to_string(), Box::new(engines::JavaScriptEngine::new()));
        // engines.insert("agent".to_string(), Box::new(engines::AgentEngine::new()));
        // engines.insert("tool".to_string(), Box::new(engines::ToolEngine::new()));
        
        Ok(Self {
            kernel: Arc::new(kernel),
            engines: RwLock::new(engines),
            execution_history: RwLock::new(Vec::new()),
            code_cache: RwLock::new(HashMap::new()),
        })
    }
    
    /// **Phase 1: Unified Execute Method**
    /// 
    /// Single entry point for all execution types
    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let _start_time = Instant::now();
        
        // Determine engine based on execution model
        let engine_key = match &request.model {
            ExecutionModel::DynamicCode { code_type, .. } => {
                format!("code_{:?}", code_type).to_lowercase()
            }
            ExecutionModel::AgentWorkflow { .. } => "agent".to_string(),
            ExecutionModel::ToolExecution { .. } => "tool".to_string(),
        };
        
        // Get appropriate execution engine
        let engines = self.engines.read().await;
        let engine = engines.get(&engine_key)
            .ok_or_else(|| anyhow::anyhow!("Unsupported execution model: {}", engine_key))?;
        
        // Get required capabilities for this engine
        let required_capabilities = engine.required_capabilities();
        
        // Create execution context with kernel
        let context = self.kernel.create_execution_context(
            &engine_key,
            &request.session_id,
            &required_capabilities,
            request.security_level.clone(),
        ).await?;
        
        // Validate request before execution
        engine.validate_request(&request).await?;
        
        // Check cache for previously compiled code (if applicable)
        let code_hash = self.calculate_request_hash(&request);
        let _cached_artifact = self.get_cached_execution(&code_hash).await;
        
        // Execute with kernel enforcement
        let result = self.kernel.enforce_execution(&context, async {
            // Execute through the appropriate engine
            engine.execute(&context, &request, &self.kernel).await
        }).await?;
        
        // Update cache if compilation occurred
        if let Some(artifact) = result.artifacts.first() {
            self.update_cache(code_hash, artifact.clone()).await;
        }
        
        // Store execution history
        let mut history = self.execution_history.write().await;
        history.push(result.clone());
        
        // Keep only recent executions
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        Ok(result)
    }

    /// **Phase 1: Legacy Support - Execute Code**
    /// 
    /// Maintains backward compatibility with existing code execution
    pub async fn execute_code(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        // Convert to unified execution model if needed
        let unified_request = match &request.model {
            ExecutionModel::DynamicCode { .. } => request,
            _ => {
                // Convert legacy request format if needed
                request
            }
        };
        
        self.execute(unified_request).await
    }
    
    /// Generate code dynamically based on requirements
    pub async fn generate_code(
        &self,
        prompt: &str,
        code_type: CodeType,
        session_id: &str,
    ) -> Result<String> {
        // Validate code generation capability
        let capabilities = CapabilitySet::with_capabilities(vec![
            Capability::CodeGeneration,
        ]);
        
        let context = self.kernel.create_execution_context(
            "code_generator",
            session_id,
            &capabilities,
            SecurityLevel::Restricted,
        ).await?;
        
        // Generate code through kernel enforcement
        self.kernel.enforce_execution(&context, async {
            // TODO: Implement code generation when generation module is available
            // generation::generate_code(prompt, code_type).await
            Ok(format!("// TODO: Generated {} code for prompt: {}", 
                      match code_type {
                          CodeType::Python => "Python",
                          CodeType::JavaScript => "JavaScript",
                          CodeType::WebAssembly => "WebAssembly",
                          CodeType::Shell => "Shell",
                          CodeType::Rust => "Rust",
                      }, prompt))
        }).await
    }
    
    /// **Phase 1: Agent Execution Support**
    /// 
    /// Execute agent workflows through the unified runtime
    pub async fn execute_agent(&self, agent_config: AgentConfig, agent_id: EntityId, session_id: &str) -> Result<ExecutionResult> {
        let request = ExecutionRequest {
            model: ExecutionModel::AgentWorkflow {
                agent_config,
                agent_id,
            },
            session_id: session_id.to_string(),
            security_level: SecurityLevel::Medium,
            inputs: JsonValue::Object(serde_json::Map::new()),
            timeout_override: None,
            environment: None,
        };
        
        self.execute(request).await
    }
    
    /// **Phase 1: Tool Execution Support**
    /// 
    /// Execute tools through the unified runtime
    pub async fn execute_tool(&self, tool_name: &str, tool_args: JsonValue, session_id: &str) -> Result<ExecutionResult> {
        let request = ExecutionRequest {
            model: ExecutionModel::ToolExecution {
                tool_name: tool_name.to_string(),
                tool_args,
            },
            session_id: session_id.to_string(),
            security_level: SecurityLevel::Medium,
            inputs: JsonValue::Object(serde_json::Map::new()),
            timeout_override: None,
            environment: None,
        };
        
        self.execute(request).await
    }
    
    /// Register a custom execution engine
    pub async fn register_engine(
        &self,
        engine_key: String,
        engine: Box<dyn ExecutionEngine + Send + Sync>,
    ) -> Result<()> {
        let mut engines = self.engines.write().await;
        engines.insert(engine_key, engine);
        Ok(())
    }
    
    /// List available execution engines
    pub async fn list_engines(&self) -> Vec<EngineMetadata> {
        let engines = self.engines.read().await;
        engines.values()
            .map(|engine| engine.metadata())
            .collect()
    }
    
    /// Get execution history
    pub async fn get_execution_history(&self) -> Vec<ExecutionResult> {
        let history = self.execution_history.read().await;
        history.clone()
    }
    
    /// Clear execution cache
    pub async fn clear_cache(&self) {
        let mut cache = self.code_cache.write().await;
        cache.clear();
    }

    /// Submit a message to the kernel (delegation method)
    pub async fn submit(&self, message: Message) -> Result<toka_bus_core::KernelEvent> {
        // TODO: This is a placeholder - implement proper message submission when kernel supports it
        // For now, just log the message and return a mock event
        tracing::info!("Message submitted: {:?}", message);
        use toka_bus_core::KernelEvent;
        use chrono::Utc;
        
        // Create a mock response based on the operation type
        match &message.op {
            Operation::SpawnSubAgent { parent, spec } => {
                Ok(KernelEvent::AgentSpawned {
                    parent: *parent,
                    spec: spec.clone(),
                    timestamp: Utc::now(),
                })
            }
            Operation::ScheduleAgentTask { agent, task } => {
                Ok(KernelEvent::TaskScheduled {
                    agent: *agent,
                    task: task.clone(),
                    timestamp: Utc::now(),
                })
            }
            Operation::EmitObservation { agent, data: _ } => {
                Ok(KernelEvent::TaskScheduled {
                    agent: *agent,
                    task: TaskSpec {
                        description: "Mock observation task".to_string(),
                    },
                    timestamp: Utc::now(),
                })
            }
        }
    }
    
    /// Calculate hash for request caching
    fn calculate_request_hash(&self, request: &ExecutionRequest) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        // Hash different parts based on execution model
        match &request.model {
            ExecutionModel::DynamicCode { code, .. } => {
                hasher.update(code.as_bytes());
            }
            ExecutionModel::AgentWorkflow { agent_config, .. } => {
                if let Ok(serialized) = serde_json::to_string(agent_config) {
                    hasher.update(serialized.as_bytes());
                }
            }
            ExecutionModel::ToolExecution { tool_name, tool_args } => {
                hasher.update(tool_name.as_bytes());
                if let Ok(serialized) = serde_json::to_string(tool_args) {
                    hasher.update(serialized.as_bytes());
                }
            }
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    /// Get cached execution if available
    async fn get_cached_execution(&self, code_hash: &str) -> Option<Artifact> {
        let mut cache = self.code_cache.write().await;
        
        if let Some(cached) = cache.get_mut(code_hash) {
            cached.last_used = Instant::now();
            cached.execution_count += 1;
            cached.compiled_artifact.clone()
        } else {
            None
        }
    }
    
    /// Update code cache with new execution
    async fn update_cache(&self, code_hash: String, artifact: Artifact) {
        let mut cache = self.code_cache.write().await;
        
        cache.insert(code_hash.clone(), CachedExecution {
            code_hash,
            compiled_artifact: Some(artifact),
            last_used: Instant::now(),
            execution_count: 1,
        });
        
        // Cleanup old cache entries
        if cache.len() > 1000 {
            let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
            cache.retain(|_, cached| cached.last_used > cutoff);
        }
    }
}

/// Builder for runtime manager with custom configuration
pub struct RuntimeBuilder {
    kernel: ToolKernel,
    engines: HashMap<String, Box<dyn ExecutionEngine + Send + Sync>>,
}

impl RuntimeBuilder {
    /// Create new runtime builder
    pub fn new(kernel: ToolKernel) -> Self {
        Self {
            kernel,
            engines: HashMap::new(),
        }
    }
    
    /// Add custom execution engine
    pub fn with_engine(
        mut self,
        engine_key: String,
        engine: Box<dyn ExecutionEngine + Send + Sync>,
    ) -> Self {
        self.engines.insert(engine_key, engine);
        self
    }
    
    /// Build the runtime manager
    pub async fn build(self) -> Result<RuntimeManager> {
        let mut runtime = RuntimeManager::new(self.kernel).await?;
        
        // Register custom engines
        for (key, engine) in self.engines {
            runtime.register_engine(key, engine).await?;
        }
        
        Ok(runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_model_serialization() {
        let model = ExecutionModel::DynamicCode {
            code_type: CodeType::Python,
            code: "print('hello')".to_string(),
        };
        
        let serialized = serde_json::to_string(&model).unwrap();
        let deserialized: ExecutionModel = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            ExecutionModel::DynamicCode { code_type, code } => {
                assert_eq!(code_type, CodeType::Python);
                assert_eq!(code, "print('hello')");
            }
            _ => panic!("Wrong execution model type"),
        }
    }

    #[test]
    fn test_code_types() {
        let types = vec![
            CodeType::Python,
            CodeType::JavaScript,
            CodeType::WebAssembly,
            CodeType::Shell,
            CodeType::Rust,
        ];
        
        for code_type in types {
            let serialized = serde_json::to_string(&code_type).unwrap();
            let deserialized: CodeType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(code_type, deserialized);
        }
    }

    #[test]
    fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.tasks_attempted, 0);
        assert_eq!(metrics.tasks_completed, 0);
        assert_eq!(metrics.total_execution_time, Duration::ZERO);
    }
}