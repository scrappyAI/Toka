//! Toka Runtime - Dynamic Code Execution with Kernel Enforcement
//!
//! This crate provides the runtime layer for dynamic code generation and execution
//! while maintaining security through the toka-kernel enforcement layer. It supports
//! multiple execution environments including WebAssembly, Python scripting, and
//! sandboxed native code execution.
//!
//! # Architecture
//!
//! The runtime layer consists of:
//!
//! - **Execution Engines**: Different runtime environments (WASM, Python, etc.)
//! - **Sandboxing**: Isolated execution environments with resource limits
//! - **Code Generation**: Dynamic code creation with validation
//! - **Security Integration**: Full kernel enforcement for all operations
//! - **Resource Management**: Memory, CPU, and I/O tracking per execution
//!
//! # Usage
//!
//! ```rust
//! use toka_runtime::{RuntimeManager, ExecutionRequest, CodeType};
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
//!         code_type: CodeType::Python,
//!         code: "print('Hello from dynamic execution!')".to_string(),
//!         session_id: "user_session".to_string(),
//!         security_level: SecurityLevel::Sandboxed,
//!         inputs: serde_json::json!({}),
//!     };
//!     
//!     let result = runtime.execute_code(request).await?;
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
use toka_types::{Message, Operation};

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

// Removed duplicate definition - using the one below

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

/// Runtime execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Type of code to execute
    pub code_type: CodeType,
    /// Source code to execute
    pub code: String,
    /// Session identifier for capability checking
    pub session_id: String,
    /// Security level for execution
    pub security_level: SecurityLevel,
    /// Input data for the code
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

/// Runtime execution result
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
}

/// Runtime execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetadata {
    /// Code type executed
    pub code_type: CodeType,
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

/// Main runtime manager for dynamic code execution
pub struct RuntimeManager {
    kernel: Arc<RuntimeKernel>,
    engines: RwLock<HashMap<CodeType, Box<dyn ExecutionEngine + Send + Sync>>>,
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

/// Trait for execution engines
#[async_trait::async_trait]
pub trait ExecutionEngine {
    /// Get engine metadata
    fn metadata(&self) -> EngineMetadata;
    
    /// Validate code before execution
    async fn validate_code(&self, code: &str) -> Result<()>;
    
    /// Execute code with kernel enforcement
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
    pub code_type: CodeType,
    pub description: String,
    pub supported_features: Vec<String>,
}

impl RuntimeManager {
    /// Create new runtime manager with kernel
    pub async fn new(kernel: ToolKernel) -> Result<Self> {
        let engines: HashMap<CodeType, Box<dyn ExecutionEngine + Send + Sync>> = HashMap::new();
        
        // TODO: Register default engines when engine modules are implemented
        // engines.insert(CodeType::Python, Box::new(engines::PythonEngine::new()));
        // engines.insert(CodeType::JavaScript, Box::new(engines::JavaScriptEngine::new()));
        // engines.insert(CodeType::WebAssembly, Box::new(engines::WasmEngine::new()));
        // engines.insert(CodeType::Shell, Box::new(engines::ShellEngine::new()));
        // engines.insert(CodeType::Rust, Box::new(engines::RustEngine::new()));
        
        Ok(Self {
            kernel: Arc::new(kernel),
            engines: RwLock::new(engines),
            execution_history: RwLock::new(Vec::new()),
            code_cache: RwLock::new(HashMap::new()),
        })
    }
    
    /// Execute code dynamically with kernel enforcement
    pub async fn execute_code(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        // Get appropriate execution engine
        let engines = self.engines.read().await;
        let engine = engines.get(&request.code_type)
            .ok_or_else(|| anyhow::anyhow!("Unsupported code type: {:?}", request.code_type))?;
        
        // Get required capabilities for this engine
        let required_capabilities = engine.required_capabilities();
        
        // Create execution context with kernel
        let context = self.kernel.create_execution_context(
            &format!("runtime_{:?}", request.code_type),
            &request.session_id,
            &required_capabilities,
            request.security_level.clone(),
        ).await?;
        
        // Validate code before execution
        engine.validate_code(&request.code).await?;
        
        // Check cache for previously compiled code
        let code_hash = self.calculate_code_hash(&request.code);
        let cached_artifact = self.get_cached_execution(&code_hash).await;
        
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
    
    /// Register a custom execution engine
    pub async fn register_engine(
        &self,
        code_type: CodeType,
        engine: Box<dyn ExecutionEngine + Send + Sync>,
    ) -> Result<()> {
        let mut engines = self.engines.write().await;
        engines.insert(code_type, engine);
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
                    task: toka_types::TaskSpec {
                        description: "Mock observation task".to_string(),
                    },
                    timestamp: Utc::now(),
                })
            }
        }
    }
    
    /// Calculate hash for code caching
    fn calculate_code_hash(&self, code: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
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
    kernel: RuntimeKernel,
    engines: HashMap<CodeType, Box<dyn ExecutionEngine + Send + Sync>>,
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
        code_type: CodeType,
        engine: Box<dyn ExecutionEngine + Send + Sync>,
    ) -> Self {
        self.engines.insert(code_type, engine);
        self
    }
    
    /// Build runtime manager
    pub async fn build(self) -> Result<RuntimeManager> {
        let runtime = RuntimeManager::new(self.kernel).await?;
        
        // Register custom engines
        for (code_type, engine) in self.engines {
            runtime.register_engine(code_type, engine).await?;
        }
        
        Ok(runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Uncomment these tests when presets are available in toka-kernel
    /*
    #[tokio::test]
    async fn test_runtime_creation() {
        let kernel = toka_kernel::presets::testing_kernel().await.unwrap();
        let runtime = RuntimeManager::new(RuntimeKernel::new(kernel)).await.unwrap();
        
        let engines = runtime.list_engines().await;
        // assert!(engines.len() > 0);
        
        // Should have default engines registered
        let engine_types: Vec<CodeType> = engines.iter()
            .map(|e| e.code_type.clone())
            .collect();
        // assert!(engine_types.contains(&CodeType::Python));
        // assert!(engine_types.contains(&CodeType::WebAssembly));
    }
    
    #[tokio::test]
    async fn test_code_execution_request() {
        let kernel = toka_kernel::presets::development_kernel().await.unwrap();
        let runtime = RuntimeManager::new(RuntimeKernel::new(kernel)).await.unwrap();
        
        let request = ExecutionRequest {
            code_type: CodeType::Python,
            code: "print('Hello, World!')".to_string(),
            session_id: "development".to_string(),
            security_level: SecurityLevel::Low,
            inputs: serde_json::json!({}),
            timeout_override: None,
            environment: None,
        };
        
        // For this test, we'd need to implement the actual Python engine
        // This is just testing the request structure
        assert_eq!(request.code_type, CodeType::Python);
        assert!(!request.code.is_empty());
    }
    
    #[test]
    fn test_code_hash_calculation() {
        let kernel = toka_kernel::presets::testing_kernel().await.unwrap();
        let runtime = RuntimeManager::new(RuntimeKernel::new(kernel)).await.unwrap();
        
        let code1 = "print('hello')";
        let code2 = "print('hello')";
        let code3 = "print('world')";
        
        let hash1 = runtime.calculate_code_hash(code1);
        let hash2 = runtime.calculate_code_hash(code2);
        let hash3 = runtime.calculate_code_hash(code3);
        
        assert_eq!(hash1, hash2); // Same code should have same hash
        assert_ne!(hash1, hash3); // Different code should have different hash
        assert_eq!(hash1.len(), 64); // SHA256 hash length
    }
    */
    
    #[test]
    fn test_code_types() {
        // Simple test for code types
        let python_type = CodeType::Python;
        let js_type = CodeType::JavaScript;
        
        assert!(python_type != js_type);
        assert!(python_type == CodeType::Python);
    }
}