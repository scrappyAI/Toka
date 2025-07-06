# Toka Control Flow Analysis Report

Generated on: Sun Jul  6 09:00:27 UTC 2025

## Overview

- Total functions analyzed: 1154
- Total source files: 104
- Async functions: 508
- Component interactions: 7

## Complexity Analysis

- Average cyclomatic complexity: 1.71
- Max cyclomatic complexity: 47
- Functions with complexity > 10: 3

## Most Complex Functions

- **test_error_framework_events**: 47 (in tests/integration/kernel_events_v0_3.rs)
- **validate**: 13 (in crates/toka-tools/src/manifest.rs)
- **process_append_entries_request**: 11 (in crates/raft-core/src/node.rs)
- **sanitize**: 10 (in crates/toka-llm-gateway/src/sanitizer.rs)
- **apply_operation**: 10 (in tests/integration/property_based.rs)
- **print_summary**: 10 (in tests/integration/mod.rs)
- **execute**: 9 (in tests/integration/agent_lifecycle.rs)
- **detect_circular_dependencies**: 8 (in crates/toka-orchestration/src/dependency.rs)
- **handle_append_entries_response**: 8 (in crates/raft-core/src/node.rs)
- **validate_file_path**: 8 (in crates/toka-config-cli/src/validation.rs)

## Async Patterns

- Simple Async: 327 functions
- Sequential Async: 72 functions
- Spawn And Await: 8 functions

## Component Interactions

- **orchestration** interacts with: llm, cli, runtime, storage
- **kernel** interacts with: storage, llm
- **cli** interacts with: llm, storage
- **unknown** interacts with: llm, runtime, storage
- **runtime** interacts with: llm, storage, kernel
- **storage** interacts with: cli, llm
- **bus** interacts with: storage, llm
