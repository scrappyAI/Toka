# API Documentation

> **Category**: API Documentation  
> **Location**: `docs/api/`  
> **Status**: Stable

This directory contains API reference guides, integration documentation, and SDK documentation for the Toka OS.

## 📋 Quick Navigation

- [**Core APIs**](#core-apis) - Core system APIs
- [**Agent APIs**](#agent-apis) - Agent orchestration APIs
- [**Integration APIs**](#integration-apis) - External system integration
- [**SDK Documentation**](#sdk-documentation) - Client libraries and SDKs
- [**Protocols**](#protocols) - Communication protocols

## 🔧 Core APIs

### Runtime APIs
- **Runtime API** - Core runtime management
- **Capability API** - Security and permission management
- **Event API** - Event processing and handling
- **State API** - Persistent state management

### Tool APIs
- **Tool Registry API** - Tool discovery and management
- **Tool Execution API** - Tool execution and monitoring
- **Capability Validation API** - Security validation

## 🤖 Agent APIs

### Orchestration APIs
- **Agent Management API** - Agent lifecycle management
- **Task Distribution API** - Workload distribution
- **Inter-agent Communication API** - Message passing
- **Capability Management API** - Permission handling

### LLM Integration APIs
- **Model Context Protocol (MCP)** - LLM communication
- **Prompt Management API** - Prompt engineering
- **Response Processing API** - LLM output handling

## 🔗 Integration APIs

### External System APIs
- **GitHub Integration API** - Repository management
- **Docker Integration API** - Container orchestration
- **Monitoring Integration API** - Metrics and logging
- **Database Integration API** - Data persistence

### Protocol APIs
- **REST API** - HTTP-based communication
- **WebSocket API** - Real-time communication
- **gRPC API** - High-performance RPC
- **GraphQL API** - Flexible data querying

## 📚 SDK Documentation

### Rust SDK
- **Core SDK** - Main Rust client library
- **Agent SDK** - Agent development toolkit
- **Tool SDK** - Tool development framework
- **Runtime SDK** - Runtime integration library

### Client Libraries
- **Python SDK** - Python client library
- **JavaScript SDK** - Node.js client library
- **Go SDK** - Go client library
- **Java SDK** - Java client library

## 📡 Protocols

### Communication Protocols
- **MCP (Model Context Protocol)** - LLM integration standard
- **A2A (Agent-to-Agent)** - Inter-agent communication
- **REST** - HTTP-based API communication
- **WebSocket** - Real-time bidirectional communication

### Data Formats
- **JSON** - Primary data exchange format
- **YAML** - Configuration file format
- **Protocol Buffers** - High-performance serialization
- **MessagePack** - Compact binary format

## 🔗 Related Documentation

- [Architecture](../architecture/) - System design
- [Development](../development/) - Development guides
- [Operations](../operations/) - Deployment guides

## 🚨 Quick Reference

### API Endpoints
```bash
# Health check
GET /health

# Agent status
GET /agents

# Task submission
POST /tasks

# Capability validation
POST /capabilities/validate
```

### Authentication
```bash
# Bearer token
Authorization: Bearer <capability-token>

# API key
X-API-Key: <api-key>
```

### Error Handling
```json
{
  "error": {
    "code": "CAPABILITY_DENIED",
    "message": "Insufficient capabilities",
    "details": {
      "required": ["filesystem-write"],
      "granted": ["filesystem-read"]
    }
  }
}
```

---

*This API documentation is maintained as part of the Toka project's commitment to clear, accurate, and well-organized API information.* 