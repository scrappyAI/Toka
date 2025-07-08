# Python Environment Management

This document describes the Python environment management system for the universal extension framework.

## Overview

The Python environment management system provides tools and scripts to handle virtual environments, dependencies, and cleanup operations in a deterministic and reproducible manner.

## Components

### 1. Python Environment Manager (`scripts/python-env.py`)

A comprehensive tool for managing Python virtual environments and dependencies.

#### Features
- Virtual environment creation and cleanup
- Dependency installation from requirements.txt
- Environment status checking
- Script execution in virtual environment
- Automatic cleanup on recreation

#### Commands

```bash
# Create a new virtual environment
python3 scripts/python-env.py create

# Force recreate virtual environment
python3 scripts/python-env.py create --force

# Install dependencies
python3 scripts/python-env.py install

# Check environment status
python3 scripts/python-env.py check

# Get activation command
python3 scripts/python-env.py activate

# Run a script in the environment
python3 scripts/python-env.py run --script scripts/date-enforcer.py

# Clean up virtual environment
python3 scripts/python-env.py cleanup
```

#### Output Formats

The tool supports both JSON and text output formats:

```bash
# JSON output (default)
python3 scripts/python-env.py check

# Text output
python3 scripts/python-env.py check --output text
```

### 2. Python Environment Cleanup (`scripts/cleanup-python-env.py`)

A specialized script for cleaning up virtual environment remnants and managing environment properly.

#### Features
- Detection of virtual environment remnants
- Safe cleanup with dry-run mode
- Automatic creation of new environments
- Size calculation and reporting

#### Usage

```bash
# Dry run to see what would be cleaned
python3 scripts/cleanup-python-env.py --dry-run

# Actually perform cleanup
python3 scripts/cleanup-python-env.py --force

# Clean up and create new environment
python3 scripts/cleanup-python-env.py --force --create-venv

# Search in specific directory
python3 scripts/cleanup-python-env.py --directory /path/to/search
```

#### Remnant Detection

The script detects various types of virtual environment remnants:

**Directory Names:**
- `venv`, `env`, `.venv`, `.env`
- `virtualenv`, `python_env`, `python-env`, `pyenv`

**Configuration Files:**
- `pyvenv.cfg`, `activate`, `activate.bat`, `activate.ps1`
- `python.exe`, `python`, `pip.exe`, `pip`

**Structure Detection:**
- `bin/`, `lib/`, `lib64/`, `Scripts/` directories
- Presence of activation scripts or configuration files

### 3. Tool Schema (`tools/python-env.json`)

JSON schema defining the Python environment management tool for the universal extension system.

#### Configuration

```json
{
  "venvDirectory": "venv",
  "requirementsFile": "requirements.txt",
  "pythonVersion": "3.8",
  "autoCleanup": true
}
```

#### Commands

The tool schema defines all available commands with their arguments and output formats:

- `create`: Create new virtual environment
- `install`: Install dependencies
- `cleanup`: Clean up environment
- `activate`: Get activation command
- `check`: Check environment status
- `run`: Run script in environment

## Best Practices

### 1. Environment Isolation

Always use virtual environments to isolate dependencies:

```bash
# Create environment
python3 scripts/python-env.py create

# Install dependencies
python3 scripts/python-env.py install

# Check status
python3 scripts/python-env.py check
```

### 2. Dependency Management

Keep `requirements.txt` updated with all necessary dependencies:

```txt
click>=8.0.0
python-dateutil>=2.8.0
gitpython>=3.1.0
jsonschema>=4.0.0
```

### 3. Cleanup Procedures

Regularly clean up old environments:

```bash
# Check for remnants
python3 scripts/cleanup-python-env.py --dry-run

# Clean up and recreate
python3 scripts/cleanup-python-env.py --force --create-venv
```

### 4. Script Execution

Use the environment manager to run scripts:

```bash
# Run a script in the virtual environment
python3 scripts/python-env.py run --script scripts/date-enforcer.py

# With arguments
python3 scripts/python-env.py run --script scripts/date-enforcer.py --args --validate
```

## Integration with Universal Extension System

### Tool Registration

The Python environment manager is registered in the tool registry:

```bash
# Register the tool
python3 scripts/tool-registry.py register tools/python-env.json

# List registered tools
python3 scripts/tool-registry.py list
```

### Date Enforcement Integration

The Python environment manager integrates with the date enforcement system:

```bash
# Run date enforcer in managed environment
python3 scripts/python-env.py run --script scripts/date-enforcer.py --args --validate

# Check date enforcement status
python3 scripts/python-env.py run --script scripts/date-enforcer.py --args --status
```

### Folder Tidy Integration

The cleanup script integrates with the folder tidy system:

```bash
# Use folder tidy to clean Python environments
python3 scripts/folder-tidy.py --analyze --target venv

# Clean up with folder tidy
python3 scripts/folder-tidy.py --clean --target venv --backup
```

## Troubleshooting

### Common Issues

1. **Virtual Environment Not Found**
   ```bash
   # Check if environment exists
   python3 scripts/python-env.py check
   
   # Recreate if needed
   python3 scripts/python-env.py create --force
   ```

2. **Dependencies Not Installed**
   ```bash
   # Install dependencies
   python3 scripts/python-env.py install
   
   # Check installed packages
   python3 scripts/python-env.py check
   ```

3. **Permission Issues**
   ```bash
   # Clean up and recreate with proper permissions
   python3 scripts/cleanup-python-env.py --force --create-venv
   ```

4. **Python Version Mismatch**
   ```bash
   # Check Python version
   python3 scripts/python-env.py check
   
   # Recreate with specific version (if supported)
   # Update configuration in tools/python-env.json
   ```

### Debugging

Enable verbose output for debugging:

```bash
# Check environment with detailed output
python3 scripts/python-env.py check --output text

# Clean up with verbose output
python3 scripts/cleanup-python-env.py --force --output text
```

## Security Considerations

### Environment Isolation

- Virtual environments provide isolation from system Python
- Dependencies are contained within the environment
- No system-wide package installation

### Dependency Security

- Use specific version requirements in `requirements.txt`
- Regularly update dependencies for security patches
- Audit dependencies for known vulnerabilities

### Cleanup Security

- Dry-run mode prevents accidental deletion
- Backup creation before cleanup operations
- Audit trails for all cleanup operations

## Performance Optimization

### Environment Size

- Regular cleanup reduces disk usage
- Remove unused dependencies
- Use minimal requirements files

### Startup Time

- Keep virtual environments in fast storage
- Minimize dependency count
- Use pre-built environments for CI/CD

## Future Enhancements

### Planned Features

1. **Multi-Environment Support**
   - Support for multiple Python versions
   - Environment switching
   - Version-specific requirements

2. **Dependency Resolution**
   - Automatic dependency conflict resolution
   - Lock file generation
   - Dependency graph visualization

3. **CI/CD Integration**
   - Automated environment setup
   - Cache management
   - Build optimization

4. **Monitoring and Metrics**
   - Environment usage tracking
   - Performance metrics
   - Dependency update notifications

### Extension Points

The system is designed for extensibility:

- Custom environment types
- Alternative package managers
- Integration with other tools
- Plugin system for specialized workflows

## Conclusion

The Python environment management system provides a robust, secure, and efficient way to handle Python environments in the universal extension framework. By following the best practices and using the provided tools, you can ensure consistent, reproducible, and maintainable Python environments. 