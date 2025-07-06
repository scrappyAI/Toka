# Toka Documentation

> **Last Updated:** 2025-07-06

Welcome to the Toka documentation. This directory contains all project documentation organized by category.

## 📋 Quick Navigation

- [**Getting Started**](#getting-started) - New user guides and setup
- [**Architecture**](#architecture) - System design and specifications  
- [**Development**](#development) - Development guides and processes
- [**Operations**](#operations) - Deployment and monitoring
- [**Research**](#research) - Technical research and proposals
- [**Reports**](#reports) - Analysis and status reports

## 🚀 Getting Started

| Document | Description |
|----------|-------------|
| [QUICK_START_TESTING.md](../QUICK_START_TESTING.md) | Quick setup guide for testing |
| [TOKA_TESTING_SETUP_GUIDE.md](../TOKA_TESTING_SETUP_GUIDE.md) | Comprehensive testing setup |
| [DEVELOPMENT_ENVIRONMENT.md](DEVELOPMENT_ENVIRONMENT.md) | Development environment setup |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contributing guidelines |

## 🏗️ Architecture

| Document | Description |
|----------|-------------|
| [CRATES.md](CRATES.md) | Crate architecture overview |
| [40_capability_tokens_spec_v0.2.md](40_capability_tokens_spec_v0.2.md) | Capability tokens specification |
| [41_capability_tokens_architecture.md](41_capability_tokens_architecture.md) | Capability tokens architecture |
| [44_toka_kernel_spec_v0.2.md](44_toka_kernel_spec_v0.2.md) | Toka kernel specification |
| [protocols/](protocols/) | Protocol specifications |

## 💻 Development

| Document | Description |
|----------|-------------|
| [guides/](guides/) | Development guides |
| [30_doc-generation.mdc](30_doc-generation.mdc) | Documentation generation |
| [31_doc-maintenance.mdc](31_doc-maintenance.mdc) | Documentation maintenance |
| [code_coverage_guide.mdc](code_coverage_guide.mdc) | Code coverage guide |

## 🔧 Operations

| Document | Description |
|----------|-------------|
| [RAFT_MONITORING_README.md](../RAFT_MONITORING_README.md) | Raft monitoring system |
| [RAFT_MONITORING_DEPLOYMENT_SUMMARY.md](../RAFT_MONITORING_DEPLOYMENT_SUMMARY.md) | Deployment summary |
| [SECURITY_HARDENING_SUMMARY.md](SECURITY_HARDENING_SUMMARY.md) | Security hardening |
| [MEMORY_LEAK_ANALYSIS.md](MEMORY_LEAK_ANALYSIS.md) | Memory leak analysis |

## 🔬 Research

| Document | Description |
|----------|-------------|
| [research/](research/) | Technical research papers |
| [proposals/](proposals/) | Enhancement proposals |
| [data/](data/) | Research data and analysis |

## 📊 Reports

| Document | Description |
|----------|-------------|
| [reports/](reports/) | Analysis and status reports |
| [code_coverage_reports/](code_coverage_reports/) | Code coverage reports |
| [CLEANUP_SUMMARY.md](CLEANUP_SUMMARY.md) | Cleanup activities |
| [REFACTOR_SUMMARY.md](REFACTOR_SUMMARY.md) | Refactoring activities |
| [PROJECT_ORGANIZATION_SUMMARY.md](PROJECT_ORGANIZATION_SUMMARY.md) | Project organization |

## 🏷️ Documentation Standards

### Date Enforcement

All documentation follows strict date accuracy requirements:
- Use `2025-07-06` for current date placeholders
- Use `2025-07-06` for release/commit dates
- Historical dates require `<!-- DATE:EXEMPT source="reference" -->` exemption
- Run `scripts/validate_dates.py` to check compliance

### Format Standards

- Use clear, descriptive headings
- Include table of contents for long documents
- Use consistent date format: `YYYY-MM-DD`
- Include status indicators: **Draft**, **Stable**, **Deprecated**
- Link to related documents

### Maintenance

- Documents are reviewed monthly for accuracy
- Outdated information is marked for update or removal
- Version numbers track major content changes
- Use semantic versioning for specifications

## 🔍 Finding Information

Use the following strategies to find what you need:

1. **Check this index** - organized by category
2. **Search by file name** - descriptive naming convention
3. **Use grep** - most documents are well-tagged
4. **Check proposals/** - for planned features
5. **Check reports/** - for historical analysis

## 🤝 Contributing to Documentation

1. Follow the [CONTRIBUTING.md](CONTRIBUTING.md) guidelines
2. Use the date enforcement tools (`scripts/validate_dates.py`)
3. Update this index when adding new documents
4. Maintain consistent formatting and structure
5. Link to related documents

---

*This documentation structure is maintained as part of the Toka project's commitment to clear, accurate, and well-organized information.*