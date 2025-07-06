# Toka OS

[![Rust](https://github.com/ScrappyAI/toka/workflows/Rust/badge.svg)](https://github.com/ScrappyAI/toka/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**Toka OS** is a deterministic, capability-based operating system for agentic AI systems, built in Rust with security and reliability at its core.

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ with `cargo`
- Git for version control

### Basic Setup

```bash
# Clone the repository
git clone https://github.com/ScrappyAI/toka.git
cd toka

# Validate build system
./scripts/validate-build-system.sh

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace
```

## 🤖 Multi-Agent Orchestration

Toka OS supports true multi-agent orchestration with LLM-powered intelligence. See the [v0.3.0 Enhancement Roadmap](docs/proposals/2025-07-04_v0_3_enhancement_roadmap.md) for current development status.

### LLM Integration Setup

To enable AI-powered agent orchestration, configure LLM credentials:

```bash
# For Anthropic Claude (recommended)
export ANTHROPIC_API_KEY="your-anthropic-api-key"
export LLM_PROVIDER="anthropic"

# For OpenAI GPT-4
export OPENAI_API_KEY="your-openai-api-key"  
export LLM_PROVIDER="openai"

# Optional configuration
export LLM_MODEL="claude-3-5-sonnet-20241022"  # or "gpt-4"
export LLM_RATE_LIMIT="60"  # requests per minute
export LLM_TIMEOUT="30"     # timeout in seconds
```

### Agent Orchestration Example

```rust
use toka_orchestration::{OrchestrationEngine, OrchestrationConfig};
use toka_llm_gateway::{Config as LlmConfig, LlmGateway};
use toka_runtime::{Runtime, RuntimeConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load agent configurations
    let config = OrchestrationConfig::from_directory("agents/v0.3.0/workstreams")?;
    
    // Initialize Toka runtime
    let runtime = Arc::new(Runtime::new(
        RuntimeConfig::default(),
        Arc::new(your_token_validator),
    ).await?);
    
    // Configure LLM integration
    let llm_config = LlmConfig::from_env()?;
    let llm_gateway = Arc::new(LlmGateway::new(llm_config).await?);
    
    // Create orchestration engine with LLM intelligence
    let engine = Arc::new(
        OrchestrationEngine::new(config, runtime)
            .await?
            .with_llm_gateway(llm_gateway)
    );
    
    // Start multi-agent orchestration
    let session = engine.start_orchestration().await?;
    session.wait_for_completion().await?;
    
    Ok(())
}
```

## 📊 Current Development Status

### v0.3.0 Enhancement Roadmap Progress

- ✅ **Build System Stabilization** - COMPLETED
- ✅ **Testing Infrastructure Expansion** - COMPLETED  
- ✅ **Kernel Event Model Enhancement** - COMPLETED
- 🚀 **Storage Layer Advancement** - READY TO SPAWN
- ⏳ **Security Framework Extension** - QUEUED
- ⏳ **Performance & Observability Foundation** - QUEUED

### Recent Agent Additions

- 🔧 **GitHub CI/CD Issues Resolution Agent** - Fixes workflow problems and branch protection
- 🤖 **LLM Credentials Setup Agent** - Enables secure AI-powered orchestration
- 📊 **Storage Layer Advancement Agent** - Next priority for v0.3.0 roadmap

## 🔧 Unified Tool System

**New in v1.0**: All tools have been consolidated into a unified, composable system with hot-swappable execution capabilities.

### Architecture Overview

The unified tool system provides:
- **Composable Tools**: All tools are discoverable and executable through a unified interface
- **Hot-Swappable Execution**: Tools can be updated without agent restart
- **Sandbox Security**: Capability-based security with resource limits and network restrictions
- **YAML Configuration**: Consistent configuration format across all tools and agents

### Directory Structure

```
tools/
├── README.md              # Unified tool system overview
├── manifests/            # YAML tool manifests
│   ├── README.md           # Manifest documentation
│   ├── control-flow-analyzer.yaml
│   ├── dependency-visualizer.yaml
│   ├── raft-analyzer.yaml
│   ├── build-validator.yaml
│   ├── date-validator.yaml
│   └── date-template-processor.yaml
├── agents/               # Agent configurations (YAML)
│   └── date-enforcement-agent.yaml
├── analysis/            # Code and system analysis tools
│   ├── control_flow_graph_visualizer.py
│   ├── dependency_graph_visualizer.py
│   ├── raft_analysis.py
│   ├── monitor_raft_development.py
│   └── cfg_demo.py
├── system/             # System management tools
│   ├── validate-build-system.sh
│   ├── setup-env.sh
│   ├── setup-parallel-workstreams.sh
│   └── test-toka-system.sh
├── validation/         # Quality assurance and validation
│   ├── validate_dates.py
│   └── insert_date.sh
└── runtime/           # Runtime integration tools
```

### Integration with Agent Runtime

The unified tool system integrates seamlessly with the Toka agent runtime through:

1. **RuntimeToolRegistry**: Automatic tool discovery and manifest loading
2. **Capability Validation**: Tools validate agent capabilities before execution
3. **Event Hooks**: Runtime event processing for agent lifecycle and task completion
4. **Protocol Support**: MCP and A2A protocol integration

### Usage Example

```rust
use toka_tools::{RuntimeToolRegistry, RuntimeContext};

// Initialize the unified tool system
let registry = RuntimeToolRegistry::new("tools").await?;

// Create execution context
let context = RuntimeContext {
    agent_id: "analysis-agent-001".to_string(),
    agent_type: "analysis".to_string(),
    capabilities: vec!["filesystem-read", "code-analysis", "visualization"],
    // ... other fields
};

// Execute a tool with runtime integration
let result = registry.execute_tool_runtime(
    "control-flow-analyzer",
    &params,
    &context.capabilities,
    &context,
).await?;
```

### Tool Categories

- **Analysis**: Code analysis, dependency graphing, control flow visualization
- **System**: Build validation, environment setup, system management
- **Validation**: Date validation, compliance checking, quality assurance
- **Runtime**: Agent lifecycle, orchestration, runtime management

## 🔒 Security Features

- **Capability-based security**: Fine-grained permission system
- **Deterministic execution**: Reproducible agent behavior
- **Secure credential management**: Environment-based secrets with auto-rotation
- **Audit logging**: Comprehensive activity tracking
- **Sandboxed execution**: Resource isolation and limits
- **Memory safety**: Written in Rust with `#![forbid(unsafe_code)]`

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace --all-features

# Run integration tests
cargo test --test integration_*

# Run property-based tests  
cargo test --test property_*

# Generate coverage report
cargo llvm-cov --workspace --all-features --html
```

## 📖 Documentation

### Project Organization

All documentation has been organized into the `docs/` directory:

- **[docs/guides/](docs/guides/)**: Setup and user guides
  - [RAFT Monitoring Setup](docs/guides/RAFT_MONITORING_README.md)
  - [Testing Setup Guide](docs/guides/TOKA_TESTING_SETUP_GUIDE.md)
  - [Quick Start Testing](docs/guides/QUICK_START_TESTING.md)
  - [RAFT Monitoring Deployment](docs/guides/RAFT_MONITORING_DEPLOYMENT_SUMMARY.md)

- **[docs/reports/](docs/reports/)**: Technical analysis and implementation reports
  - [Unified Tools Integration](docs/reports/UNIFIED_PYTHON_TOOLS_INTEGRATION_README.md)
  - [Implementation Roadmap](docs/reports/IMPLEMENTATION_ROADMAP.md)
  - [Control Flow Analysis](docs/reports/CONTROL_FLOW_SUMMARY.md)
  - [Dependency Analysis](docs/reports/DEPENDENCY_GRAPH_README.md)
  - [Workspace Cleanup Summary](docs/reports/WORKSPACE_CLEANUP_SUMMARY.md)

- **[docs/proposals/](docs/proposals/)**: Architecture proposals and RFCs
- **[docs/protocols/](docs/protocols/)**: Protocol specifications
- **[docs/research/](docs/research/)**: Research documents and analysis

### Generated Visualizations

- **[dependency_graphs/](dependency_graphs/)**: System dependency visualizations
- **[control_flow_graphs/](control_flow_graphs/)**: Control flow analysis reports

### Additional Resources

- [Agent Configuration Guide](agents/v0.3.0/README.md)
- [API Documentation](https://docs.rs/toka-kernel)
- [Docker Setup Guide](README-Docker.md)

## 🛠️ Development

### GitHub Workflows

The project uses sophisticated CI/CD workflows:

- **Workstream CI**: Branch-specific validation for parallel development
- **Integration Testing**: Cross-workstream compatibility validation  
- **Dependency Management**: Automated conflict detection and resolution
- **Security Auditing**: Continuous vulnerability scanning

### Contributing

1. Fork the repository
2. Create a feature branch following workstream conventions
3. Make your changes with comprehensive tests
4. Ensure all CI checks pass
5. Submit a pull request with detailed description

### Workstream Development

Follow the [parallel workstream development strategy](docs/proposals/2025-07-04_v0_3_enhancement_roadmap.md):

1. Each workstream has dedicated feature branches
2. Agent configurations define objectives and tasks
3. Dependencies are resolved automatically
4. Integration testing ensures compatibility

## 🎯 Current Priorities

1. **Fix GitHub CI/CD Issues**: Resolve workflow conditionals and missing scripts
2. **Enable LLM Integration**: Set up secure credentials for AI-powered orchestration  
3. **Storage Layer Enhancement**: Implement WAL generalization and semantic analysis
4. **True Multi-Agent Orchestration**: Demonstrate coordinated workstream execution

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🤝 Community

- [GitHub Issues](https://github.com/ScrappyAI/toka/issues)
- [Discussions](https://github.com/ScrappyAI/toka/discussions)
- [Contributing Guide](CONTRIBUTING.md)

---

**Toka OS**: Where deterministic systems meet agentic intelligence. 🤖⚡
