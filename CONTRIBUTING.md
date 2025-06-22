# Contributing to Toka

Thank you for your interest in contributing to Toka! This document provides guidelines and information to help you get started.

## Getting Started

### Prerequisites

- **Rust**: Toka requires Rust 1.70 or later. You can install Rust using [rustup](https://rustup.rs/).
- **Git**: Make sure you have Git installed and configured.

### Setting Up Your Development Environment

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/toka.git
   cd toka
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run the tests:**
   ```bash
   cargo test
   ```

## Development Workflow

### Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to check for common issues and improvements
- Ensure all code is documented with doc comments (`///`)

### Testing

- Write tests for new functionality
- Ensure all existing tests pass
- Run the full test suite before submitting a pull request:
  ```bash
  cargo test --workspace
  ```

### Commit Messages

Use clear, descriptive commit messages. Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat: add new agent creation functionality
fix: resolve issue with token validation
docs: update README with usage examples
test: add tests for vault operations
```

## Making Changes

### 1. Create a Feature Branch

Always work on a feature branch:

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clear, well-documented code
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
```

### 4. Submit a Pull Request

1. Push your branch to your fork
2. Create a pull request against the main branch
3. Provide a clear description of your changes
4. Reference any related issues

## Project Structure

Toka is organized as a workspace with multiple crates:

- `toka`: Main meta-crate that re-exports commonly used components
- `toka-core`: Core business logic and domain models
- `toka-agents`: Agent implementation
- `toka-runtime`: Runtime for agents and tools
- `toka-toolkit`: Collection of standard tools
- `toka-security-auth`: Authentication primitives
- `toka-secrets`: Secure vault implementation
- `toka-events`: Event system
- `toka-primitives`: Basic data types
- `toka-cli`: Command-line interface
- `toka-storage`: Storage abstractions

## Areas for Contribution

### High Priority

- **CLI Implementation**: The CLI currently has placeholder implementations. Help wire up the commands to the runtime.
- **Tool Development**: Add new tools to the toolkit
- **Documentation**: Improve existing documentation or add new guides
- **Testing**: Add more comprehensive test coverage

### Medium Priority

- **Performance**: Optimize existing code
- **Error Handling**: Improve error messages and handling
- **Configuration**: Add configuration management
- **Monitoring**: Add observability and metrics

### Low Priority

- **Examples**: Create example applications
- **Benchmarks**: Add performance benchmarks
- **CI/CD**: Improve the continuous integration setup

## Reporting Issues

When reporting issues, please include:

- A clear description of the problem
- Steps to reproduce the issue
- Expected vs. actual behavior
- Your environment (OS, Rust version, etc.)
- Any relevant error messages or logs

## Security

If you discover a security vulnerability, please **do not** open a public issue. Instead, send a detailed report to `agentix.tech@gmail.com`.

## Code of Conduct

This project is committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and considerate of others.

## Getting Help

If you have questions or need help:

1. Check the existing documentation
2. Search existing issues and pull requests
3. Open a new issue for questions or discussions

## License

By contributing to Toka, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

Thank you for contributing to Toka! 