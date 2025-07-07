# Toka Scripts

> **Category**: System Scripts  
> **Location**: `scripts/`  
> **Status**: Stable

This directory contains organized system scripts for the Toka OS project, following the core baseline guidelines for maintainability and consistency.

## 📋 Quick Navigation

- [**Setup Scripts**](#setup-scripts) - Environment and system setup
- [**Testing Scripts**](#testing-scripts) - Testing and validation
- [**Workflow Scripts**](#workflow-scripts) - Development workflows
- [**Monitoring Scripts**](#monitoring-scripts) - System monitoring

## 🚀 Setup Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| [setup_toka_testing.sh](setup/setup_toka_testing.sh) | Complete testing environment setup | `./scripts/setup/setup_toka_testing.sh` |
| [setup-docker-environments.sh](setup/setup-docker-environments.sh) | Docker environment configuration | `./scripts/setup/setup-docker-environments.sh` |

### Setup Scripts Overview

**Purpose**: Initialize and configure the Toka development environment
- **Environment Configuration**: Set up development, testing, and production environments
- **Dependency Management**: Install and configure required dependencies
- **Build System**: Prepare the build environment and compile binaries
- **Configuration Files**: Generate necessary configuration files

## 🧪 Testing Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| [test_toka_agents.sh](testing/test_toka_agents.sh) | Comprehensive agent testing | `./scripts/testing/test_toka_agents.sh` |
| [run_simple_test.sh](testing/run_simple_test.sh) | Basic functionality testing | `./scripts/testing/run_simple_test.sh` |

### Testing Scripts Overview

**Purpose**: Validate system functionality and agent behavior
- **Agent Testing**: Test individual agent capabilities and interactions
- **Integration Testing**: Validate system integration and communication
- **Performance Testing**: Measure system performance and resource usage
- **Regression Testing**: Ensure changes don't break existing functionality

## 🔄 Workflow Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| [toka_workflow.sh](workflow/toka_workflow.sh) | Complete system workflow demonstration | `./scripts/workflow/toka_workflow.sh` |
| [toka_interactive.sh](workflow/toka_interactive.sh) | Interactive CLI for agent management | `./scripts/workflow/toka_interactive.sh` |

### Workflow Scripts Overview

**Purpose**: Demonstrate and manage Toka system workflows
- **System Workflows**: Complete end-to-end system demonstrations
- **Interactive Management**: Command-line interface for system management
- **Agent Orchestration**: Multi-agent coordination and task management
- **Development Workflows**: Streamlined development processes

## 📊 Monitoring Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| [raft_monitoring_service.sh](monitoring/raft_monitoring_service.sh) | Raft development monitoring | `./scripts/monitoring/raft_monitoring_service.sh` |

### Monitoring Scripts Overview

**Purpose**: Monitor system health and performance
- **System Monitoring**: Track system resources and performance
- **Development Monitoring**: Monitor development progress and metrics
- **Health Checks**: Validate system health and functionality
- **Performance Analysis**: Analyze system performance and bottlenecks

## 🛠️ Script Standards

### Code Quality
- **Error Handling**: All scripts use `set -e` for error propagation
- **Input Validation**: Validate inputs and check prerequisites
- **Logging**: Consistent logging with colored output
- **Documentation**: Clear documentation and usage examples

### Security
- **Input Sanitization**: Validate and sanitize all inputs
- **Permission Checks**: Verify required permissions before execution
- **Secure Defaults**: Use secure default configurations
- **Audit Trail**: Log all significant operations

### Maintainability
- **Modular Design**: Break complex scripts into functions
- **Consistent Naming**: Use descriptive, consistent naming conventions
- **Configuration**: Externalize configuration to environment files
- **Version Control**: Track script changes and versions

## 📋 Usage Examples

### Setup Environment
```bash
# Setup testing environment
./scripts/setup/setup_toka_testing.sh

# Setup Docker environments
./scripts/setup/setup-docker-environments.sh
```

### Run Tests
```bash
# Run comprehensive agent tests
./scripts/testing/test_toka_agents.sh

# Run simple functionality test
./scripts/testing/run_simple_test.sh
```

### Execute Workflows
```bash
# Run complete system workflow
./scripts/workflow/toka_workflow.sh

# Start interactive CLI
./scripts/workflow/toka_interactive.sh
```

### Monitor System
```bash
# Start Raft monitoring
./scripts/monitoring/raft_monitoring_service.sh start

# Check monitoring status
./scripts/monitoring/raft_monitoring_service.sh status
```

## 🔧 Script Development

### Adding New Scripts
1. **Choose Category**: Place scripts in appropriate subdirectory
2. **Follow Standards**: Use consistent formatting and error handling
3. **Add Documentation**: Include clear usage instructions
4. **Update Index**: Add to this README with description

### Script Template
```bash
#!/bin/bash

# Script Name - Brief Description
# Detailed description of purpose and usage

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_NAME="$(basename "$0")"
LOG_FILE="./logs/${SCRIPT_NAME%.*}.log"

# Functions
print_status() {
    echo -e "${1}${2}${NC}"
}

# Main execution
main() {
    print_status $BLUE "Starting $SCRIPT_NAME..."
    # Script logic here
    print_status $GREEN "Completed successfully"
}

# Run main function
main "$@"
```

## 🔗 Related Documentation

- [Development Guide](../docs/development/) - Development workflows
- [Operations Guide](../docs/operations/) - Deployment and monitoring
- [Testing Guide](../docs/development/TOKA_TESTING_GUIDE.md) - Testing strategies

---

*This scripts directory is maintained as part of the Toka project's commitment to clear, accurate, and well-organized system management.* 