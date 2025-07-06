# Toka Control Flow Analysis Report

Generated on: Sun Jul  6 16:26:41 UTC 2025

## Overview

- Total functions analyzed: 1200
- Total source files: 107
- Async functions: 528
- Component interactions: 7

## Complexity Analysis

- Average cyclomatic complexity: 1.69
- Max cyclomatic complexity: 47
- Functions with complexity > 10: 2

## Most Complex Functions

- **test_error_framework_events**: 47 (in tests/integration/kernel_events_v0_3.rs)
- **process_append_entries_request**: 11 (in crates/raft-core/src/node.rs)
- **sanitize**: 10 (in crates/toka-llm-gateway/src/sanitizer.rs)
- **print_summary**: 10 (in tests/integration/mod.rs)
- **apply_operation**: 10 (in tests/integration/property_based.rs)
- **detect_circular_dependencies**: 8 (in crates/toka-orchestration/src/dependency.rs)
- **handle_append_entries_response**: 8 (in crates/raft-core/src/node.rs)
- **validate_file_path**: 8 (in crates/toka-config-cli/src/validation.rs)
- **validate_key_path**: 8 (in crates/toka-config-cli/src/validation.rs)
- **infer_capabilities**: 8 (in crates/toka-tools/src/wrappers/discovery.rs)

## Async Patterns

- Simple Async: 337 functions
- Sequential Async: 74 functions
- Spawn And Await: 7 functions

## Component Interactions

- **cli** interacts with: llm, storage
- **kernel** interacts with: llm, storage
- **orchestration** interacts with: runtime, cli, llm, storage
- **unknown** interacts with: llm, runtime, storage
- **runtime** interacts with: llm, storage, kernel
- **storage** interacts with: cli, llm
- **bus** interacts with: storage, llm
