# DEPRECATED LEGACY RULES

These `.mdc` files are **DEPRECATED** and have been replaced with agent-friendly YAML configurations in the parent directory.

## Migration Status

All rules have been migrated to structured YAML format for better AI agent compatibility:

- **Original Format**: XML-like markup in `.mdc` files
- **New Format**: Structured YAML with standardized schemas
- **Benefits**: Easier parsing, validation, and application by AI agents

## What Changed

1. **Consolidated**: Multiple related rules merged into single files
2. **Standardized**: Consistent YAML schema across all rules
3. **Prioritized**: Added priority system for rule application order
4. **Categorized**: Clear categories for different types of guidelines

## New File Mapping

| Legacy File | New File | Status |
|-------------|----------|---------|
| `00_baseline.mdc` | `../00-core-baseline.yaml` | ✅ Migrated |
| `10_security-hardening-base.mdc` | `../10-security-base.yaml` | ✅ Migrated |
| `11_security-hardening-agents.mdc` | `../11-security-agents.yaml` | ✅ Migrated |
| `12_security-agent-tools.mdc` | `../11-security-agents.yaml` | ✅ Merged |
| `13_security-agent-debugging.mdc` | `../11-security-agents.yaml` | ✅ Merged |
| `20_testing-code-coverage.mdc` | `../20-testing-quality.yaml` | ✅ Migrated |
| `25_debugging-rust.mdc` | `../20-testing-quality.yaml` | ✅ Merged |
| `30_doc-generation.mdc` | `../30-documentation.yaml` | ✅ Migrated |
| `31_doc-maintenance.mdc` | `../30-documentation.yaml` | ✅ Merged |
| `40_refactoring-guidelines.mdc` | `../40-development-process.yaml` | ✅ Migrated |
| `date-enforcement.mdc` | `../40-development-process.yaml` | ✅ Merged |
| `proposal-gen-guide.mdc` | `../40-development-process.yaml` | ✅ Merged |
| `50_protocol-adherence.mdc` | `../50-architecture-research.yaml` | ✅ Migrated |
| `architecture-gen.mdc` | `../50-architecture-research.yaml` | ✅ Merged |
| `code-research.mdc` | `../50-architecture-research.yaml` | ✅ Merged |
| `code_optimization.mdc` | `../50-architecture-research.yaml` | ✅ Merged |
| `60_toka-workspace-evolution.mdc` | `../60-toka-workspace.yaml` | ✅ Migrated |

## Removal Timeline

These legacy files will be removed in a future cleanup once all agents have been updated to use the new YAML format.

---

**Use the YAML files in the parent directory instead of these deprecated files.**