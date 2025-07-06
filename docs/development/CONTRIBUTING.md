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
- `toka-capability-core` + `toka-capability-jwt-hs256`: Capability token primitives
- `toka-secrets`: Secure vault implementation