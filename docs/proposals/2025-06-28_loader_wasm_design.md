# Tool Loader – WASM Transport Design Proposal

> Version: 0.1 – 2025-06-28  
> Author: Automated refactor agent  
> Status: DRAFT (open for review)

## Context

`Transport::Wasm` in `toka-tools::loader` is currently a stub that carries a `path` string which is **never read**.  The original intention was to allow loading pre-compiled `.wasm` binaries or sources compiled on the fly (via `wasmtime`). With the recent refactor we removed the unused field but kept the variant – now is the right moment to formalise its semantics.

### Goals

1.  Provide an ergonomic API for registering and executing WASM-based tools.
2.  Keep the default build lightweight; WASM support must be **opt-in** via the `wasm` feature.
3.  Allow both in-process execution (via `wasmtime`) and host-executed modules (sandbox, OCI, etc.).
4.  Maintain clear separation between loader, runtime and storage layers per @60_toka-workspace-evolution.mdc.

## Proposed API Changes

```rust
//! crates/toka-tools/src/loader.rs

/// How a tool implementation is delivered and executed.
#[non_exhaustive]
pub enum Transport<'a> {
    /// Static Rust implementation linked into the binary.
    Native { entry: &'a dyn Tool },
    /// WebAssembly module executed with `wasmtime` inside the current process.
    ///
    /// * `module_bytes` may come from the filesystem, S3 or a remote URL – the
    ///   loader takes care of fetching & caching.
    /// * `engine` and `linker` are shared between invocations for perf.
    WasmInProc {
        module_bytes: Cow<'a, [u8]>,
        wasi: WasiConfig,
    },
    /// WASI-compatible module executed in a dedicated sandbox process (e.g.
    /// Wasmtime CLI, Containerd shim).
    WasmSandbox {
        module_path: PathBuf,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
}
```

### Feature Flags

* `wasm` – enables all variants above and pulls in `wasmtime` + `wasi-common`.
* `wasm-sandbox` – additionally depends on `tokio::process` and the *chosen* sandbox runner crate.

### Loader Flow

1.  When a tool manifest specifies `transport = "wasm"`, the loader resolves:
    1. local file path ("./tools/echo.wasm")
    2. named artifact in `toka-storage`
    3. remote URL (download + cache)
2.  The loader validates that the module exports a `_start` entry complying with WASI.
3.  On first invocation the module is compiled and stored in the in-memory `ModuleCache`.
4.  Parameters are passed via STDIN as JSON; result is read from STDOUT.
5.  Streaming progress is surfaced through the existing `ToolEvent` bus.

## Backwards Compatibility

* The current `Transport::Native` path remains untouched.
* Crates **without** the `wasm` feature do not build or link against `wasmtime`.
* Existing manifests that prematurely used `wasm` will now load correctly.

## Open Questions

* Do we need a shared WASI host preview2 implementation?
* Should we support WASIX for async networking right away or keep MVP synchronous?
* Where should we store compiled module artefacts? – `~/.cache/toka/wasm/` by default.

---

Please review and leave comments inline.  Once approved, follow-up tasks will track incremental implementation slices:

1. Introduce `wasm` feature & dependency scaffolding.
2. Implement `WasmInProc` path with minimal echo tool.
3. Integrate with CLI: `toka tool run wasm://echo --payload '{...}'`.