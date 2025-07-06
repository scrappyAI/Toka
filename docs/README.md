# Toka Documentation

> **Last Updated:** 2025-07-06  
> **Status**: Reorganized and Cleaned

Welcome to the Toka documentation. This directory contains all project documentation organized by category according to the document organization agent specification.

## 📋 Quick Navigation

- [**Getting Started**](#getting-started) - New user guides and setup
- [**Architecture**](#architecture) - System design and specifications  
- [**Development**](#development) - Development guides and processes
- [**Operations**](#operations) - Deployment and monitoring
- [**Agent System**](#agent-system) - Agent configurations and orchestration
- [**API**](#api) - Integration guides and references
- [**Research**](#research) - Technical research and proposals
- [**Reports**](#reports) - Analysis and status reports

## 🚀 Getting Started

| Document | Description | Location |
|----------|-------------|----------|
| [QUICKSTART.md](../QUICKSTART.md) | Quick start guide | Root |
| [Development Environment](development/README.md) | Development setup | Development |
| [Contributing Guidelines](development/CONTRIBUTING.md) | How to contribute | Development |

## 🏗️ Architecture

| Document | Description | Location |
|----------|-------------|----------|
| [Architecture Overview](architecture/README.md) | System design and specifications | Architecture |
| [CRATES.md](CRATES.md) | Crate architecture overview | Root |
| [Capability Tokens Spec](architecture/40_capability_tokens_spec_v0.2.md) | Security model specification | Architecture |
| [Capability Architecture](architecture/41_capability_tokens_architecture.md) | Implementation details | Architecture |
| [Kernel Specification](architecture/44_toka_kernel_spec_v0.2.md) | Core system design | Architecture |

## 💻 Development

| Document | Description | Location |
|----------|-------------|----------|
| [Development Guide](development/README.md) | Development workflows and tools | Development |
| [Testing Guide](development/TOKA_TESTING_GUIDE.md) | Testing strategies and setup | Development |
| [CLI Guide](development/TOKA_CLI_GUIDE.md) | Command-line interface | Development |
| [Environment Setup](development/DEVELOPMENT_ENVIRONMENT.md) | Development environment | Development |

## 🔧 Operations

| Document | Description | Location |
|----------|-------------|----------|
| [Operations Guide](operations/README.md) | Deployment and monitoring | Operations |
| [Docker Guide](operations/README-Docker.md) | Docker deployment | Operations |
| [Docker Environments](operations/README-Docker-Environments.md) | Environment configuration | Operations |
| [Environment Config](operations/README-Environment.md) | Environment setup | Operations |
| [Security Hardening](operations/SECURITY_HARDENING_SUMMARY.md) | Security best practices | Operations |
| [Memory Analysis](operations/MEMORY_LEAK_ANALYSIS.md) | Performance optimization | Operations |

## 🤖 Agent System

| Document | Description | Location |
|----------|-------------|----------|
| [Agent System Guide](agents/README.md) | Agent configuration and orchestration | Agents |
| [Orchestration Guide](agents/README-Orchestration.md) | Multi-agent orchestration | Agents |
| [Workstream Configs](../../agents/v0.3.0/workstreams/) | Workstream specifications | Agents |

## 🔌 API

| Document | Description | Location |
|----------|-------------|----------|
| [API Documentation](api/README.md) | Integration guides and references | API |

## 🔬 Research

| Document | Description | Location |
|----------|-------------|----------|
| [research/](research/) | Technical research papers | Research |
| [proposals/](proposals/) | Enhancement proposals | Proposals |
| [data/](data/) | Research data and analysis | Data |

## 📊 Reports

| Document | Description | Location |
|----------|-------------|----------|
| [reports/](reports/) | Analysis and status reports | Reports |
| [code_coverage_reports/](code_coverage_reports/) | Code coverage reports | Coverage |
| [Cleanup Summary](reports/CLEANUP_SUMMARY.md) | Cleanup activities | Reports |
| [Refactor Summary](reports/REFACTOR_SUMMARY.md) | Refactoring activities | Reports |
| [Project Organization](reports/PROJECT_ORGANIZATION_SUMMARY.md) | Project organization | Reports |
| [Config CLI Implementation](reports/CONFIG_CLI_IMPLEMENTATION_SUMMARY.md) | CLI implementation | Reports |
| [Memory Context Management](reports/memory-context-management-report.md) | Memory management research | Reports |
| [GitHub CI/CD Fixes](reports/github-cicd-fixes-report.md) | CI/CD improvements | Reports |

## 🏷️ Documentation Standards

### Organization Standards

All documentation follows the document organization agent specification:
- **Architecture & Design** → `docs/architecture/`
- **Development Guides** → `docs/development/`
- **API Documentation** → `docs/api/`
- **Deployment & Operations** → `docs/operations/`
- **Agent System** → `docs/agents/`

### Quality Standards

- **Formatting**: Consistent markdown with clear headings
- **Navigation**: Table of contents for long documents
- **Status Indicators**: Draft, Stable, Deprecated
- **Cross-references**: Link to related documents
- **Date Format**: `YYYY-MM-DD` for all dates

### Maintenance

- **Monthly Review**: Documents reviewed for accuracy
- **Version Control**: Track major content changes
- **Link Validation**: Ensure all links remain functional
- **Content Updates**: Remove outdated information

## 🔍 Finding Information

Use these strategies to find what you need:

1. **Category Navigation** - Use the organized directory structure
2. **Search by Topic** - Look in the appropriate category
3. **Cross-references** - Follow links between related documents
4. **Status Indicators** - Check document status for relevance
5. **Quick Navigation** - Use the table of contents in each category

## 🤝 Contributing to Documentation

1. Follow the [Contributing Guidelines](development/CONTRIBUTING.md)
2. Use the appropriate category directory
3. Update this index when adding new documents
4. Maintain consistent formatting and structure
5. Link to related documents within the same category

---

*This documentation structure is maintained as part of the Toka project's commitment to clear, accurate, and well-organized information.*