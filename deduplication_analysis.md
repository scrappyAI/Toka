# Toka Orchestration Deduplication Analysis

## Overview

This document analyzes code duplication between `toka-orchestration-service` and `toka-orchestration` crates and proposes refactoring solutions following the principles from `00-core-baseline.yaml`.

## Key Duplication Areas Identified

### 1. Runtime Initialization Patterns

**Duplicated in:**
- `toka-orchestration-service/src/main.rs` (lines 150-187)
- `toka-orchestration/src/lib.rs` (lines 215-251)
- `toka-orchestration/examples/parallel_orchestration.rs` (lines 30-35)

**Current Issues:**
- Similar patterns for creating `RuntimeManager`, `LlmGateway`, and authentication
- Repeated kernel/event bus initialization
- Inconsistent error handling across implementations

### 2. Configuration Loading Logic

**Duplicated in:**
- `toka-orchestration-service/src/main.rs` (`load_orchestration_config` function)
- `toka-orchestration/src/config.rs` (`OrchestrationConfig::from_directory`)

**Current Issues:**
- Both implement directory-based configuration loading
- Similar YAML parsing and validation logic
- Repeated error context patterns

### 3. Logging Initialization

**Duplicated in:**
- `toka-orchestration-service/src/main.rs` (`init_logging` function)
- Multiple other crates with similar patterns

**Current Issues:**
- Repeated tracing subscriber setup
- Similar log level parsing
- Inconsistent logging configuration

### 4. Error Handling Patterns

**Duplicated in:**
- Multiple files using `with_context` patterns
- Similar error propagation strategies

## Proposed Refactoring Solutions

### Solution 1: Create `toka-orchestration-core` Crate

Create a new crate to house shared orchestration logic:

```
crates/
├── toka-orchestration-core/          # NEW: Shared orchestration logic
│   ├── src/
│   │   ├── lib.rs
│   │   ├── runtime_builder.rs        # Runtime initialization patterns
│   │   ├── config_loader.rs          # Unified config loading
│   │   ├── error.rs                  # Standardized error types
│   │   └── logging.rs                # Logging utilities
├── toka-orchestration/               # Library functionality
└── toka-orchestration-service/       # Service binary
```

### Solution 2: Runtime Builder Pattern

**File: `crates/toka-orchestration-core/src/runtime_builder.rs`**

```rust
//! Unified runtime initialization builder pattern.

use std::sync::Arc;
use anyhow::{Context, Result};
use toka_auth::JwtHs256Validator;
use toka_bus_core::InMemoryBus;
use toka_kernel::{Kernel, WorldState};
use toka_llm_gateway::{Config as LlmConfig, LlmGateway};
use toka_runtime::{RuntimeKernel, RuntimeManager};

/// Builder for creating orchestration runtime components.
#[derive(Default)]
pub struct OrchestrationRuntimeBuilder {
    jwt_secret: Option<String>,
    llm_config: Option<LlmConfig>,
    bus_capacity: Option<usize>,
}

impl OrchestrationRuntimeBuilder {
    /// Create a new runtime builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set JWT secret for authentication.
    pub fn with_jwt_secret(mut self, secret: String) -> Self {
        self.jwt_secret = Some(secret);
        self
    }

    /// Set LLM configuration.
    pub fn with_llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// Set event bus capacity.
    pub fn with_bus_capacity(mut self, capacity: usize) -> Self {
        self.bus_capacity = Some(capacity);
        self
    }

    /// Build the runtime components.
    pub async fn build(self) -> Result<OrchestrationRuntime> {
        // Initialize authentication
        let jwt_secret = self.jwt_secret
            .or_else(|| std::env::var("JWT_SECRET").ok())
            .unwrap_or_else(|| "toka-orchestration-secret-change-in-production".to_string());
        let auth = Arc::new(JwtHs256Validator::new(jwt_secret));

        // Initialize runtime
        let world_state = WorldState::default();
        let event_bus = Arc::new(InMemoryBus::new(self.bus_capacity.unwrap_or(1024)));
        let kernel = Kernel::new(world_state, auth, event_bus);
        let runtime_kernel = RuntimeKernel::new(kernel);
        let runtime = Arc::new(RuntimeManager::new(runtime_kernel).await
            .context("Failed to create runtime manager")?);

        // Initialize LLM gateway
        let llm_gateway = if let Some(config) = self.llm_config {
            Some(Arc::new(LlmGateway::new(config).await
                .context("Failed to create LLM gateway")?))
        } else {
            match LlmConfig::from_env() {
                Ok(config) => Some(Arc::new(LlmGateway::new(config).await?)),
                Err(_) => None,
            }
        };

        Ok(OrchestrationRuntime {
            runtime,
            llm_gateway,
        })
    }
}

/// Runtime components for orchestration.
pub struct OrchestrationRuntime {
    pub runtime: Arc<RuntimeManager>,
    pub llm_gateway: Option<Arc<LlmGateway>>,
}
```

### Solution 3: Unified Configuration Loader

**File: `crates/toka-orchestration-core/src/config_loader.rs`**

```rust
//! Unified configuration loading with validation.

use std::path::Path;
use anyhow::{Context, Result};
use tracing::{info, debug};
use crate::error::OrchestrationError;

/// Unified configuration loader for orchestration.
pub struct ConfigurationLoader;

impl ConfigurationLoader {
    /// Load orchestration configuration from various sources.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<OrchestrationConfig> {
        let path = path.as_ref();
        
        if path.is_file() {
            Self::load_from_file(path)
        } else if path.is_dir() {
            Self::load_from_directory(path)
        } else {
            Err(OrchestrationError::ConfigNotFound(path.to_path_buf()).into())
        }
    }

    /// Load from a single configuration file.
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<OrchestrationConfig> {
        let path = path.as_ref();
        debug!("Loading configuration from file: {}", path.display());
        
        // Implementation here...
        todo!()
    }

    /// Load from a directory of configuration files.
    fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<OrchestrationConfig> {
        let dir = dir.as_ref();
        info!("Loading configuration from directory: {}", dir.display());
        
        // Use existing OrchestrationConfig::from_directory logic
        OrchestrationConfig::from_directory(dir)
            .with_context(|| format!("Failed to load from directory: {}", dir.display()))
    }
}
```

### Solution 4: Standardized Error Types

**File: `crates/toka-orchestration-core/src/error.rs`**

```rust
//! Standardized error types for orchestration.

use std::path::PathBuf;
use thiserror::Error;

/// Orchestration-specific errors.
#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("Configuration not found: {0}")]
    ConfigNotFound(PathBuf),
    
    #[error("Invalid configuration: {reason}")]
    InvalidConfig { reason: String },
    
    #[error("Runtime initialization failed: {source}")]
    RuntimeInit { 
        #[source] 
        source: Box<dyn std::error::Error + Send + Sync> 
    },
    
    #[error("Agent spawn failed: {agent} - {reason}")]
    AgentSpawnFailed { agent: String, reason: String },
}

/// Result type alias for orchestration operations.
pub type Result<T> = std::result::Result<T, OrchestrationError>;

/// Extension trait for adding orchestration context to errors.
pub trait OrchestrationContext<T> {
    fn with_orchestration_context(self, context: &str) -> anyhow::Result<T>;
}

impl<T, E> OrchestrationContext<T> for std::result::Result<T, E> 
where 
    E: Into<anyhow::Error>,
{
    fn with_orchestration_context(self, context: &str) -> anyhow::Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| context.to_string())
    }
}
```

### Solution 5: Logging Utilities

**File: `crates/toka-orchestration-core/src/logging.rs`**

```rust
//! Unified logging configuration utilities.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging with standard Toka configuration.
pub fn init_orchestration_logging(log_level: &str) -> Result<()> {
    let log_filter = format!(
        "toka_orchestration_service={},toka_orchestration={},toka_runtime={},toka_orchestration_core={}",
        log_level, log_level, log_level, log_level
    );
    
    tracing_subscriber::registry()
        .with(EnvFilter::new(log_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Initialize logging for tests.
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt::try_init();
}
```

## Refactoring Implementation Plan

### Phase 1: Create Core Crate
1. Create `toka-orchestration-core` crate
2. Move shared types and utilities
3. Implement builder patterns

### Phase 2: Update Library Crate
1. Refactor `toka-orchestration` to use core utilities
2. Remove duplicated initialization logic
3. Update configuration loading

### Phase 3: Update Service Crate
1. Refactor `toka-orchestration-service` to use builders
2. Simplify main.rs using core utilities
3. Remove duplicated functions

### Phase 4: Update Dependencies
1. Update all dependent crates
2. Update examples to use new patterns
3. Update documentation

## Benefits of This Refactoring

### 1. Reduced Duplication
- Single source of truth for runtime initialization
- Unified configuration loading patterns
- Consistent error handling

### 2. Improved Maintainability
- Centralized orchestration logic
- Easier to test and validate
- Clear separation of concerns

### 3. Better Developer Experience
- Consistent APIs across crates
- Simplified setup for new services
- Reusable patterns

### 4. Enhanced Security
- Centralized security configuration
- Consistent validation patterns
- Easier security auditing

## Files to Modify

### New Files
- `crates/toka-orchestration-core/Cargo.toml`
- `crates/toka-orchestration-core/src/lib.rs`
- `crates/toka-orchestration-core/src/runtime_builder.rs`
- `crates/toka-orchestration-core/src/config_loader.rs`
- `crates/toka-orchestration-core/src/error.rs`
- `crates/toka-orchestration-core/src/logging.rs`

### Modified Files
- `crates/toka-orchestration-service/src/main.rs`
- `crates/toka-orchestration-service/Cargo.toml`
- `crates/toka-orchestration/src/lib.rs`
- `crates/toka-orchestration/Cargo.toml`
- `crates/toka-orchestration/examples/parallel_orchestration.rs`

## Next Steps

1. **Review and Approve**: Review this plan with the team
2. **Create Core Crate**: Start with the foundation utilities
3. **Iterative Migration**: Migrate one module at a time
4. **Testing**: Ensure all functionality works with new structure
5. **Documentation**: Update guides and examples

This refactoring aligns with the core baseline principles of:
- **Simplicity Over Complexity**: Consolidating related functionality
- **Clear Documentation**: Each component has a clear purpose  
- **Thoughtful Craftsmanship**: Eliminating accidental complexity
- **Minimalism in Dependencies**: Reducing duplication across crates