# Agent Configuration Rules

This directory contains **agent-friendly configuration rules** for the Rust workspace. These rules have been migrated from the previous XML-flavored markup to a structured YAML format that's easier for AI agents to parse and understand.

## 🚀 Migration Status

**✅ MIGRATED TO AGENT-FRIENDLY FORMAT**

The rules have been reorganized and converted to YAML for better agent compatibility:
- **From**: XML-like `.mdc` files with inconsistent structure
- **To**: Structured YAML files with standardized schemas
- **Benefit**: Easier for AI agents to parse, validate, and apply

## 📁 Directory Structure

| File | Category | Priority | Description |
|------|----------|----------|-------------|
| `00-core-baseline.yaml` | Core | 100 | Foundational guidelines for all Rust projects |
| `10-security-base.yaml` | Security | 90 | Core security hardening and best practices |
| `11-security-agents.yaml` | Security | 85 | AI agent-specific security constraints |
| `20-testing-quality.yaml` | Testing | 80 | Testing guidelines and coverage requirements |
| `30-documentation.yaml` | Documentation | 70 | Documentation generation and maintenance |
| `40-development-process.yaml` | Process | 60 | Development workflows and change management |
| `50-architecture-research.yaml` | Architecture | 50 | Architecture docs and codebase research |
| `60-toka-workspace.yaml` | Project | 40 | Toka-specific workspace organization |

## 🏗️ YAML Schema

Each rule file follows this standardized structure:

```yaml
name: "RuleName"
version: "1.0.0"
description: "What this rule accomplishes"
category: "core|security|testing|documentation|process|architecture|project"
priority: 1-100  # Higher numbers = higher priority
always_apply: true|false

extends: ["CoreBaseline", "OtherRules"]  # Inheritance chain

objectives:
  - "Clear goal 1"
  - "Clear goal 2"

# Rule-specific sections with actionable guidelines
guidelines:
  section_name:
    - "Actionable instruction"
    - "Another instruction"

# Commit conventions, tools, workflows, etc.
```

## 🔄 Extension System

Rules support composition through the `extends` field:

```yaml
# Example: Security rule that builds on baseline
extends: ["CoreBaseline", "SecurityBase"]
```

This creates a hierarchy where:
1. `CoreBaseline` provides universal foundations
2. `SecurityBase` adds security-specific requirements  
3. Later rules can override or extend specific sections

## 🎯 Agent Integration

These YAML rules are designed for AI agents to:

- **Parse easily**: Structured YAML vs XML-like markup
- **Validate automatically**: Clear schema and required fields
- **Apply contextually**: Priority system and category filtering
- **Compose dynamically**: Extension system for rule combinations

## 🛠️ Usage by Agents

Agents should:

1. **Load rules by priority**: Start with highest priority (100) and work down
2. **Resolve extensions**: Follow the `extends` chain to build complete rule sets
3. **Filter by category**: Apply only relevant categories for the current task
4. **Validate compliance**: Check that implementations follow the guidelines

## 📋 Legacy Files

The following legacy `.mdc` files are **DEPRECATED** and will be removed:

- ~~`00_baseline.mdc`~~ → `00-core-baseline.yaml`
- ~~`10_security-hardening-base.mdc`~~ → `10-security-base.yaml`
- ~~`11_security-hardening-agents.mdc`~~ → `11-security-agents.yaml`
- ~~`20_testing-code-coverage.mdc`~~ → `20-testing-quality.yaml`
- ~~`25_debugging-rust.mdc`~~ → `20-testing-quality.yaml` (merged)
- ~~`30_doc-generation.mdc`~~ → `30-documentation.yaml`
- ~~`31_doc-maintenance.mdc`~~ → `30-documentation.yaml` (merged)
- ~~`40_refactoring-guidelines.mdc`~~ → `40-development-process.yaml`
- ~~`date-enforcement.mdc`~~ → `40-development-process.yaml` (merged)
- ~~`proposal-gen-guide.mdc`~~ → `40-development-process.yaml` (merged)
- ~~`50_protocol-adherence.mdc`~~ → `50-architecture-research.yaml`
- ~~`architecture-gen.mdc`~~ → `50-architecture-research.yaml` (merged)
- ~~`code-research.mdc`~~ → `50-architecture-research.yaml` (merged)
- ~~`code_optimization.mdc`~~ → `50-architecture-research.yaml` (merged)
- ~~`60_toka-workspace-evolution.mdc`~~ → `60-toka-workspace.yaml`

## 🎓 Best Practices for Agents

1. **Start with baseline**: Always include `CoreBaseline` in your rule set
2. **Layer appropriately**: Add security, testing, docs rules as needed
3. **Check priorities**: Higher priority rules take precedence
4. **Validate compliance**: Ensure generated code follows all applicable guidelines
5. **Document choices**: When extending rules, explain the rationale

## 🔧 Future Improvements

- [ ] Add JSON Schema validation for rule files
- [ ] Create agent tooling for rule composition and validation
- [ ] Add rule templates for common patterns
- [ ] Integrate with CI/CD for automated compliance checking
- [ ] Add rule versioning and migration tooling

---

**Note**: This represents a significant improvement in agent usability. The structured YAML format makes these rules much more practical for AI agents to understand and apply consistently.