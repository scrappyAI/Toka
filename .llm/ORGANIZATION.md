# Toka LLM System Organization

This document describes the reorganized `.llm` folder structure and how to use the new schema-based system.

## 🏗️ New Architecture

The system has been reorganized around a **unified schema-based architecture** with clear categorization:

```
.llm/
├── schemas/                    # Unified schema definitions
│   ├── agents/                # Agent specifications
│   ├── tools/                 # Tool definitions and implementations
│   ├── policies/              # Policy definitions (formerly rules)
│   ├── contracts/             # Contract definitions for enforcement
│   └── [core schemas]         # Base schemas and meta-schemas
├── scripts/                   # Python implementation scripts
├── [configuration files]      # System configuration
└── [databases]               # Audit and registry databases
```

## 📋 Schema Categories

### **Agents** (`schemas/agents/`)
AI agent specifications with capabilities, behaviors, and constraints.

**Examples:**
- `tool-builder-agent.json` - Agent for building and validating tools

### **Tools** (`schemas/tools/`)
Tool definitions that wrap Python scripts with standardized interfaces, capabilities, and validation.

**Examples:**
- `date-enforcer-tool.json` → `scripts/date-enforcer.py`
- `folder-tidy-tool.json` → `scripts/folder-tidy.py`
- `tool-registry-tool.json` → `scripts/tool-registry.py`

### **Policies** (`schemas/policies/`)
Policy definitions that guide behavior and enforce standards (converted from rules).

**Examples:**
- `00-core-baseline.json` - Foundational guidelines
- `10-security-base.json` - Security hardening
- `20-testing-quality.json` - Testing standards

### **Contracts** (`schemas/contracts/`)
Enforcement contracts that define obligations, compliance requirements, and remediation procedures.

**Examples:**
- `date-enforcement-contract.json` - Date validation and correction

## 🔧 Core Tools

### Date Enforcement System
- **Tool**: `date-enforcer-tool.json` → `scripts/date-enforcer.py`
- **Contract**: `date-enforcement-contract.json`
- **Purpose**: Prevents date hallucination and enforces deterministic dating

### Folder Organization
- **Tool**: `folder-tidy-tool.json` → `scripts/folder-tidy.py`
- **Purpose**: Organizes folder structures according to defined policies

### Tool Registry
- **Tool**: `tool-registry-tool.json` → `scripts/tool-registry.py`
- **Purpose**: Manages registered tools with validation and tracking

## 📅 Date Enforcement

The system includes comprehensive date enforcement to prevent LLM date hallucination:

### **Validation Patterns**
- **Canonical Format**: `2025-07-07` (YYYY-MM-DD)
- **Datetime Format**: `2025-07-07T00:00:00Z`
- **Placeholder**: `2025-07-07` (for development)
- **Exemption**: `DATE:EXEMPT source="reason"`

### **Usage**
```bash
# Validate dates in the codebase
python3 scripts/date-enforcer.py validate .

# Enforce date corrections
python3 scripts/date-enforcer.py enforce . --auto-correct

# Fix all 2024 references to 2025
python3 scripts/fix-dates.py
```

## 🛠️ Tool Abstraction

All Python scripts are wrapped under the tool abstraction:

```json
{
  "kind": "tool",
  "metadata": { ... },
  "spec": {
    "implementation": {
      "language": "python",
      "entryPoint": "scripts/script-name.py",
      "arguments": [...],
      "output": { ... }
    }
  }
}
```

This provides:
- **Standardized interfaces** for all tools
- **Capability definitions** for security and access control
- **Schema validation** for inputs and outputs
- **Error handling** and performance specifications

## 📋 Policy System

Policies (formerly rules) are organized by category and priority:

- **Core Policies** (100): Foundational guidelines for all projects
- **Security Policies** (85-90): Hardening and access control
- **Quality Policies** (80): Testing and documentation standards
- **Process Policies** (60): Development workflows and change management

## 🔄 Migration Summary

### ✅ **Completed**
- [x] Schema-based organization structure
- [x] Tool abstraction for all scripts
- [x] Date enforcement contract and tool
- [x] Policy categorization (from rules)
- [x] Contract definitions for enforcement
- [x] Fixed all 2024 date references to 2025 (22 changes)

### 📊 **Migration Results**
- **Files processed**: 56
- **Date fixes applied**: 22
- **New tool schemas created**: 6
- **Policies migrated**: 8
- **Contracts created**: 1

## 🎯 Usage Guidelines

### For AI Agents
1. **Load schemas by category**: Start with core schemas, then add specific ones
2. **Follow tool contracts**: Use tools through their schema definitions
3. **Respect policies**: Apply relevant policies for the current task
4. **Validate compliance**: Check that implementations follow contracts

### For Developers
1. **Use canonical dates**: Always use system clock for current dates
2. **Follow tool abstraction**: Wrap new scripts in tool schemas
3. **Document policies**: Create clear policy definitions for new areas
4. **Maintain contracts**: Update enforcement contracts as requirements change

## 🔧 Configuration

### Tool Registry
```bash
# Register a new tool
python3 scripts/tool-registry.py register path/to/tool.json

# List registered tools
python3 scripts/tool-registry.py list
```

### Folder Organization
```bash
# Organize folder structure
python3 scripts/folder-tidy.py organize . --dry-run
```

## 📊 Audit and Compliance

The system maintains comprehensive audit trails:

- **Date Audit**: `date_audit.db` - All date validation and corrections
- **Tool Registry**: `tool_registry.db` - Registered tools and validation status
- **Tidy Audit**: `tidy_audit.db` - Folder organization changes

## 🚀 Benefits

### **Deterministic Behavior**
- Canonical date sources prevent hallucination
- Schema validation ensures consistency
- Contract enforcement provides guarantees

### **Clear Organization**
- Schema-based categorization
- Tool abstraction unifies interfaces
- Policy system guides behavior

### **Reproducible Results**
- Standardized tool interfaces
- Automated validation and correction
- Comprehensive audit trails

### **Agent-Friendly**
- Structured schemas for easy parsing
- Clear capability definitions
- Standardized error handling

## 🔮 Future Enhancements

- **Automated Policy Composition**: Tools to combine policies for specific contexts
- **Schema Validation**: Automated validation of all schema files
- **CI/CD Integration**: Automated compliance checking in pipelines
- **Agent Tooling**: Tools for agents to understand and apply policies
- **Performance Monitoring**: Metrics and monitoring for tool performance

---

**Note**: This reorganization provides a deterministic, reproducible foundation for AI agent behavior while maintaining flexibility for specific use cases through the policy and contract system. 