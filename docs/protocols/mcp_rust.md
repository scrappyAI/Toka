# Model Context Protocol – Rust Guidance

> Protocol: MCP v2025-03-26 – see upstream spec for canonical details.
>
> Local doc path: `docs/protocols/mcp_rust.md`

This page summarises *how* each crate in the workspace should implement MCP
support.  It is **derived from** the [official specification][spec].  If you
spot divergence, open a *Docs Update* PR _first_.

[spec]: https://modelcontextprotocol.io/specification/2025-03-26

---

## Quick Checklist (cheat-sheet)

1. Identify the intent: *Tool / Resource / Prompt*.
2. Add JSON-RPC 2.0 method under the namespace:
   * `tools/{capability}` for tool calls
   * `resources/*`   for data fetchers
   * `prompts/*`     for reusable prompts
3. Supply a **JSON Schema** for params & result.
4. Implement the `initialize` handshake for capability negotiation.
5. Emit `tool_progress` notifications if the operation may exceed 500 ms.
6. Record a transcript and run `mcp-lint` (CI target `mcp_conformance`).

## Version Matrix

| MCP Spec | Status in Toka | Notes |
|----------|----------------|-------|
| 2025-03-26 | **current** | Default target for new work |
| <2025-07-11 | deprecated  | Submit a *spec bump* PR to remove |

## Code Patterns

```rust
//! Protocol: MCP v2025-03-26 (see docs/protocols/mcp_rust.md)
use tokio_json_rpc::{Server, Request};
use serde_json::Value;

async fn echo(req: Request<Value>) -> anyhow::Result<Value> {
    #[derive(serde::Deserialize)]
    struct Params { text: String }
    let p: Params = req.parse()?;
    Ok(serde_json::json!({ "text": p.text }))
}
```

## Security Notes

* Authenticate using OAuth2 **Bearer** tokens (as per § 8 of the spec).
* Reject any request lacking `Authorization` unless the tool is explicitly
  marked **public** in its [`ToolManifest`](../../crates/toka-tools/src/manifest.rs).

## Testing

Run conformance tests locally:

```bash
cargo test -p my_crate --features "mcp_conformance"
```

## Example Schemas

Below is a minimal set of **draft-07** JSON Schemas you can copy-paste when
authoring a simple *echo* style tool.  They pass the new deep-validation logic
introduced in `toka-tools` v0.1.1 and illustrate common best practices:

```jsonc
"input_schema": {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["text"],
  "properties": {
    "text": { "type": "string", "minLength": 1 }
  }
},
"output_schema": {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["echo"],
  "properties": {
    "echo": { "type": "string" }
  }
}
```

### Guidelines

* **Keep it small** – the tooling enforces a hard 64 KiB limit per schema.
* **Avoid remote `$ref`** – they're disabled by default for security.  Enable
  with the `allow_remote_refs` crate feature only if absolutely necessary.
* **Draft compatibility** – target draft-07 today; *draft-2020-12* support will
  land once the upstream crate is stable.

See the *Tool Development Guidelines* for a broader discussion on capability
design and side-effect classification. 