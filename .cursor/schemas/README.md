# Universal Envelope-Based Schema System

This directory contains the formal schemas and automation tools for all Toka resources using a universal envelope-based architecture. The system ensures consistency, automatic versioning, and validation across all configuration files while providing maximum flexibility and future-proofing.

## 🚀 Overview

The envelope-based schema system provides:
- **Universal envelope** that wraps ALL resource types (agents, rules, tools, plans, events)
- **Future-proof design** with hierarchical tags instead of rigid enums
- **Automatic versioning** based on semantic changes
- **Validation** against JSON Schema specifications
- **Metadata management** with checksums and timestamps
- **Consistency enforcement** across all configuration files
- **Vendor extensions** through `x-` prefixed fields
- **Schema composition** through reusable `$defs`

## 📁 Directory Structure

```
.cursor/schemas/
├── README.md                       # This documentation
├── resource-envelope-v1.json       # Universal envelope schema (canonical)
├── cursor-rule-schema.json         # Rule-specific schema (extends envelope)
├── agent-spec-schema.json          # Agent-specific schema (extends envelope)
├── plan-schema.json                # Plan-specific schema (extends envelope)
├── event-schema.json               # Event-specific schema (extends envelope)
├── schema-index.json               # Auto-generated schema index
├── schema-helper.py                # Schema management utilities
├── migrate-to-envelope.py          # Migration script for legacy files
└── version-manager.py              # Automatic versioning tool
```

## 🏗️ Universal Envelope Architecture

### Core Concept

Every resource in the Toka system uses the same top-level envelope structure:

```json
{
  "kind": "rule | agent | tool | plan | event | capability | workflow | policy | metric",
  "metadata": {
    "name": "resource-identifier",
    "version": "1.0.0",
    "title": "Human Readable Title",
    "description": "Detailed description",
    "tags": ["hierarchical.tags", "for.categorization"],
    "priority": "critical | high | medium | low"
  },
  "spec": {
    // Kind-specific payload
  },
  "extensions": {
    "x-vendor-field": "vendor-specific data"
  }
}
```

### Key Benefits

1. **Single Top-Level Shape**: All resources use the same envelope structure
2. **Hierarchical Tags**: Replace rigid enums with flexible tags (e.g., `security.auth`, `io.fs.read`)
3. **Reusable Components**: Common `$defs` shared across all schemas
4. **Explicit Extensions**: Vendor fields through `x-` prefix
5. **Future-Proof**: New kinds can be added without schema changes

## 🔧 Schema Files

### resource-envelope-v1.json (Universal)
The canonical envelope schema that defines:
- Universal metadata structure
- Reusable `$defs` for common types
- Extension point definitions
- Tag and capability patterns

### Kind-Specific Schemas
Each resource kind has its own schema that extends the envelope:
- **cursor-rule-schema.json**: IDE rules with guidelines and objectives
- **agent-spec-schema.json**: AI agents with capabilities and tasks
- **plan-schema.json**: Execution plans with steps and rollback
- **event-schema.json**: Runtime events with tracing and DAG pointers

## 🏷️ Hierarchical Tagging System

Replace rigid categorical enums with flexible hierarchical tags:

### Tag Examples
```
# Security domain
security.auth          # Authentication rules
security.encryption    # Encryption requirements
security.audit         # Audit and compliance

# System domain  
system.core           # Core system functionality
system.infrastructure # Infrastructure components
system.monitoring     # Monitoring and metrics

# Integration domain
integration.github    # GitHub integration
integration.ai        # AI service integration
integration.api       # API integrations

# Process domain
process.documentation # Documentation processes
process.testing       # Testing procedures
process.deployment    # Deployment workflows
```

### Benefits
- **Searchable**: Filter by tag prefix (`security.*`)
- **Hierarchical**: Natural organization and grouping
- **Extensible**: New tags without schema updates
- **Compositional**: Multiple tags per resource

## 🔄 Capability Maps

Structured capability definitions replace simple string lists:

```json
{
  "capabilities": {
    "read": ["fs:/src", "net:github.com"],
    "write": ["fs:/tmp", "db:events"],
    "execute": [
      {"tool": "cargo", "args": ["check"]},
      {"tool": "docker", "args": ["build"]}
    ],
    "network": ["github.com", "api.openai.com"]
  }
}
```

Benefits:
- **POSIX/RBAC Compatible**: Maps to standard permission systems
- **Composable**: Runtime can compute unions/intersections
- **Structured**: Tool execution with args and environment

## 🔗 Rule References

Behavioral directives are now rule references:

```json
{
  "rules": [
    {
      "rule_id": "https://example.com/rules/security-baseline",
      "version": "v1.2.0",
      "conditions": {"environment": "production"}
    }
  ]
}
```

Benefits:
- **Composition**: Rules live independently, agents import them
- **Versioning**: Specific rule versions
- **Conditional**: Context-specific rule application

## 🔄 Automatic Versioning System

The enhanced version manager handles envelope-based resources:

### Installation
```bash
# Required dependencies already installed
pip install pyyaml jsonschema semantic-version
```

### Usage

#### Update All Files
```bash
# Update all envelope-based resources with automatic versioning
python .cursor/version-manager.py --all

# Update only rules (migrated to envelope format)
python .cursor/version-manager.py --rules
```

#### Migration from Legacy Format
```bash
# Migrate legacy files to envelope format
python .cursor/schemas/migrate-to-envelope.py --all

# Migrate specific file
python .cursor/schemas/migrate-to-envelope.py --file path/to/legacy/file.json
```

#### Schema Management
```bash
# Inject GitHub $id fields into all schemas
python .cursor/schemas/schema-helper.py --inject-ids

# Create schema index
python .cursor/schemas/schema-helper.py --create-index

# All schema operations
python .cursor/schemas/schema-helper.py --all
```

### How Versioning Works

The system automatically determines version bumps based on changes:

#### Major Version (X.0.0)
- Migration to envelope format (breaking change)
- Capability or objective removals
- Priority level changes
- Schema-breaking modifications

#### Minor Version (X.Y.0)
- New capabilities added
- New objectives added
- New tags or features
- Backward-compatible enhancements

#### Patch Version (X.Y.Z)
- Documentation updates
- Minor corrections
- Non-breaking clarifications
- Metadata updates

## 📋 Schema Validation

### Envelope Validation

All resources must validate against both:
1. **Universal envelope schema** - Structure and metadata
2. **Kind-specific schema** - Payload validation

#### Example Valid Rule (Envelope Format)
```json
{
  "kind": "rule",
  "metadata": {
    "name": "example-rule",
    "version": "1.0.0",
    "title": "Example Rule", 
    "description": "Example rule demonstrating envelope structure",
    "tags": ["system.core", "quality.baseline"],
    "priority": "high"
  },
  "spec": {
    "always_apply": true,
    "extends": [],
    "objectives": ["Demonstrate envelope structure"],
    "guidelines": {
      "example_section": [
        "Follow envelope pattern for all resources",
        "Use hierarchical tags for categorization"
      ]
    }
  }
}
```

### Tag Validation

Tags must follow the pattern: `^[a-z0-9][a-z0-9-\.]*[a-z0-9]$`

Valid examples:
- `system.core`
- `security.auth.oauth2`
- `integration.github-api`
- `process.ci-cd`

## 🛠️ Development Workflow

### Creating New Resources

1. **Choose the appropriate kind**:
   ```bash
   # Available kinds: agent, rule, tool, plan, event, capability, workflow, policy, metric
   ```

2. **Create with envelope structure**:
   ```json
   {
     "kind": "rule",
     "metadata": {
       "name": "new-rule",
       "version": "1.0.0",
       "title": "New Rule",
       "description": "Description of the new rule",
       "tags": ["appropriate.category"],
       "priority": "medium"
     },
     "spec": {
       // Kind-specific fields
     }
   }
   ```

3. **Validate and version**:
   ```bash
   python .cursor/version-manager.py --file path/to/new/resource.json
   ```

### Updating Existing Resources

1. **Make your changes** to the resource
2. **Run the version manager** to auto-version:
   ```bash
   python .cursor/version-manager.py --file path/to/changed/resource.json
   ```
3. **Review the version bump** and changelog
4. **Commit the changes** with appropriate commit message

## 📊 Future Schema Extensions

The envelope architecture supports future resource types:

### Planned Extensions
- **tool-schema.json**: External tool definitions
- **capability-schema.json**: Capability definitions and policies  
- **workflow-schema.json**: Multi-step workflow definitions
- **policy-schema.json**: Security and governance policies
- **metric-schema.json**: Monitoring and measurement definitions

### Extension Points
- **Vendor Extensions**: `x-` prefixed fields for vendor-specific data
- **New Kinds**: Add to envelope `kind` enum
- **Tag Hierarchies**: Extend tag patterns for new domains
- **Capability Types**: New capability map structures

## 🔍 Validation and Quality Assurance

### Pre-commit Validation

Add to your git hooks:
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating envelope-based resources..."
python .cursor/version-manager.py --all --validate-only

if [ $? -ne 0 ]; then
    echo "Resource validation failed!"
    exit 1
fi
```

### CI/CD Integration

Add to your CI pipeline:
```yaml
# .github/workflows/validate-resources.yml
name: Validate Resources
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: pip install pyyaml jsonschema semantic-version
      - name: Validate resources
        run: python .cursor/version-manager.py --all --validate-only
```

## 🎯 Best Practices

### Resource Creation
- **Use descriptive names** in kebab-case format
- **Choose appropriate tags** with proper hierarchy
- **Set realistic priorities** based on importance
- **Follow envelope structure** consistently
- **Document vendor extensions** clearly

### Tag Strategy
- **Start specific, generalize up**: `security.auth.oauth2` → `security.auth` → `security`
- **Use domain prefixes**: `system.*`, `security.*`, `integration.*`
- **Avoid deep nesting**: Maximum 3-4 levels
- **Be consistent**: Same patterns across resources

### Version Management
- **Run validation regularly** during development
- **Review version bumps** before committing
- **Use semantic commit messages** that reflect changes
- **Update related documentation** when schemas change

## 🚨 Migration from Legacy Format

### Automatic Migration

All legacy rule files have been automatically migrated to envelope format:

```bash
# Migration was completed with:
python .cursor/schemas/migrate-to-envelope.py --all
```

### Migration Changes

- **Structure**: Wrapped in universal envelope
- **Categories**: Converted to hierarchical tags
- **Priorities**: Numeric → string conversion
- **Metadata**: Enhanced with additional fields
- **Versioning**: Major version bump (v2.0.0)

### Backward Compatibility

The version manager maintains backward compatibility for any remaining legacy files while strongly encouraging migration to the envelope format.

---

This universal envelope-based schema system provides maximum flexibility while maintaining strict validation and consistency. It enables the Toka system to evolve gracefully while ensuring deterministic behavior across all runtime environments. 