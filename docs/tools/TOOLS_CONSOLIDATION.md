# Tools Directory Consolidation Plan

## Problem Statement

The current setup has two separate tool systems:
1. **`@/tools/` directory** - Python scripts, shell scripts, YAML manifests, music system
2. **`toka-tools` crate** - Rust-based kernel-enforced tool system

This creates fragmentation, maintenance overhead, and goes against the core Rust-first architecture.

## Consolidation Strategy

### Phase 1: Preserve Valuable Functionality ✅

**Moved to Rust system:**
- ✅ Date validation logic → `crates/toka-tools/src/tools/validation.rs`
- ✅ Build validation logic → `crates/toka-tools/src/tools/validation.rs`  
- ✅ Tool manifests → `crates/toka-tools/manifests/`

**Benefits:**
- Kernel-enforced security
- Type safety and performance 
- Unified tool discovery
- Integrated with agent runtime

### Phase 2: Remove Fragmented Directory

**Safe to remove:**
- `tools/music/` - Complex Python system not core to Toka
- `tools/analysis/` - Python analysis tools (can be reimplemented in Rust if needed)
- `tools/agents/` - YAML agent specs (handled by core system)
- `tools/system/` - Shell scripts (functionality moved to Rust tools)
- `tools/validation/` - Python validation (replaced by Rust implementation)

### Phase 3: Migration Commands

```bash
# Remove the entire tools directory
rm -rf tools/

# The functionality is now available through:
# 1. Rust tools in toka-tools crate
# 2. Tool manifests in crates/toka-tools/manifests/
# 3. Agent system in core runtime
```

## New Architecture

```
Toka System
├── crates/toka-tools/           # Unified tool system
│   ├── src/tools/              # Built-in Rust tools
│   │   ├── validation.rs       # Date & build validation
│   │   ├── file_tools.rs       # File operations
│   │   └── text_tools.rs       # Text processing
│   ├── manifests/              # Tool definitions
│   │   ├── date-validator.yaml
│   │   └── build-validator.yaml
│   └── src/runtime_integration.rs # Agent integration
├── crates/toka-kernel/         # Security enforcement
├── crates/toka-runtime/        # Dynamic execution
└── crates/toka-agent-runtime/  # Agent orchestration
```

## Benefits of Consolidation

### 1. **Single Source of Truth**
- All tools managed through `toka-tools` crate
- Unified discovery and execution
- Consistent security model

### 2. **Performance & Safety**
- Rust performance vs Python
- Type safety at compile time
- Memory safety guaranteed

### 3. **Security**
- Kernel-enforced capabilities
- Sandboxing and resource limits
- Audit logging built-in

### 4. **Maintainability**
- Single toolchain (Rust)
- Consistent error handling
- Integrated testing

### 5. **Composability**
- Tools can be chained and composed
- Model integration as "brains"
- Dynamic hot-swapping

## Usage Examples

### Before (Fragmented)
```bash
# Python script
python tools/validation/validate_dates.py --path .

# Shell script  
bash tools/system/validate-build-system.sh

# Separate music system
cd tools/music && python orchestrator/music_playlist_orchestrator.py
```

### After (Unified)
```rust
use toka_tools::{ToolRegistry, ToolParams};

// All tools through unified system
let registry = ToolRegistry::new().await?;
toka_tools::tools::register_essential_tools(&registry).await?;

// Date validation
let mut params = ToolParams::new("date-validator");
params.set("path", ".");
params.set("fix_violations", true);
let result = registry.execute_tool("date-validator", &params).await?;

// Build validation
let mut params = ToolParams::new("build-validator");
params.set("workspace_path", ".");
let result = registry.execute_tool("build-validator", &params).await?;
```

## Migration Checklist

- [x] Extract date validation logic to Rust
- [x] Extract build validation logic to Rust  
- [x] Create unified tool manifests
- [x] Update tool registry with validation tools
- [ ] Remove `@/tools/` directory
- [ ] Update documentation
- [ ] Update CI/CD pipelines
- [ ] Test unified system

## Decision Rationale

The separate `@/tools/` directory violates the core principle of:
> "Focus on the core Rust system where models can be wired in as 'brains' and tools built and composed as needed"

By consolidating into the Rust-based `toka-tools` system:
1. **Alignment** - Everything goes through the kernel security model
2. **Simplicity** - Single tool system to learn and maintain  
3. **Performance** - Rust performance for all operations
4. **Composability** - Tools can be combined and orchestrated
5. **Model Integration** - Clear path for AI model integration

## Execution

Ready to remove the `@/tools/` directory. All valuable functionality has been preserved in the Rust system. 