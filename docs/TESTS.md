# Toka Testing Guide

## Overview
Comprehensive testing strategies for the Toka workspace.

## Test Categories
- **Unit Tests**: `cargo test --lib` - Test individual modules and functions
- **Integration Tests**: `cargo test --test` - Test component interactions  
- **End-to-End Tests**: `cargo test --package toka-orchestration` - Test full workflows
- **Performance Tests**: `cargo bench` - Benchmark critical paths

## Coverage Reports
```bash
# Generate HTML coverage report
cargo tarpaulin --workspace --out Html --output-dir target/coverage

# View coverage in browser
open target/coverage/tarpaulin-report.html
```

## Test Configuration
- Test environments in `config/testing/`
- Test data in `tests/data/`
- Mock services in `tests/mocks/`

## Running Specific Tests
```bash
# Test specific crate
cargo test --package toka-agent-runtime

# Test with logging
RUST_LOG=debug cargo test

# Test with specific features
cargo test --features llm-integration
```

## Test Best Practices
- Use descriptive test names
- Test both success and failure cases
- Mock external dependencies
- Use property-based testing for complex logic
- Maintain test data integrity
