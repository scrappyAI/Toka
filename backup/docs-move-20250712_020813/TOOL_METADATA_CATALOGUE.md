# Toka Tool Metadata Catalogue

**Date**: 2025-07-12  
**Version**: 1.0.0  
**Purpose**: Comprehensive catalogue of all available tools, scripts, and capabilities for agent discovery and usage

## Overview

This catalogue provides a centralized registry of all tools, scripts, and capabilities available in the Toka workspace, enabling efficient discovery and usage by agents and automated systems.

## Core Tool Categories

### 1. Built-in Rust Tools (`toka-tools` crate)

#### File Operations
- **file-reader**: Read file contents with security validation
  - Capabilities: `filesystem-read`
  - Parameters: `path` (string)
  - Security: Workspace-scoped access only

- **file-writer**: Write content to files with validation
  - Capabilities: `filesystem-write`
  - Parameters: `path` (string), `content` (string)
  - Security: Workspace-scoped access only

- **file-lister**: List directory contents
  - Capabilities: `filesystem-read`
  - Parameters: `path` (string), `recursive` (boolean)
  - Security: Workspace-scoped access only

#### Validation Tools
- **date-validator**: Validate and fix future dates in files
  - Capabilities: `filesystem-read`, `validation`, `filesystem-write` (optional)
  - Parameters: `path` (string), `fix_violations` (boolean), `strict_mode` (boolean)
  - Security: Medium level, 128MB memory limit, 30s timeout
  - Manifest: `crates/toka-tools/manifests/date-validator.yaml`

- **build-validator**: Validate build system configuration
  - Capabilities: `cargo-execution`, `filesystem-read`
  - Parameters: `workspace_path` (string), `check_dependencies` (boolean)
  - Security: Medium level, resource limits applied
  - Manifest: `crates/toka-tools/manifests/build-validator.yaml`

### 2. System Scripts (`scripts/` directory)

#### Setup Scripts
- **setup_toka_testing.sh**: Complete testing environment setup
  - Location: `scripts/setup/setup_toka_testing.sh`
  - Purpose: Initialize development environment, configure LLM providers
  - Capabilities: Environment configuration, dependency management
  - Interactive: Yes (prompts for API keys)

- **setup-docker-environments.sh**: Docker environment configuration
  - Location: `scripts/setup/setup-docker-environments.sh`
  - Purpose: Configure Docker environments for development and production
  - Capabilities: Docker management, environment setup

#### Testing Scripts
- **test_toka_agents.sh**: Comprehensive agent testing
  - Location: `scripts/testing/test_toka_agents.sh`
  - Purpose: Test agent functionality, integration, and performance
  - Capabilities: Agent spawning, task scheduling, system monitoring
  - Modes: start, test, logs, stop, full

- **run_simple_test.sh**: Basic functionality testing
  - Location: `scripts/testing/run_simple_test.sh`
  - Purpose: Quick validation of core functionality
  - Capabilities: Basic system validation

#### Orchestration Scripts
- **start-orchestration.sh**: Start orchestration system
  - Location: `scripts/start-orchestration.sh`
  - Purpose: Launch multi-agent orchestration with LLM integration
  - Capabilities: Agent coordination, LLM integration, resource management
  - Modes: dev, production, cursor-mode

#### Monitoring Scripts
- **toka_interactive.sh**: Interactive CLI interface
  - Location: `toka_interactive.sh`
  - Purpose: Menu-driven interface for agent management
  - Capabilities: Agent spawning, task scheduling, state queries, token generation
  - Features: Stateful operation, SQLite storage, real-time monitoring

### 3. Agent Capabilities System

#### Core Capabilities
- **filesystem-read**: Read files and directories
  - Tools: cat, ls, find, grep, head, tail
  - Security: Workspace-scoped access

- **filesystem-write**: Write and modify files
  - Tools: touch, mkdir, mv, cp, rm, echo
  - Security: Workspace-scoped access with validation

- **cargo-execution**: Rust build system operations
  - Tools: cargo, rustc, rustfmt, clippy
  - Security: Build environment access

- **git-access**: Version control operations
  - Tools: git
  - Security: Repository access with authentication

- **network-access**: Network operations
  - Tools: curl, wget
  - Security: Controlled network access

- **database-access**: Database operations
  - Tools: sqlite3, psql
  - Security: Database connection limits

- **code-execution**: Dynamic code execution
  - Modes: WASM, Python scripts
  - Security: Sandboxed execution

- **tool-registration**: Tool system management
  - Capabilities: Tool discovery, registration, metadata management
  - Security: Administrative privileges required

### 4. LLM Integration Tools

#### Providers
- **Anthropic Claude**: Primary LLM provider
  - Models: claude-3-5-sonnet-20241022
  - Capabilities: Agent coordination, intelligent task planning
  - Configuration: `ANTHROPIC_API_KEY` environment variable

- **OpenAI GPT**: Alternative LLM provider
  - Models: gpt-4, gpt-4-turbo
  - Capabilities: Agent coordination, task planning
  - Configuration: `OPENAI_API_KEY` environment variable

#### Integration Features
- **LLM-guided tool selection**: Intelligent tool selection based on context
- **Context-aware decision making**: Adaptive behavior based on system state
- **Automatic fallback**: Rule-based coordination when LLM unavailable
- **Secure API management**: Encrypted key storage and rotation

### 5. Security and Sandbox System

#### Security Levels
- **Basic**: File processing, basic utilities
  - Memory: 64MB, CPU: 10%, Timeout: 30s
  - Network: Disabled

- **Medium**: System tools, validation
  - Memory: 128MB, CPU: 25%, Timeout: 30s
  - Network: Controlled access

- **High**: Analysis tools, privileged operations
  - Memory: 256MB, CPU: 50%, Timeout: 60s
  - Network: Full access with monitoring

#### Sandbox Features
- **Resource Limits**: Memory, CPU, execution time constraints
- **Capability Validation**: Fine-grained permission system
- **Network Isolation**: Controlled network access
- **Filesystem Isolation**: Workspace-scoped file access

### 6. Development and Testing Tools

#### CLI Tools
- **toka**: Main CLI binary
  - Commands: generate-token, daemon, query-state, spawn-agent, schedule-task
  - Storage: SQLite for persistent state
  - Authentication: JWT token system

#### Configuration Management
- **Environment Files**: `.env`, `config/*.toml`
- **Agent Configurations**: YAML-based agent definitions
- **Docker Compose**: Multi-service orchestration
- **Cursor Integration**: Background agent support

#### Monitoring and Observability
- **Health Checks**: HTTP endpoints for system status
- **Logging**: Structured logging with tracing
- **Metrics**: Performance and resource usage monitoring
- **Real-time Updates**: WebSocket integration for live updates

## Tool Discovery API

### Programmatic Access
```rust
use toka_tools::{ToolRegistry, ToolSystem};

// Initialize tool system
let system = ToolSystem::new().await?;

// List available tools
let tools = system.list_tools().await;

// Get tool metadata
let metadata = system.get_tool_metadata("date-validator").await?;

// Execute tool with parameters
let result = system.execute_tool("date-validator", &params).await?;
```

### CLI Access
```bash
# List available tools
toka list-tools

# Get tool information
toka describe-tool date-validator

# Execute tool
toka execute-tool date-validator --path=/workspace --fix-violations=true
```

### Agent Integration
```yaml
# Agent configuration with tool access
agent:
  name: "validation-agent"
  capabilities:
    - "filesystem-read"
    - "filesystem-write"
    - "validation"
  tools:
    - "date-validator"
    - "build-validator"
    - "file-reader"
```

## Usage Examples

### Date Validation
```bash
# Using the date-validator tool
./target/release/toka execute-tool date-validator \
  --path=/workspace \
  --fix-violations=true \
  --strict-mode=true
```

### Agent Spawning
```bash
# Using the interactive CLI
./toka_interactive.sh
# Select option 3: Spawn Agent
# Enter agent name and configuration
```

### System Testing
```bash
# Run comprehensive tests
./scripts/testing/test_toka_agents.sh full
```

### Orchestration
```bash
# Start orchestration with LLM integration
./scripts/start-orchestration.sh --dev --log-level debug
```

## Integration Guidelines

### For Agents
1. **Capability Declaration**: Declare required capabilities in agent configuration
2. **Tool Discovery**: Use the tool registry for dynamic tool discovery
3. **Security Compliance**: Ensure all tool usage follows security policies
4. **Error Handling**: Implement robust error handling for tool execution

### For Tool Developers
1. **Manifest Creation**: Create YAML manifests for all tools
2. **Capability Mapping**: Map tool functionality to capability requirements
3. **Security Classification**: Assign appropriate security levels
4. **Documentation**: Provide comprehensive tool documentation

### For System Integration
1. **Registry Management**: Use the unified tool registry for all tool access
2. **Security Enforcement**: Implement capability-based access control
3. **Monitoring Integration**: Add observability hooks for tool execution
4. **Performance Optimization**: Implement caching and resource management

## Future Enhancements

### Planned Features
- **Tool Composition**: Ability to chain tools together
- **Dynamic Tool Loading**: Hot-swappable tool registration
- **Advanced Analytics**: Tool usage analytics and optimization
- **Multi-Language Support**: Support for additional programming languages
- **Distributed Execution**: Tool execution across multiple nodes

### Roadmap
- **Q1 2025**: Enhanced tool discovery and composition
- **Q2 2025**: Advanced security features and audit logging
- **Q3 2025**: Distributed tool execution and scaling
- **Q4 2025**: AI-powered tool optimization and recommendation

---

© 2025 Toka Contributors  
Licensed under Apache-2.0