# Toka

> _A modular runtime and toolkit for building secure, event-driven AI systems_

Toka is an experimental Rust workspace that provides the foundation for building, running, and operating **agents** – long-lived programs that observe the world, reason, and take action with tools.

The project is in early development. Interfaces may change at any time.

---

## Why Toka?

1. **Event Sourcing by Default** – Everything that happens is captured in an **append-only event store** (`toka-events`).
2. **Security First** – Capability-based auth, encrypted storage, and sand-boxed execution are built-in, not bolted-on.
3. **Composable Building Blocks** – Each crate does one job well and can be used à la carte.
4. **Lean Dependencies** – Features that pull in heavy ML or database stacks are **opt-in**.

---

## Workspace at a Glance

| Layer | Crate | Purpose |
|-------|-------|---------|
| _Primitives_ | `toka-primitives` | `no_std` types such as IDs & currency. |
| _Domain_ | `toka-core` | Business rules & pricing logic built on primitives. |
| _Event Store_ | `toka-events` | **Canonical source of truth** – supports in-memory & sled back-ends. |
| _Agents_ | `toka-agents` | Re-usable agent behaviours & lifecycles. |
| _Runtime_ | `toka-runtime` | Async host that wires agents, vault, and toolkit together. |
| _Toolkit_ | `toka-toolkit-core`, `toka-toolkit` | Trait + reference tool implementations. |
| _Security_ | `toka-security-auth` | Capability tokens & auth helpers. |
| _CLI_ | `toka-cli` | Dev-friendly command-line interface. |
| _Meta_ | `toka` | Convenience crate that re-exports a sensible default prelude. |

---

## Getting Started

Add the **meta-crate** for an all-in-one experience:

```toml
[dependencies]
toka = "0.1"
```

Need only the event store? Keep it lean:

```toml
[dependencies]
toka-events = { version = "0.1", features = ["persist-sled"] }
```

---

## Roadmap Highlights

- v0.2 – Stabilise the vault API & runtime lifecycle.
- v0.3 – Distributed vault back-ends (e.g. gRPC, Redis).
- v0.4 – Intent clustering & semantic navigation _(currently on hold – design in progress)._ 

---

## Contributing

Contributions are welcome! Please read `CONTRIBUTING.md` before opening PRs.

---

## License

Dual-licensed under either:

• **Apache-2.0** OR
• **MIT**

---

© 2024 Toka Contributors 