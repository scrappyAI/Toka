# Toka Dependency Analysis Report

Generated on: Sun Jul  6 08:19:55 AM UTC 2025

## Overview

- Total crates analyzed: 23
- Total agent specifications: 4
- Total internal dependencies: 23

## Crate Categories

- core: 3 crates
- general: 7 crates
- llm: 1 crates
- orchestration: 1 crates
- runtime: 2 crates
- storage: 6 crates
- tools: 3 crates

## Crate Details

### raft-core
- Version: 0.2.1
- Category: core
- Description: Core Raft consensus algorithm implementation
- Workspace dependencies: 0
- External dependencies: 11

### raft-storage
- Version: 0.2.1
- Category: storage
- Description: Storage abstraction for Raft consensus algorithm
- Workspace dependencies: 1
- External dependencies: 10

### toka-agent-runtime
- Version: {'workspace': True}
- Category: runtime
- Description: Agent execution runtime for Toka OS - interprets and executes agent configurations with LLM integration
- Workspace dependencies: 5
- External dependencies: 11

### toka-bus-core
- Version: {'workspace': True}
- Category: core
- Description: Core event bus abstraction for Toka OS - lightweight, deterministic messaging.
- Workspace dependencies: 1
- External dependencies: 5

### toka-capability-core
- Version: 0.2.0-alpha
- Category: core
- Description: Core, no-std capability token primitives – Claims struct and capability traits shared by all implementation crates.
- Workspace dependencies: 0
- External dependencies: 2

### toka-capability-delegation
- Version: 0.3.0
- Category: general
- Description: Hierarchical capability delegation primitives for Toka security framework
- Workspace dependencies: 2
- External dependencies: 10

### toka-capability-jwt-hs256
- Version: 0.2.1
- Category: general
- Description: Concrete JWT HS256 implementation for Toka capability tokens
- Workspace dependencies: 1
- External dependencies: 6

### toka-cli
- Version: {'workspace': True}
- Category: tools
- Description: Command-line interface for Toka OS - interact with the agentic operating system.
- Workspace dependencies: 3
- External dependencies: 8

### toka-config-cli
- Version: {'workspace': True}
- Category: tools
- Description: Configuration file management CLI tool - supports YAML, JSON, and TOML formats with validation.
- Workspace dependencies: 0
- External dependencies: 10

### toka-cvm
- Version: 0.2.0-alpha
- Category: general
- Description: Capability Validation Module – host wrapper for verifying capabilities inside WASM guest modules.
- Workspace dependencies: 0
- External dependencies: 2

### toka-key-rotation
- Version: 0.3.0
- Category: general
- Description: Automatic JWT key rotation and management for Toka capability tokens
- Workspace dependencies: 1
- External dependencies: 11

### toka-llm-gateway
- Version: 0.3.0
- Category: llm
- Description: Secure LLM provider gateway with memory-safe configuration
- Workspace dependencies: 1
- External dependencies: 18

### toka-orchestration
- Version: 0.2.1
- Category: orchestration
- Description: Agent orchestration and coordination for Toka OS
- Workspace dependencies: 6
- External dependencies: 13

### toka-performance
- Version: 0.3.0
- Category: general
- Description: Performance monitoring and observability foundation for Toka OS
- Workspace dependencies: 2
- External dependencies: 21

### toka-rate-limiter
- Version: 0.3.0
- Category: general
- Description: Authentication rate limiting middleware for Toka security framework
- Workspace dependencies: 1
- External dependencies: 9

### toka-revocation
- Version: 0.2.0-alpha
- Category: general
- Description: Revocation primitives (RFC 7009) for capability tokens in the Toka platform.
- Workspace dependencies: 0
- External dependencies: 6

### toka-runtime
- Version: {'workspace': True}
- Category: runtime
- Description: Runtime adapter for Toka OS - bridges deterministic kernel with storage and provides configuration management.
- Workspace dependencies: 8
- External dependencies: 6

### toka-store-core
- Version: {'workspace': True}
- Category: storage
- Description: Core storage abstraction for Toka OS - event store traits and helpers without concrete implementations.
- Workspace dependencies: 1
- External dependencies: 10

### toka-store-memory
- Version: {'workspace': True}
- Category: storage
- Description: In-memory storage driver for Toka OS - fast, non-persistent event storage.
- Workspace dependencies: 1
- External dependencies: 6

### toka-store-semantic
- Version: {'workspace': True}
- Category: storage
- Description: Semantic analysis implementation for Toka OS event store - provides concrete plugin registry and engine implementations.
- Workspace dependencies: 2
- External dependencies: 9

### toka-store-sled
- Version: {'workspace': True}
- Category: storage
- Description: Sled-based persistent storage driver for Toka OS - durable, embedded event storage.
- Workspace dependencies: 1
- External dependencies: 5

### toka-store-sqlite
- Version: {'workspace': True}
- Category: storage
- Description: SQLite-based persistent storage driver for Toka OS - reliable, portable event storage.
- Workspace dependencies: 1
- External dependencies: 7

### toka-tools
- Version: 0.1.0
- Category: tools
- Description: 
- Workspace dependencies: 0
- External dependencies: 22

## Agent Specifications

### code-analyst
- Domain: code-analysis
- Priority: high
- Capabilities: code-review, static-analysis, security-scan
- Dependencies: 0

### test-orchestrator
- Domain: testing
- Priority: critical
- Capabilities: test-execution, test-generation, coverage-analysis
- Dependencies: 0

### security-auditor
- Domain: security
- Priority: high
- Capabilities: vulnerability-scan, dependency-audit, security-policy-check
- Dependencies: 0

### performance-optimizer
- Domain: performance
- Priority: medium
- Capabilities: performance-profiling, bottleneck-analysis, optimization-suggestions
- Dependencies: 0

