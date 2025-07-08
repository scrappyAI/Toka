# Universal Extension System

This directory contains the universal extension system that provides agnostic schema-driven development capabilities. The system is designed to work with any extension directory (`.llm`, `.agent`, `.cursor`, etc.) and provides a unified approach to managing schemas, rules, tools, and agents.

## Architecture Overview

### Universal Extension Structure

```
.llm/                          # Universal extension root
├── extension.json             # Extension schema definition
├── extension-config.json      # Extension configuration
├── schemas/                   # All schema definitions
│   ├── resource-envelope-v1.json  # Universal envelope schema
│   ├── agent-spec-schema.json     # Agent specifications
│   ├── cursor-rule-schema.json    # Rule definitions
│   ├── plan-schema.json           # Execution plans
│   ├── event-schema.json          # Runtime events
│   └── schema-index.json          # Schema registry
├── rules/                     # Rule definitions
│   ├── 00-core-baseline.json
│   ├── 10-security-base.json
│   └── ...
├── tools/                     # Tool specifications
│   ├── schema-validator.json
│   ├── version-manager.json
│   └── ...
├── agents/                    # Agent specifications
│   └── tool-builder-agent.json
├── scripts/                   # Utility scripts
│   ├── universal-loader.py
│   ├── version-manager.py
│   └── schema-helper.py
└── version_db.json           # Version tracking database
```

### Cursor Integration

The `.cursor` directory now serves as a lightweight pointer to the universal extension:

```
.cursor/
├── cursor-extension.json     # Cursor extension schema
├── cursor-config.json        # Cursor-specific configuration
└── ...                       # Legacy files (deprecated)
```

## Key Features

### 1. Universal Envelope Schema

All resources follow a consistent envelope structure:

```json
{
  "$schema": "https://github.com/toka-ai/toka/.llm/schemas/resource-envelope-v1.json",
  "$id": "https://github.com/toka-ai/toka/.llm/resource-type/resource-name.json",
  "kind": "agent|rule|tool|plan|event|capability|workflow|policy|metric",
  "metadata": {
    "name": "resource-name",
    "version": "1.0.0",
    "title": "Human-readable title",
    "description": "Detailed description",
    "tags": ["category.subcategory", "feature.type"],
    "priority": "critical|high|medium|low",
    "maintainer": "Team or individual",
    "created": "2024-01-01T00:00:00Z",
    "lastModified": "2024-01-01T00:00:00Z"
  },
  "spec": {
    // Resource-specific specification
  }
}
```

### 2. Hierarchical Tagging System

Replace rigid enums with flexible tag hierarchies:

```json
{
  "tags": [
    "security.auth",           // Security authentication
    "io.fs.read",             // File system read operations
    "development.codegen",     // Code generation
    "system.core"             // Core system functionality
  ]
}
```

### 3. Structured Capability Maps

POSIX and RBAC compatible capability definitions:

```json
{
  "capabilities": {
    "system": {
      "posix": ["read", "write", "execute"],
      "rbac": ["user:create", "role:assign"]
    },
    "io": {
      "fs": ["read", "write", "create", "delete"],
      "network": ["http:get", "http:post"]
    }
  }
}
```

### 4. Reference-Based Architecture

Resources reference other resources instead of embedding:

```json
{
  "rules": [
    {
      "ref": "https://github.com/toka-ai/toka/.llm/rules/00-core-baseline.json",
      "priority": "critical"
    }
  ],
  "tools": [
    {
      "ref": "https://github.com/toka-ai/toka/.llm/tools/schema-validator.json",
      "priority": "high"
    }
  ]
}
```

## Usage

### Universal Extension Loader

The `universal-loader.py` script provides agnostic access to any extension:

```bash
# Get extension information
python .llm/scripts/universal-loader.py --command info

# Validate a resource
python .llm/scripts/universal-loader.py --command validate --file my-resource.json

# List all resources
python .llm/scripts/universal-loader.py --command list

# Load schemas
python .llm/scripts/universal-loader.py --command load
```

### Tool Builder Agent

The tool builder agent provides explicit guidance for creating new tools:

```bash
# Use the tool builder agent to create a new tool
python .llm/scripts/universal-loader.py --command validate --file .llm/agents/tool-builder-agent.json
```

### Version Management

Manage versions and migrations:

```bash
# Validate and bump version
python .llm/scripts/version-manager.py validate my-resource.json
python .llm/scripts/version-manager.py bump my-resource.json minor

# Migrate legacy resources
python .llm/scripts/migrate-to-envelope.py my-legacy-rule.json
```

## Extension Agnosticism

### Multiple Extension Directories

The system supports multiple extension directory names:

- `.llm` - Universal LLM extension (default)
- `.agent` - Agent-specific extension
- `.cursor` - Cursor-specific extension
- `.ai` - General AI extension
- `.schema` - Schema-focused extension

### Cursor Integration

The `.cursor` directory now serves as a lightweight pointer:

```json
{
  "name": "toka-cursor-extension",
  "version": "1.0.0",
  "universalExtension": {
    "path": "../.llm",
    "configFile": "extension-config.json"
  },
  "cursorSpecific": {
    "schemaValidation": true,
    "autoFormat": true,
    "linting": true
  }
}
```

## Migration from Legacy

### From .cursor to .llm

1. **Automatic Migration**: Use the migration script to convert legacy files
2. **Schema Updates**: All schemas now use the universal envelope format
3. **Reference Updates**: Update references to point to new locations
4. **Validation**: Validate all migrated resources

### Backward Compatibility

The system maintains backward compatibility:
- Legacy schemas are still supported
- Migration tools handle format conversion
- Version manager supports both formats

## Future Extensions

### Rust Implementation

The tools can be ported to Rust in the `toka-tools` crate:

```rust
// Future Rust implementation
use toka_tools::universal_loader::UniversalExtensionLoader;

let loader = UniversalExtensionLoader::new(".llm");
let config = loader.load_config()?;
let schemas = loader.load_schemas()?;
```

### Additional Resource Types

The envelope system supports future resource types:
- `capability` - System capabilities
- `workflow` - Process workflows
- `policy` - Governance policies
- `metric` - Performance metrics

## Configuration

### Extension Configuration

```json
{
  "name": "toka-universal-extension",
  "version": "1.0.0",
  "schemaRoot": ".llm/schemas",
  "ruleRoot": ".llm/rules",
  "toolRoot": ".llm/tools",
  "supportedKinds": [
    "agent", "rule", "tool", "plan", "event",
    "capability", "workflow", "policy", "metric"
  ],
  "defaultSchema": "resource-envelope-v1.json"
}
```

### Environment Variables

```bash
export SCHEMA_ROOT=".llm/schemas"
export RULES_ROOT=".llm/rules"
export TOOLS_ROOT=".llm/tools"
export VALIDATION_STRICT="true"
```

## Best Practices

### 1. Schema Design
- Use hierarchical tags for flexibility
- Define reusable `$defs` components
- Include vendor extensions with `x-` prefix
- Maintain backward compatibility

### 2. Resource Organization
- Group related resources by kind
- Use consistent naming conventions
- Include comprehensive metadata
- Reference rather than embed

### 3. Validation
- Validate against schemas before deployment
- Use strict validation in production
- Include deprecation warnings
- Maintain version compatibility

### 4. Extension Development
- Follow the universal envelope format
- Provide clear documentation
- Include migration tools
- Support multiple extension directories

## Troubleshooting

### Common Issues

1. **Schema Not Found**: Ensure schema files are in the correct location
2. **Validation Errors**: Check resource structure against envelope schema
3. **Reference Errors**: Verify referenced resources exist
4. **Version Conflicts**: Use version manager to resolve conflicts

### Debug Commands

```bash
# Check extension configuration
python .llm/scripts/universal-loader.py --command info --output text

# Validate specific resource
python .llm/scripts/universal-loader.py --command validate --file my-resource.json --kind agent

# List all resources
python .llm/scripts/universal-loader.py --command list --output text
```

## Contributing

### Adding New Resources

1. Create resource file following envelope format
2. Add to appropriate directory (rules/, tools/, agents/, etc.)
3. Update schema index if needed
4. Validate against schemas
5. Update documentation

### Extending Schemas

1. Modify schema files in schemas/
2. Update $defs for reusable components
3. Add new resource kinds to supportedKinds
4. Update validation tools
5. Test with existing resources

### Creating Tools

1. Use the tool builder agent for guidance
2. Follow the tool schema specification
3. Include proper capabilities and metadata
4. Add to tools/ directory
5. Update tool registry

## License

This extension system is part of the Toka project and follows the same licensing terms. 