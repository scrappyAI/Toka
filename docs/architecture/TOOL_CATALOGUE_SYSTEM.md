# Toka Tool Catalogue System

**Date**: 2025-07-12  
**Version**: 1.0.0  
**Purpose**: Unified tool metadata catalogue for efficient discovery, filtering, and dynamic tool selection

## Overview

The Toka Tool Catalogue System provides a comprehensive metadata management solution for tool discovery, filtering, and dynamic selection. It supports multiple formats (JSON, YAML, MD, MDC) with a roadmap for .proto schema compatibility, designed for both LLM-to-LLM and human UI consumption.

## Architecture

### Core Components

1. **Tool Metadata Structure** (`crates/toka-tools/src/catalogue.rs`)
   - Unified metadata representation
   - Multi-format support (JSON, YAML, MD, MDC)
   - Extensible design for future .proto integration

2. **Protocol Buffers Schema** (`proto/tool_catalogue.proto`)
   - Type-safe schema definition
   - LLM-to-LLM communication
   - Human UI consumption
   - Future-proof architecture

3. **Catalogue Service** (`crates/toka-tools/src/catalogue.rs`)
   - Dynamic tool discovery
   - Efficient filtering and querying
   - Hot-swappable tool registration
   - Export capabilities

## Metadata Structure

### ToolMetadata

The core metadata structure that represents all tool information:

```rust
pub struct ToolMetadata {
    pub id: String,                           // Unique identifier
    pub name: String,                         // Human-readable name
    pub version: String,                      // Semantic version
    pub description: String,                  // Tool description
    pub category: ToolCategory,               // Classification
    pub required_capabilities: Vec<String>,   // Required capabilities
    pub optional_capabilities: Vec<String>,   // Optional capabilities
    pub security_level: SecurityLevel,        // Security classification
    pub resource_limits: ResourceLimits,      // Execution limits
    pub sandbox_config: SandboxConfig,        // Sandbox configuration
    pub side_effects: SideEffect,             // Side effects classification
    pub transports: Vec<Transport>,           // Transport options
    pub protocol_mappings: Vec<ProtocolMapping>, // External protocols
    pub input_schema: Option<String>,         // Input JSON Schema
    pub output_schema: Option<String>,        // Output JSON Schema
    pub execution_metadata: ExecutionMetadata, // Performance data
    pub discovery_metadata: DiscoveryMetadata, // Discovery info
    pub extensions: HashMap<String, Value>,   // Arbitrary extensions
    pub last_modified: DateTime<Utc>,         // Modification timestamp
    pub file_path: PathBuf,                   // File location
    pub checksum: String,                     // Content hash
}
```

### Tool Categories

```rust
pub enum ToolCategory {
    FileSystem,      // File operations
    Validation,      // Testing and validation
    Build,          // Compilation and building
    Security,       // Authentication and security
    Network,        // Network operations
    Database,       // Database operations
    Analysis,       // Code analysis
    System,         // System administration
    Development,    // Development utilities
    Monitoring,     // Observability
    Documentation,  // Documentation tools
    Workflow,       // Orchestration
    Integration,    // External integrations
    Other,          // Uncategorized
}
```

### Security Levels

```rust
pub enum SecurityLevel {
    Basic,    // Low-risk operations
    Medium,   // Standard operations
    High,     // Privileged operations
}
```

### Side Effects

```rust
pub enum SideEffect {
    None,        // Pure function
    ReadOnly,    // Only reads data
    Idempotent,  // Safe to retry
    External,    // External operations
    Privileged,  // Elevated access
}
```

## Protocol Buffers Schema

### Key Features

1. **Type Safety**: Strongly typed schema for reliable communication
2. **Multi-format Support**: Compatible with JSON, YAML, MD, MDC
3. **LLM Integration**: Optimized for LLM-to-LLM communication
4. **Human UI**: Designed for human interface consumption
5. **Extensible**: Future-proof with optional fields and extensions

### Service Definition

```protobuf
service ToolCatalogueService {
  rpc ListTools(ListToolsRequest) returns (ListToolsResponse);
  rpc GetTool(GetToolRequest) returns (GetToolResponse);
  rpc FilterTools(FilterToolsRequest) returns (FilterToolsResponse);
  rpc GetStatistics(GetStatisticsRequest) returns (GetStatisticsResponse);
  rpc ExportCatalogue(ExportCatalogueRequest) returns (ExportCatalogueResponse);
  rpc RegisterTool(RegisterToolRequest) returns (RegisterToolResponse);
  rpc UpdateTool(UpdateToolRequest) returns (UpdateToolResponse);
  rpc UnregisterTool(UnregisterToolRequest) returns (UnregisterToolResponse);
}
```

### Export Formats

```protobuf
enum ExportFormat {
  EXPORT_FORMAT_UNSPECIFIED = 0;
  EXPORT_FORMAT_JSON = 1;
  EXPORT_FORMAT_YAML = 2;
  EXPORT_FORMAT_PROTO = 3;
  EXPORT_FORMAT_MARKDOWN = 4;
}
```

## Implementation

### Catalogue Service

The `ToolCatalogue` provides efficient tool management:

```rust
pub struct ToolCatalogue {
    tools: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    categories: Arc<RwLock<HashMap<ToolCategory, HashSet<String>>>>,
    capabilities: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    tags: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    workspace_root: PathBuf,
    last_scan: Arc<RwLock<Option<DateTime<Utc>>>>,
}
```

### Key Operations

1. **Workspace Scanning**: Auto-discovery of tool manifests
2. **Dynamic Filtering**: Efficient querying by multiple criteria
3. **Hot-swapping**: Runtime tool registration and updates
4. **Multi-format Export**: JSON, YAML, Markdown support

### Filtering System

```rust
pub struct ToolFilter {
    pub categories: Option<Vec<ToolCategory>>,
    pub required_capabilities: Option<Vec<String>>,
    pub security_levels: Option<Vec<SecurityLevel>>,
    pub transport_types: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub execution_metadata: Option<ExecutionFilter>,
    pub search_query: Option<String>,
    pub limit: Option<usize>,
    pub include_deprecated: bool,
}
```

## Usage Examples

### Basic Catalogue Operations

```rust
use toka_tools::catalogue::{ToolCatalogue, ToolFilter, ToolCategory, SecurityLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create catalogue
    let mut catalogue = ToolCatalogue::new();
    
    // Scan workspace for tools
    catalogue.scan_workspace().await?;
    
    // Get all tools
    let all_tools = catalogue.get_all_tools().await;
    println!("Found {} tools", all_tools.len());
    
    // Filter by category
    let validation_tools = catalogue.filter_tools(
        ToolFilter::new().with_category(ToolCategory::Validation)
    ).await?;
    
    // Filter by capability
    let filesystem_tools = catalogue.filter_tools(
        ToolFilter::new().with_capability("filesystem-read")
    ).await?;
    
    // Filter by security level
    let safe_tools = catalogue.filter_tools(
        ToolFilter::new().with_security_level(SecurityLevel::Basic)
    ).await?;
    
    // Search tools
    let search_results = catalogue.filter_tools(
        ToolFilter::new().with_search("validation")
    ).await?;
    
    // Export for LLM consumption
    let json_export = catalogue.export_json().await?;
    
    Ok(())
}
```

### Advanced Filtering

```rust
// Complex filter with multiple criteria
let advanced_filter = ToolFilter::new()
    .with_categories(vec![ToolCategory::Validation, ToolCategory::Security])
    .with_required_capabilities(vec!["filesystem-read".to_string(), "validation".to_string()])
    .with_security_levels(vec![SecurityLevel::Basic, SecurityLevel::Medium])
    .with_tags(vec!["automated".to_string(), "testing".to_string()])
    .with_search("date")
    .with_limit(10);

let filtered_tools = catalogue.filter_tools(advanced_filter).await?;
```

### Tool Registration

```rust
// Register a new tool
let new_tool = ToolMetadata {
    id: "my-custom-tool".to_string(),
    name: "My Custom Tool".to_string(),
    version: "1.0.0".to_string(),
    description: "A custom tool for specific tasks".to_string(),
    category: ToolCategory::Development,
    required_capabilities: vec!["filesystem-read".to_string()],
    optional_capabilities: vec!["filesystem-write".to_string()],
    security_level: SecurityLevel::Medium,
    // ... other fields
};

catalogue.add_tool(new_tool).await;
```

## Integration with Existing Systems

### Agent Runtime Integration

```rust
use toka_agent_runtime::{AgentRuntime, ToolRegistry};

// Integrate with agent runtime
let mut runtime = AgentRuntime::new();
let catalogue = ToolCatalogue::new();

// Scan for tools
catalogue.scan_workspace().await?;

// Register tools with runtime
for tool in catalogue.get_all_tools().await {
    runtime.register_tool(tool.id, tool).await?;
}
```

### LLM Integration

```rust
// Export catalogue for LLM consumption
let catalogue_json = catalogue.export_json().await?;

// Send to LLM for tool selection
let llm_response = llm_client.select_tools(&catalogue_json, &task_description).await?;

// Parse LLM response and execute selected tools
for tool_id in llm_response.selected_tools {
    let tool = catalogue.get_tool(&tool_id).await?;
    runtime.execute_tool(&tool.id, &parameters).await?;
}
```

## Performance Considerations

### Indexing Strategy

1. **Category Index**: Fast filtering by tool category
2. **Capability Index**: Efficient capability-based queries
3. **Tag Index**: Quick tag-based filtering
4. **Search Index**: Full-text search capabilities

### Caching

- **Manifest Cache**: Cached tool manifests for fast access
- **Filter Cache**: Cached filter results for repeated queries
- **Export Cache**: Cached exports for LLM consumption

### Memory Management

- **Lazy Loading**: Load tool metadata on demand
- **LRU Eviction**: Automatic cleanup of unused entries
- **Resource Limits**: Bounded memory usage

## Security Model

### Capability-Based Access Control

```rust
pub struct CapabilityValidator {
    pub required_capabilities: Vec<String>,
    pub granted_capabilities: Vec<String>,
    pub security_level: SecurityLevel,
}
```

### Sandbox Configuration

```rust
pub struct SandboxConfig {
    pub allow_network: bool,
    pub readonly_paths: Vec<String>,
    pub writable_paths: Vec<String>,
    pub network_restrictions: Vec<String>,
}
```

## Roadmap

### Phase 1: Core Implementation ✅
- [x] Metadata structure definition
- [x] Basic catalogue operations
- [x] Filtering system
- [x] Export capabilities

### Phase 2: Protocol Buffers Integration 🚧
- [ ] .proto schema implementation
- [ ] gRPC service implementation
- [ ] Multi-format serialization
- [ ] LLM integration layer

### Phase 3: Advanced Features 📋
- [ ] Real-time updates
- [ ] Distributed catalogue
- [ ] Advanced analytics
- [ ] Machine learning integration

### Phase 4: Production Features 📋
- [ ] High availability
- [ ] Performance optimization
- [ ] Security hardening
- [ ] Monitoring and observability

## Best Practices

### Tool Development

1. **Metadata Completeness**: Provide comprehensive metadata
2. **Capability Declaration**: Declare all required capabilities
3. **Security Classification**: Assign appropriate security levels
4. **Documentation**: Include clear descriptions and examples

### Catalogue Usage

1. **Efficient Filtering**: Use specific filters for better performance
2. **Caching**: Cache frequently accessed data
3. **Validation**: Validate tool metadata before registration
4. **Monitoring**: Track catalogue usage and performance

### Integration

1. **LLM Integration**: Export in LLM-friendly formats
2. **Human UI**: Provide human-readable exports
3. **API Design**: Follow RESTful principles
4. **Error Handling**: Implement comprehensive error handling

## Troubleshooting

### Common Issues

1. **Tool Discovery Failures**
   - Check file permissions
   - Verify manifest format
   - Review discovery patterns

2. **Filter Performance Issues**
   - Optimize filter criteria
   - Use specific indexes
   - Implement caching

3. **Export Format Issues**
   - Validate schema compliance
   - Check encoding settings
   - Verify format support

### Debugging

```rust
// Enable debug logging
tracing::subscriber::set_global_default(
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish()
)?;

// Debug catalogue operations
let catalogue = ToolCatalogue::new();
catalogue.scan_workspace().await?;

// Get statistics for debugging
let stats = catalogue.get_statistics().await;
println!("Catalogue statistics: {:?}", stats);
```

## Conclusion

The Toka Tool Catalogue System provides a robust, scalable solution for tool metadata management. With its unified metadata structure, efficient filtering capabilities, and multi-format support, it enables dynamic tool selection and seamless integration with LLM systems and human interfaces.

The .proto schema roadmap ensures future compatibility and type safety, while the current implementation provides immediate value for tool discovery and management in the Toka ecosystem. 