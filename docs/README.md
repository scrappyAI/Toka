# Toka Documentation

Welcome to the Toka documentation hub. This directory contains all project documentation organized by category for easy navigation.

## 📚 Documentation Structure

### Core Documentation
- **[DEVELOPMENT_ENVIRONMENT.md](./DEVELOPMENT_ENVIRONMENT.md)** - Comprehensive development environment setup
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Guidelines for contributing to the project
- **[CRATES.md](./CRATES.md)** - Overview of all crates in the workspace

### Project Summaries & Reports
- **[CLEANUP_SUMMARY.md](./CLEANUP_SUMMARY.md)** - Project cleanup and organization summary
- **[CONFIG_CLI_IMPLEMENTATION_SUMMARY.md](./CONFIG_CLI_IMPLEMENTATION_SUMMARY.md)** - Configuration CLI implementation details
- **[REFACTOR_SUMMARY.md](./REFACTOR_SUMMARY.md)** - Major refactoring changes and improvements
- **[SECURITY_HARDENING_SUMMARY.md](./SECURITY_HARDENING_SUMMARY.md)** - Security improvements and hardening measures
- **[MEMORY_LEAK_ANALYSIS.md](./MEMORY_LEAK_ANALYSIS.md)** - Memory leak analysis and fixes

### Specifications & Architecture
- **[44_toka_kernel_spec_v0.2.md](./44_toka_kernel_spec_v0.2.md)** - Toka kernel specification v0.2
- **[40_capability_tokens_spec_v0.2.md](./40_capability_tokens_spec_v0.2.md)** - Capability tokens specification v0.2
- **[41_capability_tokens_architecture.md](./41_capability_tokens_architecture.md)** - Capability tokens architecture

### Development Guides
- **[30_doc-generation.mdc](./30_doc-generation.mdc)** - Documentation generation guide
- **[31_doc-maintenance.mdc](./31_doc-maintenance.mdc)** - Documentation maintenance guide
- **[code_coverage_guide.mdc](./code_coverage_guide.mdc)** - Code coverage guide
- **[github-cicd-fixes-report.md](./github-cicd-fixes-report.md)** - GitHub CI/CD fixes report

### Organized Subdirectories
- **[protocols/](./protocols/)** - Protocol specifications and definitions
- **[reports/](./reports/)** - Various project reports and analyses
- **[research/](./research/)** - Research documents and findings
- **[proposals/](./proposals/)** - Project proposals and RFCs
- **[code_coverage_reports/](./code_coverage_reports/)** - Code coverage reports
- **[data/](./data/)** - Data files and large metadata

## 🔍 Quick Navigation

### For New Contributors
1. Start with [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Set up your environment using [DEVELOPMENT_ENVIRONMENT.md](./DEVELOPMENT_ENVIRONMENT.md)
3. Review the [CRATES.md](./CRATES.md) for workspace overview

### For Developers
- **Environment Setup**: [DEVELOPMENT_ENVIRONMENT.md](./DEVELOPMENT_ENVIRONMENT.md)
- **Architecture**: Check specifications in the main directory
- **Testing**: [code_coverage_guide.mdc](./code_coverage_guide.mdc)
- **Security**: [SECURITY_HARDENING_SUMMARY.md](./SECURITY_HARDENING_SUMMARY.md)

### For Maintainers
- **Project Status**: Various summary reports
- **Documentation**: [30_doc-generation.mdc](./30_doc-generation.mdc) and [31_doc-maintenance.mdc](./31_doc-maintenance.mdc)
- **CI/CD**: [github-cicd-fixes-report.md](./github-cicd-fixes-report.md)

## 🏗️ Project Structure

This documentation follows the principle of organizing files by their semantic purpose:

```
docs/
├── README.md                              # This file - documentation index
├── DEVELOPMENT_ENVIRONMENT.md             # Development setup
├── CONTRIBUTING.md                        # Contribution guidelines
├── CRATES.md                             # Crates overview
├── *_SUMMARY.md                          # Project summaries
├── *_spec_*.md                           # Specifications
├── *_architecture.md                     # Architecture documents
├── *.mdc                                 # Guides and workflows
├── protocols/                            # Protocol specifications
├── reports/                              # Project reports
├── research/                             # Research documents
├── proposals/                            # Project proposals
├── code_coverage_reports/                # Coverage reports
└── data/                                 # Data files and metadata
```

## � Recent Changes

### File Organization (Latest)
- Moved all documentation from project root to `docs/` directory
- Created semantic subdirectories for better organization
- Removed duplicate files and consolidated documentation
- Created visual dependency graph for `toka-auth` crate
- Organized large data files in `docs/data/`

### Key Improvements
- **Better Navigation**: Clear categorization of documentation
- **Reduced Clutter**: Clean project root with only essential files
- **Enhanced Discoverability**: Logical grouping of related documents
- **Visual Dependencies**: Added dependency graphs for better understanding

## 🎯 Contributing to Documentation

When adding new documentation:

1. **Place files in appropriate subdirectories** based on their purpose
2. **Update this README** if adding new categories or important documents
3. **Follow naming conventions** for consistency
4. **Cross-reference related documents** for better navigation
5. **Keep summaries current** when making significant changes

## 🔧 Tools and Resources

- **Mermaid Diagrams**: Used for visual representations (dependency graphs, flowcharts)
- **Markdown**: Standard format for all documentation
- **MDC Files**: Markdown with custom extensions for specific workflows

---

For questions about documentation structure or content, refer to the [CONTRIBUTING.md](./CONTRIBUTING.md) guidelines or open an issue in the project repository.