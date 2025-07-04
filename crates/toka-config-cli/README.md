# Toka Config CLI

A robust command-line tool for managing configuration files in YAML, JSON, and TOML formats. Part of the Toka ecosystem, this CLI provides comprehensive configuration management with validation, error handling, and security features.

## Features

- ✅ **Multi-format support**: YAML (.yml, .yaml), JSON (.json), and TOML (.toml)
- ✅ **CRUD operations**: Create, read, update, and delete configuration files and keys
- ✅ **Nested key support**: Use dot notation to access and modify nested configuration values
- ✅ **Validation**: Comprehensive syntax and structure validation
- ✅ **Security**: Input sanitization and path traversal protection
- ✅ **Error handling**: Rich error messages with helpful tips
- ✅ **Logging**: Structured logging with configurable levels
- ✅ **Directory listing**: Find and validate configuration files in directories

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/ScrappyAI/toka.git
cd toka/crates/toka-config-cli

# Build and install
cargo build --release
cargo install --path .
```

### Using cargo

```bash
cargo install toka-config-cli
```

## Usage

### Basic Commands

#### Create a configuration file

```bash
# Create a JSON configuration file
toka-config create --file config.json --format json --content '{"app": {"name": "MyApp", "version": "1.0.0"}}'

# Create a YAML configuration file
toka-config create --file config.yaml --format yaml --content '{"database": {"host": "localhost", "port": 5432}}'

# Create a TOML configuration file
toka-config create --file config.toml --format toml --content '{"server": {"port": 8080, "workers": 4}}'
```

#### Read a configuration file

```bash
toka-config read --file config.json
```

Output:
```
📄 Configuration file: config.json
📋 Format: JSON
📊 Content:
{
  "app": {
    "name": "MyApp",
    "version": "1.0.0"
  }
}
```

#### Update configuration values

```bash
# Update a simple key
toka-config update --file config.json --key app.name --value '"NewAppName"'

# Update a nested key
toka-config update --file config.json --key app.settings.debug --value 'true'

# Update with complex values
toka-config update --file config.json --key features --value '["auth", "logging", "cache"]'
```

#### Delete configuration keys

```bash
# Delete a specific key
toka-config delete --file config.json --key app.version

# Delete a nested section
toka-config delete --file config.json --key app.settings
```

#### Validate configuration files

```bash
toka-config validate --file config.json
```

Output:
```
✅ Configuration file is valid: config.json
📋 Format: JSON
📊 Keys found: 3
```

#### List configuration files

```bash
# List all configuration files in current directory
toka-config list

# List configuration files in specific directory
toka-config list --directory /path/to/configs
```

Output:
```
📁 Configuration files in: /path/to/configs
   Found 3 configuration file(s):
   ✅ Valid config.json (JSON)
   ✅ Valid app.yaml (YAML)
   ❌ Invalid broken.toml (TOML)
```

### Advanced Usage

#### Working with different formats

The tool automatically detects file format based on extension:

```bash
# YAML files (.yml or .yaml)
toka-config create --file app.yml --format yaml --content '{"name": "test"}'

# JSON files (.json)
toka-config create --file app.json --format json --content '{"name": "test"}'

# TOML files (.toml)
toka-config create --file app.toml --format toml --content '{"name": "test"}'
```

#### Nested key operations

Use dot notation to work with nested structures:

```bash
# Create nested configuration
toka-config create --file app.json --format json --content '{
  "database": {
    "mysql": {
      "host": "localhost",
      "port": 3306,
      "credentials": {
        "username": "root",
        "password": "secret"
      }
    }
  }
}'

# Update deeply nested values
toka-config update --file app.json --key database.mysql.credentials.password --value '"new_password"'

# Delete nested sections
toka-config delete --file app.json --key database.mysql.credentials
```

#### Logging configuration

Control the verbosity of output:

```bash
# Set log level to debug for detailed information
toka-config --log-level debug read --file config.json

# Set log level to error for minimal output
toka-config --log-level error validate --file config.json
```

Available log levels:
- `trace`: Most verbose, shows all operations
- `debug`: Detailed information for debugging
- `info`: General information (default)
- `warn`: Warning messages only
- `error`: Error messages only

### Configuration File Examples

#### JSON Configuration

```json
{
  "app": {
    "name": "MyApplication",
    "version": "2.1.0",
    "environment": "production"
  },
  "database": {
    "host": "db.example.com",
    "port": 5432,
    "name": "myapp_db",
    "ssl": true
  },
  "features": ["auth", "logging", "monitoring"],
  "limits": {
    "max_connections": 100,
    "timeout": 30
  }
}
```

#### YAML Configuration

```yaml
app:
  name: MyApplication
  version: 2.1.0
  environment: production

database:
  host: db.example.com
  port: 5432
  name: myapp_db
  ssl: true

features:
  - auth
  - logging
  - monitoring

limits:
  max_connections: 100
  timeout: 30
```

#### TOML Configuration

```toml
[app]
name = "MyApplication"
version = "2.1.0"
environment = "production"

[database]
host = "db.example.com"
port = 5432
name = "myapp_db"
ssl = true

features = ["auth", "logging", "monitoring"]

[limits]
max_connections = 100
timeout = 30
```

## Security Features

- **Path traversal protection**: Prevents `../` attacks
- **Input validation**: Validates file paths and key names
- **Extension validation**: Only allows supported file formats
- **Safe key names**: Prevents reserved keywords and invalid characters
- **Depth limiting**: Prevents excessive nesting in key paths

## Error Handling

The tool provides helpful error messages and suggestions:

```bash
❌ Error: Configuration file not found: /path/to/config.json
💡 Tip: Check if the file path '/path/to/config.json' is correct

❌ Error: Invalid configuration format for file 'config.xyz': Unsupported file extension: xyz
💡 Tip: Supported formats are YAML (.yml, .yaml), JSON (.json), and TOML (.toml)

❌ Error: Invalid key path 'user.1name': Key segment '1name' cannot start with a number
```

## Integration Examples

### CI/CD Pipeline

```bash
# Validate all configuration files before deployment
find . -name "*.json" -o -name "*.yaml" -o -name "*.yml" -o -name "*.toml" | \
while read file; do
  echo "Validating $file..."
  toka-config validate --file "$file" || exit 1
done
```

### Environment-specific Configurations

```bash
# Update configuration for different environments
toka-config update --file config.json --key app.environment --value '"development"'
toka-config update --file config.json --key database.host --value '"localhost"'
toka-config update --file config.json --key app.debug --value 'true'
```

### Backup and Migration

```bash
# Read and backup configuration
toka-config read --file old-config.json > backup.json

# Convert between formats
toka-config read --file config.yaml # Read YAML
toka-config create --file config.json --format json --content "$(toka-config read --file config.yaml)"
```

## CLI Reference

### Global Options

- `--log-level <LEVEL>`: Set logging verbosity (trace, debug, info, warn, error)
- `--help`: Show help information
- `--version`: Show version information

### Commands

#### `create`
Create a new configuration file.

**Arguments:**
- `-f, --file <PATH>`: Path to the configuration file to create
- `-t, --format <FORMAT>`: Format of the configuration file (yaml, json, toml)
- `-c, --content <JSON>`: Initial content as JSON string (default: "{}")

#### `read`
Read and display a configuration file.

**Arguments:**
- `-f, --file <PATH>`: Path to the configuration file to read

#### `update`
Update a key-value pair in a configuration file.

**Arguments:**
- `-f, --file <PATH>`: Path to the configuration file to update
- `-k, --key <KEY>`: Key to update (supports dot notation)
- `-v, --value <JSON>`: New value as JSON string

#### `delete`
Delete a key from a configuration file.

**Arguments:**
- `-f, --file <PATH>`: Path to the configuration file
- `-k, --key <KEY>`: Key to delete (supports dot notation)

#### `validate`
Validate the syntax and structure of a configuration file.

**Arguments:**
- `-f, --file <PATH>`: Path to the configuration file to validate

#### `list`
List all configuration files in a directory.

**Arguments:**
- `-d, --directory <PATH>`: Directory to search (default: current directory)

## Contributing

Contributions are welcome! Please ensure all code follows the project's guidelines:

- Security first: All code must be safe and resilient
- Clear documentation: Every public API must be documented
- Comprehensive testing: Include unit and integration tests
- Error handling: Never ignore `Result` or `Option`

## License

This project is licensed under the Apache License 2.0. See the LICENSE file for details.

## Support

For issues, feature requests, or questions:

1. Check the [GitHub Issues](https://github.com/ScrappyAI/toka/issues)
2. Create a new issue with detailed information
3. Include example commands and error messages when applicable

## Changelog

### v0.2.1
- Initial release
- Support for JSON, YAML, and TOML formats
- CRUD operations with dot notation
- Comprehensive validation and error handling
- Security features and input sanitization
- Structured logging and helpful error messages