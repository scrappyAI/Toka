# Toka Architecture Cleanup Implementation Guide

**Date**: 2025-01-15  
**Target**: Production-ready core architecture  
**Timeline**: 4-5 weeks  

This guide provides **specific step-by-step instructions** to implement the architecture cleanup and consolidation.

## Prerequisites

Before starting, ensure you have:
- [ ] Rust 1.75+ installed
- [ ] Python 3.8+ for script tools
- [ ] Git for provenance tracking
- [ ] Docker for container deployment
- [ ] Full Toka workspace builds successfully

## Phase 1: Runtime Consolidation (Week 1-2)

### Step 1.1: Merge Agent Runtime into Unified Runtime

**Goal**: Consolidate `toka-agent-runtime` into `toka-runtime` as an execution model.

#### 1.1.1 Create New Runtime Architecture

Edit `crates/toka-runtime/src/lib.rs`:

```rust
// ADD: New unified execution models
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Import agent runtime components
pub use toka_agent_runtime::{
    AgentExecutor, TaskExecutor, ProgressReporter, 
    CapabilityValidator, ResourceManager
};

/// Unified execution model for all runtime operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionModel {
    /// Dynamic code execution (current toka-runtime)
    DynamicCode {
        code_type: CodeType,
        code: String,
        sandbox_config: SandboxConfig,
    },
    /// Agent workflow execution (from toka-agent-runtime)
    AgentWorkflow {
        agent_config: AgentConfig,
        agent_id: EntityId,
        llm_integration: bool,
    },
    /// External tool execution
    ToolExecution {
        tool_name: String,
        tool_params: ToolParams,
        security_context: ToolSecurityContext,
    },
}

/// Unified runtime manager
pub struct RuntimeManager {
    /// Core kernel interface
    kernel: Arc<RuntimeKernel>,
    /// Code execution engine
    code_executor: CodeExecutor,
    /// Agent execution engine
    agent_executor: Arc<AgentExecutor>,
    /// Tool registry
    tool_registry: Arc<ToolRegistry>,
    /// Security validator
    security_validator: Arc<CapabilityValidator>,
    /// Resource manager
    resource_manager: Arc<ResourceManager>,
}

impl RuntimeManager {
    /// Create new unified runtime manager
    pub async fn new(kernel: RuntimeKernel) -> Result<Self> {
        let kernel = Arc::new(kernel);
        
        // Initialize execution engines
        let code_executor = CodeExecutor::new(kernel.clone()).await?;
        let tool_registry = Arc::new(ToolRegistry::new().await?);
        
        // Initialize security and resource management
        let security_validator = Arc::new(CapabilityValidator::new());
        let resource_manager = Arc::new(ResourceManager::new());
        
        // Agent executor will be created per agent
        Ok(Self {
            kernel,
            code_executor,
            agent_executor: Arc::new(AgentExecutor::placeholder()),
            tool_registry,
            security_validator,
            resource_manager,
        })
    }
    
    /// Execute any runtime model
    pub async fn execute(&self, model: ExecutionModel) -> Result<ExecutionResult> {
        match model {
            ExecutionModel::DynamicCode { code_type, code, sandbox_config } => {
                let request = ExecutionRequest {
                    code_type,
                    code,
                    sandbox_config,
                    ..Default::default()
                };
                self.code_executor.execute(request).await
            },
            ExecutionModel::AgentWorkflow { agent_config, agent_id, llm_integration } => {
                let agent_executor = AgentExecutor::new(
                    agent_config,
                    agent_id,
                    self.clone(),
                    llm_integration,
                ).await?;
                agent_executor.run().await
            },
            ExecutionModel::ToolExecution { tool_name, tool_params, security_context } => {
                self.tool_registry.execute_tool_secure(
                    &tool_name,
                    &tool_params,
                    &security_context,
                ).await
            },
        }
    }
}
```

#### 1.1.2 Move Agent Runtime Components

Create `crates/toka-runtime/src/agents/mod.rs`:

```bash
# Copy agent runtime modules into toka-runtime
mkdir -p crates/toka-runtime/src/agents
cp crates/toka-agent-runtime/src/*.rs crates/toka-runtime/src/agents/

# Update mod.rs
cat > crates/toka-runtime/src/agents/mod.rs << 'EOF'
//! Agent execution components integrated into unified runtime

pub mod executor;
pub mod task;
pub mod capability;
pub mod resource;
pub mod progress;

pub use executor::AgentExecutor;
pub use task::TaskExecutor;
pub use capability::CapabilityValidator;
pub use resource::ResourceManager;
pub use progress::ProgressReporter;
EOF
```

#### 1.1.3 Update Cargo Dependencies

Edit `crates/toka-runtime/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
toka-llm-gateway = { path = "../toka-llm-gateway" }
toka-orchestration = { path = "../toka-orchestration" }

# Add agent runtime capabilities
uuid = { version = "1", features = ["v4", "serde"] }
dashmap = "5.5"
```

#### 1.1.4 Remove Circular Dependencies

Edit `crates/toka-orchestration/src/lib.rs`:

```rust
// REMOVE: Direct agent runtime imports
// use toka_agent_runtime::{AgentExecutor, AgentProcessManager};

// ADD: Event-driven agent communication
use toka_bus_core::{EventBus, KernelEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationEvent {
    SpawnAgent {
        config: AgentConfig,
        id: EntityId,
    },
    AgentProgress {
        id: EntityId,
        progress: f64,
        message: Option<String>,
    },
    AgentComplete {
        id: EntityId,
        result: TaskResult,
    },
    AgentFailed {
        id: EntityId,
        error: String,
    },
}

pub struct OrchestrationEngine {
    config: OrchestrationConfig,
    runtime: Arc<RuntimeManager>,
    event_bus: Arc<dyn EventBus>,
    spawned_agents: Arc<DashMap<EntityId, SpawnedAgent>>,
    agent_states: Arc<DashMap<String, AgentState>>,
}

impl OrchestrationEngine {
    pub async fn spawn_agent(&self, config: AgentConfig) -> Result<EntityId> {
        let agent_id = EntityId::new();
        
        // Create agent execution model
        let execution_model = ExecutionModel::AgentWorkflow {
            agent_config: config.clone(),
            agent_id,
            llm_integration: true,
        };
        
        // Execute via runtime (non-blocking)
        let runtime = self.runtime.clone();
        tokio::spawn(async move {
            if let Err(e) = runtime.execute(execution_model).await {
                // Emit failure event
                runtime.emit_event(OrchestrationEvent::AgentFailed {
                    id: agent_id,
                    error: e.to_string(),
                });
            }
        });
        
        Ok(agent_id)
    }
}
```

### Step 1.2: Delete Agent Runtime Crate

```bash
# After confirming all functionality is moved
rm -rf crates/toka-agent-runtime

# Update workspace Cargo.toml
# REMOVE: "crates/toka-agent-runtime"
```

### Step 1.3: Update All References

Find and update all imports:

```bash
# Find all references to toka-agent-runtime
grep -r "toka_agent_runtime" crates/ --include="*.rs" --include="*.toml"

# Update to use toka_runtime::agents
sed -i 's/toka_agent_runtime/toka_runtime::agents/g' crates/*/src/*.rs
sed -i 's/toka-agent-runtime/toka-runtime/g' crates/*/Cargo.toml
```

## Phase 2: Tool System Unification (Week 2-3)

### Step 2.1: Create Unified Tool Registry

Edit `crates/toka-tools/src/registry.rs`:

```rust
//! Unified tool registry supporting all tool types

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::core::{Tool, ToolParams, ToolResult, ToolMetadata};

/// Unified tool registry supporting native, shell, and Python tools
pub struct UnifiedToolRegistry {
    /// Native Rust tools
    native_tools: Arc<RwLock<HashMap<String, Arc<dyn Tool + Send + Sync>>>>,
    /// External tools (shell, Python, etc.)
    external_tools: Arc<RwLock<HashMap<String, ExternalTool>>>,
    /// Tool manifests
    manifests: Arc<RwLock<HashMap<String, ToolManifest>>>,
    /// Security validator
    security_validator: Arc<CapabilityValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub metadata: ToolMetadata,
    pub spec: ToolSpec,
    pub interface: ToolInterface,
    pub security: SecuritySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub executable: ExecutableSpec,
    pub capabilities: Vec<String>,
    pub parameters: Vec<ParameterSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableSpec {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub path: PathBuf,
    pub interpreter: Option<String>,
    pub working_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Native,
    Shell,
    Python,
    External,
}

/// External tool wrapper for shell, Python, and other executables
#[derive(Debug, Clone)]
pub struct ExternalTool {
    pub manifest: ToolManifest,
    pub executable_path: PathBuf,
    pub interpreter: Option<String>,
}

impl UnifiedToolRegistry {
    /// Create new unified tool registry
    pub async fn new() -> Result<Self> {
        Ok(Self {
            native_tools: Arc::new(RwLock::new(HashMap::new())),
            external_tools: Arc::new(RwLock::new(HashMap::new())),
            manifests: Arc::new(RwLock::new(HashMap::new())),
            security_validator: Arc::new(CapabilityValidator::new()),
        })
    }
    
    /// Register a native Rust tool
    pub async fn register_native_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        let name = tool.name().to_string();
        
        // Create manifest for native tool
        let manifest = self.create_native_tool_manifest(&tool).await?;
        
        // Register tool and manifest
        self.native_tools.write().await.insert(name.clone(), tool);
        self.manifests.write().await.insert(name, manifest);
        
        Ok(())
    }
    
    /// Register an external tool from manifest
    pub async fn register_external_tool(&self, manifest_path: &Path) -> Result<()> {
        let manifest_content = tokio::fs::read_to_string(manifest_path).await?;
        let manifest: ToolManifest = serde_yaml::from_str(&manifest_content)?;
        
        let external_tool = ExternalTool {
            executable_path: manifest.spec.executable.path.clone(),
            interpreter: manifest.spec.executable.interpreter.clone(),
            manifest: manifest.clone(),
        };
        
        let name = manifest.metadata.name.clone();
        self.external_tools.write().await.insert(name.clone(), external_tool);
        self.manifests.write().await.insert(name, manifest);
        
        Ok(())
    }
    
    /// Execute any tool type with security validation
    pub async fn execute_tool_secure(
        &self,
        tool_name: &str,
        params: &ToolParams,
        security_context: &ToolSecurityContext,
    ) -> Result<ToolResult> {
        // 1. Validate capabilities
        self.validate_tool_access(tool_name, &security_context.capabilities).await?;
        
        // 2. Execute appropriate tool type
        if let Some(native_tool) = self.native_tools.read().await.get(tool_name) {
            self.execute_native_tool(native_tool.clone(), params, security_context).await
        } else if let Some(external_tool) = self.external_tools.read().await.get(tool_name) {
            self.execute_external_tool(external_tool.clone(), params, security_context).await
        } else {
            Err(anyhow::anyhow!("Tool '{}' not found", tool_name))
        }
    }
    
    /// Execute native Rust tool
    async fn execute_native_tool(
        &self,
        tool: Arc<dyn Tool + Send + Sync>,
        params: &ToolParams,
        _security_context: &ToolSecurityContext,
    ) -> Result<ToolResult> {
        tool.execute(params).await
    }
    
    /// Execute external tool (shell, Python, etc.)
    async fn execute_external_tool(
        &self,
        external_tool: ExternalTool,
        params: &ToolParams,
        security_context: &ToolSecurityContext,
    ) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // Build command based on tool type
        let mut cmd = match external_tool.manifest.spec.executable.tool_type {
            ToolType::Shell => {
                let mut cmd = Command::new("bash");
                cmd.arg(&external_tool.executable_path);
                cmd
            },
            ToolType::Python => {
                let mut cmd = Command::new("python3");
                cmd.arg(&external_tool.executable_path);
                cmd
            },
            ToolType::External => {
                Command::new(&external_tool.executable_path)
            },
            ToolType::Native => unreachable!("Native tools handled separately"),
        };
        
        // Add parameters as arguments
        for (key, value) in &params.args {
            cmd.arg(format!("--{}", key));
            cmd.arg(value);
        }
        
        // Configure security sandbox
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Set resource limits (simplified)
        // In production, use proper sandboxing like firejail or containers
        
        // Execute command
        let output = cmd.output().await
            .context("Failed to execute external tool")?;
        
        let execution_time = start_time.elapsed();
        
        // Create result
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            metadata: ToolMetadata {
                execution_time_ms: execution_time.as_millis() as u64,
                tool_version: external_tool.manifest.metadata.version,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }
    
    /// Discover and register all tools from manifest directory
    pub async fn discover_tools(&self, manifest_dir: &Path) -> Result<usize> {
        let mut count = 0;
        let mut entries = tokio::fs::read_dir(manifest_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Err(e) = self.register_external_tool(&path).await {
                    eprintln!("Failed to register tool from {}: {}", path.display(), e);
                } else {
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
}

/// Tool security context for capability validation
#[derive(Debug, Clone)]
pub struct ToolSecurityContext {
    pub agent_id: EntityId,
    pub capabilities: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub sandbox_config: SandboxConfig,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    pub max_memory: Option<String>,
    pub max_cpu: Option<String>,
    pub timeout: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    pub allow_network: bool,
    pub readonly_paths: Vec<PathBuf>,
    pub memory_limit: Option<u64>,
}
```

### Step 2.2: Create Tool Manifests

Create manifest generation script `scripts/generate-tool-manifests.py`:

```python
#!/usr/bin/env python3
"""
Generate tool manifests for all Toka tools
"""

import os
import yaml
from pathlib import Path
from datetime import datetime

def create_shell_tool_manifest(script_path: Path, output_dir: Path):
    """Create manifest for shell script tool"""
    name = script_path.stem.replace('_', '-')
    
    # Analyze script for capabilities
    with open(script_path, 'r') as f:
        content = f.read()
    
    capabilities = []
    if 'mkdir' in content or 'touch' in content:
        capabilities.append('filesystem-write')
    if 'cat' in content or 'grep' in content:
        capabilities.append('filesystem-read')
    if 'cargo' in content or 'python' in content:
        capabilities.append('process-execution')
    if 'curl' in content or 'wget' in content:
        capabilities.append('network-access')
    
    manifest = {
        'metadata': {
            'name': name,
            'version': '1.0.0',
            'category': 'external',
            'description': f'Shell script tool: {name}',
            'author': 'toka-system',
            'created': datetime.now().strftime('%Y-%m-%d'),
        },
        'spec': {
            'executable': {
                'type': 'shell',
                'path': str(script_path),
                'interpreter': 'bash',
                'working_directory': '.',
            },
            'capabilities': {
                'required': capabilities or ['filesystem-read'],
                'optional': [],
            },
            'security': {
                'level': 'medium',
                'sandbox': {
                    'memory_limit': '512MB',
                    'cpu_limit': '50%',
                    'timeout': '300s',
                    'allow_network': 'network-access' in capabilities,
                },
            },
        },
        'interface': {
            'discovery': {'auto_discover': True},
            'execution': {'hot_swappable': True, 'parallel_safe': True},
        },
    }
    
    output_path = output_dir / f"{name}.yaml"
    with open(output_path, 'w') as f:
        yaml.dump(manifest, f, default_flow_style=False)
    
    print(f"Created manifest: {output_path}")

def main():
    project_root = Path(__file__).parent.parent
    scripts_dir = project_root / 'scripts'
    manifests_dir = project_root / 'crates/toka-tools/manifests'
    
    # Create manifests directory
    manifests_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate manifests for all shell scripts
    for script_path in scripts_dir.rglob('*.sh'):
        if script_path.is_file():
            create_shell_tool_manifest(script_path, manifests_dir)
    
    print(f"Generated manifests in {manifests_dir}")

if __name__ == '__main__':
    main()
```

Run the script:

```bash
python3 scripts/generate-tool-manifests.py
```

### Step 2.3: Update Tool Registration

Edit `crates/toka-tools/src/lib.rs`:

```rust
// ADD: Unified tool system exports
pub use crate::registry::{
    UnifiedToolRegistry, ExternalTool, ToolManifest, 
    ToolSecurityContext, ResourceLimits, SandboxConfig
};

/// Initialize unified tool system with all available tools
pub async fn initialize_unified_tools() -> Result<UnifiedToolRegistry> {
    let registry = UnifiedToolRegistry::new().await?;
    
    // Register native tools
    crate::tools::register_essential_tools(&registry).await?;
    
    // Discover and register external tools
    let manifest_dir = Path::new("crates/toka-tools/manifests");
    if manifest_dir.exists() {
        let count = registry.discover_tools(manifest_dir).await?;
        println!("Registered {} external tools", count);
    }
    
    Ok(registry)
}
```

## Phase 3: Core Crate Consolidation (Week 3-4)

### Step 3.1: Merge Collaborative Auth

Move collaborative auth into main auth crate:

```bash
# Create collaborative module
mkdir -p crates/toka-auth/src/collaborative
cp crates/toka-collaborative-auth/src/*.rs crates/toka-auth/src/collaborative/

# Update toka-auth/src/lib.rs
echo 'pub mod collaborative;' >> crates/toka-auth/src/lib.rs

# Update Cargo.toml dependencies
# Add GitHub client dependencies to toka-auth
```

### Step 3.2: Consolidate Security Crates

Create `crates/toka-security/` to consolidate:

```bash
mkdir -p crates/toka-security/src
cd crates/toka-security

# Create unified security crate
cat > Cargo.toml << 'EOF'
[package]
name = "toka-security"
version.workspace = true
edition.workspace = true

[dependencies]
# Core dependencies
toka-types = { path = "../toka-types" }
toka-auth = { path = "../toka-auth" }

# Capability system
toka-capability-core = { path = "../security/toka-capability-core" }
toka-capability-jwt-hs256 = { path = "../security/toka-capability-jwt-hs256" }
toka-capability-delegation = { path = "../security/toka-capability-delegation" }

# Security services
toka-key-rotation = { path = "../security/toka-key-rotation" }
toka-rate-limiter = { path = "../security/toka-rate-limiter" }
toka-revocation = { path = "../security/toka-revocation" }

# Common dependencies
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
tokio = { workspace = true, features = ["full"] }
EOF

cat > src/lib.rs << 'EOF'
//! Unified Toka security system

pub mod capabilities {
    pub use toka_capability_core::*;
    pub use toka_capability_jwt_hs256::*;
    pub use toka_capability_delegation::*;
}

pub mod services {
    pub use toka_key_rotation::*;
    pub use toka_rate_limiter::*;
    pub use toka_revocation::*;
}

// Re-export all security functionality
pub use capabilities::*;
pub use services::*;
EOF
```

### Step 3.3: Remove Non-Essential Crates

```bash
# Move demo environment to examples
mkdir -p examples/demo-environment
mv crates/toka-demo-environment/* examples/demo-environment/

# Move testing utilities to individual crate tests
rm -rf crates/toka-testing

# Consolidate rule metadata into tools
mv crates/toka-rule-metadata/src/* crates/toka-tools/src/metadata/

# Remove these from workspace Cargo.toml
```

## Phase 4: Deployment Readiness (Week 4-5)

### Step 4.1: Create Unified CLI

Edit `crates/toka-cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;
use toka_tools::UnifiedToolRegistry;
use toka_runtime::RuntimeManager;

#[derive(Parser)]
#[command(name = "toka")]
#[command(about = "Toka Agent OS CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Agent orchestration commands
    Orchestration {
        #[command(subcommand)]
        cmd: OrchestrationCommands,
    },
    /// Tool management commands
    Tools {
        #[command(subcommand)]
        cmd: ToolCommands,
    },
    /// Runtime commands
    Runtime {
        #[command(subcommand)]
        cmd: RuntimeCommands,
    },
}

#[derive(Subcommand)]
enum ToolCommands {
    /// List available tools
    List,
    /// Register tools from manifests
    Register {
        #[arg(long)]
        manifest_dir: String,
    },
    /// Execute a tool
    Execute {
        tool_name: String,
        #[arg(long)]
        params: Vec<String>,
    },
}

async fn handle_tools_command(cmd: ToolCommands) -> Result<()> {
    let registry = toka_tools::initialize_unified_tools().await?;
    
    match cmd {
        ToolCommands::List => {
            println!("Available tools:");
            for tool_name in registry.list_tools().await {
                println!("  - {}", tool_name);
            }
        },
        ToolCommands::Register { manifest_dir } => {
            let count = registry.discover_tools(&manifest_dir.into()).await?;
            println!("Registered {} tools", count);
        },
        ToolCommands::Execute { tool_name, params } => {
            // Parse params and execute tool
            let tool_params = parse_tool_params(params)?;
            let security_context = create_default_security_context();
            
            let result = registry.execute_tool_secure(
                &tool_name,
                &tool_params,
                &security_context,
            ).await?;
            
            println!("Tool result: {}", result.output);
        },
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Tools { cmd } => handle_tools_command(cmd).await,
        Commands::Orchestration { cmd } => handle_orchestration_command(cmd).await,
        Commands::Runtime { cmd } => handle_runtime_command(cmd).await,
    }
}
```

### Step 4.2: Create Deployment Container

Create `Dockerfile.unified`:

```dockerfile
# Multi-stage build for unified Toka system
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Build all required binaries
RUN cargo build --release \
    --bin toka-orchestration \
    --bin toka-cli \
    --bin toka-config

# Production container
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    python3 python3-pip python3-venv \
    bash curl git \
    && rm -rf /var/lib/apt/lists/*

# Create toka user
RUN useradd -m -s /bin/bash toka

# Copy binaries
COPY --from=builder /app/target/release/toka-orchestration /usr/local/bin/
COPY --from=builder /app/target/release/toka-cli /usr/local/bin/
COPY --from=builder /app/target/release/toka-config /usr/local/bin/

# Copy tools and manifests
COPY --from=builder /app/scripts/ /app/scripts/
COPY --from=builder /app/crates/toka-tools/manifests/ /app/tools/
COPY --from=builder /app/.cursor/version-manager.py /app/tools/

# Set up Python environment for Python tools
RUN python3 -m venv /app/venv
ENV PATH="/app/venv/bin:$PATH"
COPY scripts/requirements.txt /app/
RUN pip install -r /app/requirements.txt

# Set permissions
RUN chown -R toka:toka /app
USER toka

# Initialize tool registry
RUN /usr/local/bin/toka-cli tools register --manifest-dir /app/tools/

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/toka-cli orchestration health || exit 1

# Default command
CMD ["/usr/local/bin/toka-orchestration", "--config", "/app/config/agents.toml"]
```

### Step 4.3: Create Kubernetes Deployment

Create `k8s/toka-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: toka-orchestration
  labels:
    app: toka
spec:
  replicas: 3
  selector:
    matchLabels:
      app: toka
  template:
    metadata:
      labels:
        app: toka
    spec:
      containers:
      - name: toka-orchestration
        image: toka:latest
        ports:
        - containerPort: 8080
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: toka-secrets
              key: anthropic-api-key
        - name: RUST_LOG
          value: "info"
        - name: DATABASE_URL
          value: "postgresql://toka:password@postgres:5432/toka"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2"
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: data
          mountPath: /app/data
      volumes:
      - name: config
        configMap:
          name: toka-config
      - name: data
        persistentVolumeClaim:
          claimName: toka-data
---
apiVersion: v1
kind: Service
metadata:
  name: toka-service
spec:
  selector:
    app: toka
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

## Testing and Validation

### Step 5.1: Integration Tests

Create `tests/integration/unified_system.rs`:

```rust
#[tokio::test]
async fn test_unified_runtime_execution() {
    let runtime = RuntimeManager::new(test_kernel()).await.unwrap();
    
    // Test code execution
    let code_result = runtime.execute(ExecutionModel::DynamicCode {
        code_type: CodeType::Python,
        code: "print('Hello from Python')".to_string(),
        sandbox_config: SandboxConfig::default(),
    }).await.unwrap();
    
    assert!(code_result.success);
    assert!(code_result.output.contains("Hello from Python"));
    
    // Test tool execution
    let tool_result = runtime.execute(ExecutionModel::ToolExecution {
        tool_name: "file-reader".to_string(),
        tool_params: test_tool_params(),
        security_context: test_security_context(),
    }).await.unwrap();
    
    assert!(tool_result.success);
}

#[tokio::test]
async fn test_tool_registry_discovery() {
    let registry = toka_tools::initialize_unified_tools().await.unwrap();
    let tools = registry.list_tools().await;
    
    // Should include native tools
    assert!(tools.contains(&"file-reader".to_string()));
    assert!(tools.contains(&"date-validator".to_string()));
    
    // Should include external tools
    assert!(tools.contains(&"setup-toka-testing".to_string()));
    assert!(tools.contains(&"validate-env".to_string()));
}
```

### Step 5.2: Deployment Test

```bash
# Build unified container
docker build -f Dockerfile.unified -t toka:latest .

# Test container startup
docker run -d --name toka-test \
  -p 8080:8080 \
  -e ANTHROPIC_API_KEY=test-key \
  toka:latest

# Test tool execution
docker exec toka-test toka-cli tools list
docker exec toka-test toka-cli tools execute file-reader --path /app/README.md

# Test orchestration
curl http://localhost:8080/health
curl http://localhost:8080/status

# Cleanup
docker stop toka-test
docker rm toka-test
```

## Success Checklist

- [ ] **Runtime Consolidation**
  - [ ] `toka-agent-runtime` merged into `toka-runtime`
  - [ ] Unified `ExecutionModel` enum implemented
  - [ ] Circular dependencies eliminated
  - [ ] All tests passing

- [ ] **Tool System Unification**
  - [ ] `UnifiedToolRegistry` implemented
  - [ ] All scripts have tool manifests
  - [ ] External tool execution working
  - [ ] Security validation implemented

- [ ] **Core Crate Consolidation**
  - [ ] Non-essential crates removed/merged
  - [ ] Security crates consolidated
  - [ ] Clean dependency graph

- [ ] **Deployment Readiness**
  - [ ] Unified CLI implemented
  - [ ] Container builds successfully
  - [ ] Kubernetes deployment works
  - [ ] Tool injection at runtime verified

This implementation guide provides the specific steps to achieve a clean, consolidated Toka architecture ready for production deployment.