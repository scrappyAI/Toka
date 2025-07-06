# Schema-Based Configuration System

This directory contains the formal schemas and automation tools for cursor rules and agent specifications. The system ensures consistency, automatic versioning, and validation across all configuration files.

## 🚀 Overview

The schema-based system provides:
- **Formal schemas** that ALL implementations MUST follow
- **Automatic versioning** based on semantic changes
- **Validation** against JSON Schema specifications
- **Metadata management** with checksums and timestamps
- **Consistency enforcement** across all configuration files

## 📁 Directory Structure

```
.cursor/schemas/
├── README.md                    # This documentation
├── cursor-rule-schema.yaml      # Schema for cursor rules
├── agent-spec-schema.yaml       # Schema for agent specifications
└── version-manager.py           # Automatic versioning tool
```

## 🏗️ Schema Files

### cursor-rule-schema.yaml
Defines the structure for `.cursor/rules/*.yaml` files with:
- Formal field definitions and constraints
- Required vs optional fields
- Validation patterns and examples
- Metadata management requirements

### agent-spec-schema.yaml
Defines the structure for `agents/**/*.yaml` files with:
- Comprehensive agent specification format
- Domain-specific extensions (e.g., GitHub integration)
- Security and behavioral directive requirements
- Success criteria and metrics definitions

## 🔄 Automatic Versioning System

The `version-manager.py` script provides automatic version management:

### Installation
```bash
# Install required dependencies
pip install pyyaml jsonschema semantic-version

# Make the script executable
chmod +x .cursor/version-manager.py
```

### Usage

#### Update All Files
```bash
# Update all cursor rules and agent specs with automatic versioning
python .cursor/version-manager.py --all

# Update only cursor rules
python .cursor/version-manager.py --rules

# Update only agent specifications
python .cursor/version-manager.py --specs
```

#### Update Specific Files
```bash
# Update a specific cursor rule
python .cursor/version-manager.py --file .cursor/rules/00-core-baseline.yaml

# Update a specific agent spec
python .cursor/version-manager.py --file agents/v1.0.0/github/github-api-integration.yaml
```

#### Validation Only
```bash
# Validate without updating
python .cursor/version-manager.py --all --validate-only
```

### How Versioning Works

The system automatically determines version bumps based on changes:

#### Major Version (X.0.0)
- Breaking changes to required fields
- Capability or objective removals
- Priority level changes
- Schema-breaking modifications

#### Minor Version (X.Y.0)
- New capabilities added
- New objectives added
- New guidelines or features
- Backward-compatible enhancements

#### Patch Version (X.Y.Z)
- Documentation updates
- Minor corrections
- Non-breaking clarifications
- Metadata updates

### Version Database

The system maintains a version database at `.cursor/version_db.json` containing:
- Current version for each file
- Creation and modification timestamps
- Content checksums for change detection
- Previous content for comparison

## 📋 Schema Validation

### Cursor Rules Validation

All cursor rules must follow the schema requirements:

#### Required Fields
- `name`: PascalCase rule name
- `version`: Semantic version (auto-managed)
- `description`: Clear description (10-200 chars)
- `category`: One of predefined categories
- `priority`: Integer 1-100
- `always_apply`: Boolean
- `extends`: Array of parent rules
- `objectives`: Array of objectives
- `guidelines`: Object with guideline groups

#### Example Valid Rule
```yaml
name: "ExampleRule"
version: "1.0.0"
description: "Example rule demonstrating proper structure"
category: "core"
priority: 50
always_apply: true
extends: ["CoreBaseline"]
objectives:
  - "Demonstrate proper rule structure"
guidelines:
  example_section:
    - "Follow this pattern for all rules"
    - "Maintain consistency across implementations"
```

### Agent Specification Validation

All agent specs must follow the schema requirements:

#### Required Sections
- `metadata`: Version, creation date, workstream info
- `spec`: Name, domain, priority, description
- `capabilities`: Primary and secondary capabilities
- `objectives`: Goals with deliverables and validation
- `tasks`: Default tasks and optional conditional tasks
- `dependencies`: Required and optional dependencies
- `reporting`: Frequency, channels, metrics
- `security`: Sandbox, capabilities, resource limits
- `behavioral_directives`: Operational focus, error handling, coordination
- `risk_mitigation`: High priority risks and monitoring
- `success_criteria`: Phase and final validation criteria

#### Domain-Specific Extensions

GitHub integration agents can include additional sections:
- `github_integration`: API endpoints, webhook events, permissions, rate limiting

## 🛠️ Development Workflow

### Creating New Rules

1. **Create the file** following the naming convention:
   ```bash
   touch .cursor/rules/80-new-rule.yaml
   ```

2. **Write the rule** following the schema:
   ```yaml
   name: "NewRule"
   description: "Description of the new rule"
   category: "appropriate-category"
   priority: 75
   # ... other required fields
   ```

3. **Validate and version**:
   ```bash
   python .cursor/version-manager.py --file .cursor/rules/80-new-rule.yaml
   ```

### Creating New Agent Specs

1. **Create the directory structure**:
   ```bash
   mkdir -p agents/v1.0.0/new-domain
   ```

2. **Create the specification**:
   ```bash
   touch agents/v1.0.0/new-domain/new-agent.yaml
   ```

3. **Write the specification** following the schema requirements

4. **Validate and version**:
   ```bash
   python .cursor/version-manager.py --file agents/v1.0.0/new-domain/new-agent.yaml
   ```

### Updating Existing Files

1. **Make your changes** to the file
2. **Run the version manager** to auto-version:
   ```bash
   python .cursor/version-manager.py --file path/to/changed/file.yaml
   ```
3. **Review the version bump** and changelog
4. **Commit the changes** with appropriate commit message

## 🔍 Validation and Quality Assurance

### Pre-commit Validation

Add to your git hooks:
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating configuration files..."
python .cursor/version-manager.py --all --validate-only

if [ $? -ne 0 ]; then
    echo "Configuration validation failed!"
    exit 1
fi
```

### CI/CD Integration

Add to your CI pipeline:
```yaml
# .github/workflows/validate-config.yml
name: Validate Configuration
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
      - name: Validate configuration
        run: python .cursor/version-manager.py --all --validate-only
```

## 📊 Monitoring and Maintenance

### Version Database Maintenance

The version database should be:
- **Committed to version control** for team consistency
- **Backed up regularly** to prevent data loss
- **Cleaned periodically** to remove obsolete entries

### Schema Evolution

When updating schemas:
1. **Update the schema file**
2. **Increment the schema version**
3. **Update the version manager** to handle new features
4. **Migrate existing files** if necessary
5. **Update documentation** and examples

## 🎯 Best Practices

### Rule Creation
- **Start with existing rules** and extend them
- **Use descriptive names** in PascalCase
- **Set appropriate priorities** based on importance
- **Group related guidelines** logically
- **Include validation tests** where applicable

### Agent Specification
- **Follow naming conventions** (kebab-case for identifiers)
- **Define clear objectives** with measurable deliverables
- **Specify realistic resource limits**
- **Include comprehensive risk mitigation**
- **Document domain-specific requirements**

### Version Management
- **Run validation regularly** during development
- **Review version bumps** before committing
- **Use semantic commit messages** that reflect changes
- **Update related documentation** when schemas change

## 🚨 Troubleshooting

### Common Issues

#### Schema Validation Errors
```bash
# Check specific validation errors
python .cursor/version-manager.py --file problematic-file.yaml --validate-only
```

#### Version Conflicts
```bash
# Reset version if corrupted
rm .cursor/version_db.json
python .cursor/version-manager.py --all
```

#### Missing Dependencies
```bash
# Install required Python packages
pip install pyyaml jsonschema semantic-version
```

### Getting Help

1. **Check the schema files** for field requirements
2. **Look at existing examples** for proper structure
3. **Run validation** to see specific error messages
4. **Review the version database** for historical information

---

This schema-based system ensures consistency and quality across all configuration files while providing automatic versioning and validation. Follow the guidelines above to maintain a clean, well-organized configuration system. 