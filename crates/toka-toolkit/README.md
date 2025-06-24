# Toka Toolkit

_Batteries-included collection of async **tools** + a `ToolRegistry` implementation._

---

## Purpose

* Offer **reference implementations** (ingestion, ledger, semantic index, coverage, …) that agents can invoke via capability-checked calls.
* Provide a **type-safe plugin system** (`Tool` trait) so you can register your own domain-specific helpers without modifying the runtime.
* Keep the core optional – if you only need the trait definitions use [`toka-toolkit-core`](../toka-toolkit-core/README.md).

---

## Default Tools

| Name | What it does |
|------|--------------|
| `echo` | Minimal PoC – echoes parameters back |
| `ingestion` | Validate CSV/TSV/JSON and convert to standardised CBOR |
| `ledger` | Reconcile credit / debit transactions |
| `reporting` | Generate financial summaries |
| `scheduling` | Lightweight async task scheduler |
| `semantic_index` | Tag & search arbitrary items |
| `coverage-json` / `coverage-analyse` | Help improve test coverage |

All are **optional** – you can start with an empty registry and add only what you need.

---

## Relationship to the Event Store

Tool executions emit `ToolEvent`s on the **in-process `EventBus`** which the runtime can then persist in [`toka-vault`](../toka-vault/README.md).  This historical log is critical for auditability and future semantic search.

---

## Example

```rust,ignore
use toka_toolkit::{ToolParams, ToolRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = ToolRegistry::new().await?;

    let mut args = std::collections::HashMap::new();
    args.insert("message".into(), "hello".into());
    let params = ToolParams { name: "echo".into(), args };
    let res = registry.execute_tool("echo", &params).await?;
    assert_eq!(res.output, "hello");
    Ok(())
}
```

---

## Feature Matrix

_No extra feature flags – heavy optional deps live behind the individual tool crates they pull in._

---

## License

Apache-2.0 OR MIT

© 2024 Toka Contributors 