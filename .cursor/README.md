# Cursor Integration

This directory contains the Cursor-specific integration for the Toka universal extension system. The `.cursor` directory now serves as a lightweight pointer to the universal extension located in `.llm`.

## Architecture

### Universal Extension Pointer

The `.cursor` directory is now a thin compatibility layer that points to the universal extension:

```
.cursor/
├── cursor-extension.json     # Cursor extension schema
├── cursor-config.json        # Cursor-specific configuration
└── README.md                # This file
```

### Universal Extension

The actual extension system is located in `.llm`:

```
.llm/
├── extension.json            # Universal extension schema
├── extension-config.json     # Universal extension configuration
├── schemas/                  # All schema definitions
├── rules/                    # Rule definitions
├── tools/                    # Tool specifications
├── agents/                   # Agent specifications
├── scripts/                  # Utility scripts
└── date-contract.json        # Date enforcement contract
```

## Configuration

### Cursor Extension Schema

The `cursor-extension.json` defines the schema for Cursor-specific extensions:

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

### Cursor Configuration

The `cursor-config.json` contains the actual Cursor configuration:

```json
{
  "name": "toka-cursor-extension",
  "version": "1.0.0",
  "description": "Cursor integration for Toka universal extension system",
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

## Usage

### Universal Extension Loader

The universal extension loader automatically detects and loads the appropriate extension:

```bash
# Get extension information
python .llm/scripts/universal-loader.py --command info

# Validate resources
python .llm/scripts/universal-loader.py --command validate --file my-resource.json

# List all resources
python .llm/scripts/universal-loader.py --command list
```

### Date Enforcement

The date enforcement system ensures deterministic dating:

```bash
# Comprehensive date validation
python .llm/scripts/validate-dates.py --command comprehensive

# Register tools
python .llm/scripts/tool-registry.py register --tool-path .llm/tools/my-tool.json
```

## Migration from Legacy

### What Changed

1. **Moved to Universal Extension**: All schemas, rules, and tools moved to `.llm`
2. **Pointer Architecture**: `.cursor` now points to the universal extension
3. **Agnostic Design**: Extension works with any directory name (`.llm`, `.agent`, etc.)
4. **Date Enforcement**: Added deterministic date validation system

### Backward Compatibility

- Legacy schemas are still supported
- Migration tools handle format conversion
- Version manager supports both formats

## Benefits

### 1. Agnostic Architecture

The extension system is now truly agnostic:
- Works with any extension directory name
- Not tied to Cursor-specific implementation
- Can be used by other IDEs and tools

### 2. Universal Schema System

All resources use the universal envelope schema:
- Consistent structure across all resource types
- Hierarchical tagging system
- Reference-based architecture

### 3. Date Enforcement

Deterministic dating prevents hallucination:
- Canonical date sources (system clock, git)
- Schema contract enforcement
- Automatic placeholder replacement

### 4. Tool Registry

Secure tool management:
- Database-backed tool registry
- Schema validation for all tools
- Audit trail and version tracking

## Development

### Adding New Resources

1. Create resource in appropriate `.llm` directory
2. Follow universal envelope schema
3. Use date placeholders (`{{TODAY}}`)
4. Register with tool registry

### Extending the System

1. Add new schemas to `.llm/schemas/`
2. Create tools in `.llm/tools/`
3. Update extension configuration
4. Validate with date enforcement

## Troubleshooting

### Common Issues

1. **Extension not found**: Check `.llm` directory exists
2. **Schema validation failed**: Ensure resource follows envelope schema
3. **Date validation failed**: Use `{{TODAY}}` placeholders
4. **Tool registration failed**: Check tool schema and dependencies

### Debug Commands

```bash
# Check extension configuration
python .llm/scripts/universal-loader.py --command info --output text

# Validate specific resource
python .llm/scripts/universal-loader.py --command validate --file my-resource.json

# Audit date enforcement
python .llm/scripts/date-enforcer.py audit
```

## Future Enhancements

### 1. Rust Implementation

Port tools to Rust for performance:
```rust
use toka_tools::universal_loader::UniversalExtensionLoader;
```

### 2. Additional Extensions

Support for other IDEs and tools:
- VS Code extension
- JetBrains plugin
- CLI tools

### 3. Cloud Integration

Remote extension management:
- Cloud-based tool registry
- Distributed date validation
- Multi-repo synchronization

## License

This extension system is part of the Toka project and follows the same licensing terms. 