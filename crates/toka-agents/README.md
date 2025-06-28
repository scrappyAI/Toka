# Toka Agents

Reference agent implementations for the Toka platform – ready-made shells you can drop into [`toka-runtime`](../toka-runtime/README.md).

---

## Why does this crate exist?

* Provide **batteries-included agents** (e.g. `SymbolicAgent`) so you can try the platform without writing code.
* Demonstrate how to compose the **event-sourced** architecture – every belief update, plan and tool call is recorded in [`toka-events`](../toka-events/README.md).
* Serve as a test-bed for multiple reasoning engines (symbolic, LLM, sandbox) hidden behind a single `ReasoningEngine` trait.

---

## Capabilities & Feature Flags

| Feature | Purpose | Additional Deps |
|---------|---------|-----------------|
| `core` *(default)* | Basic symbolic agent, reasoning traits | – |
| `toolkit` | Bridge to the [`toka-toolkit-core`](../toka-toolkit-core/README.md) `ToolRegistry` | `toka-toolkit-core` |
| `vault` | Persist long-term state in the canonical event store | `toka-events` |

You can therefore embed the crate **without** the heavy toolkit / vault stacks when you only need in-memory reasoning.

---

## Quick Look

```rust,ignore
use toka_agents::{BaseAgent, Observation};

let mut agent = BaseAgent::new("alice");
agent.observe(Observation {
    key: "sky_is_blue".into(),
    evidence_strength: 2.0,
    supports: true,
}).await?;

// Generate actions based on updated beliefs
let actions = agent.act().await;
println!("{:?}", actions);
```

Behind the scenes every belief update is emitted as an `AgentEvent` and can be persisted by the runtime.

---

## Relationship to the Runtime

`toka-runtime` owns the **lifecyle** – spawning agents, routing events and wiring optional vault & toolkit features.  `toka-agents` only knows about the `EventBus` trait and remains IO-free otherwise.

---

## Security & Stability

* Capability flags (`TOOL_USE`, `VAULT`, `MEMORY`, `REASONING`) declare *exactly* what each agent is allowed to do.
* All code is `#![forbid(unsafe_code)]`.
* The API is **unstable** while we move towards v0.2 – expect breaking changes.

---

## License

Apache-2.0 OR MIT — choose whichever works for you.

© 2025 Toka Contributors 