# toka-runtime

Toka Runtime – orchestrates agents, tools, event bus, and vault

This crate wires together the core subsystems (`toka-agents`, `toka-tools`,
`toka-bus`, `toka-events`, etc.) into a cohesive, async runtime.

## Features

| Feature | Purpose | Additional crates |
|---------|---------|-------------------|
| `toolkit` *(opt)* | Enables [`ToolRegistry`](crate::tools) & default tools | `toka-toolkit-core`, `toka-tools` |
| `auth` *(opt)*    | Capability-token validation & secret rotation | `toka-security-auth`, `jsonwebtoken` |
| `vault` *(opt)*   | Embed the canonical event store | `toka-events` + `sled` |

### Quick-Start
```rust
let rt = Runtime::new(RuntimeConfig::default()).await?;
rt.start().await?;
```

Built with the ergonomics of LLM-driven agent development in mind.
