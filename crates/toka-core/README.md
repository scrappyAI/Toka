# Toka Core

Core business logic and domain rules for the Toka platform, built on top of the `toka-primitives` crate.

---

## What's Inside?

| Category | Module | Feature Flag |
|----------|--------|--------------|
| Fundamental Types | IDs, Currency | `ids`, `currency` |
| Domain Models | LLM model metadata, Resources, Vault descriptors | `models`, `resources`, `vaults` |
| Economics | Fee schedules, payout rules, platform-wide take-rate policies | `economics` |
| Products | Credit packages & other SKUs | `products` |
| Pricing | Policy abstraction + default implementation | `pricing` |

Nothing is enabled by default – downstream crates opt-in to just what they need to keep binaries lean.

---

## Relation to the Event Store

All state-changing actions in Toka are captured in the **canonical event store** provided by the [`toka-vault`](../../toka-vault/README.md) crate.  `toka-core` defines *what* is stored (domain rules & invariants) but **does not talk to the vault directly** – that is the runtime's job.  This separation keeps the domain layer free of IO and heavy dependencies.

For details on the vault consolidation and why legacy bus / ledger crates were retired see [`EVENT_SYSTEM_REFACTOR.md`](../../EVENT_SYSTEM_REFACTOR.md).

---

## Quick Start

```toml
[dependencies]
# Enable only the pieces you need – here IDs & Currency
toka-core = { version = "0.1", features = ["ids", "currency"] }
```

```rust
use toka_core::currency::MicroUSD;
use toka_core::ids::UserID;

let price = MicroUSD(1_500_000); // $1.50
let user = UserID::new();
println!("User {user} has to pay {price}");
```

---

## Philosophy

* **Event-sourced by default** – Decisions are evaluated using past events, never hidden mutable state.
* **Lean dependencies** – Pure Rust `no_std` compatible where possible.
* **Composable** – Each feature gate is orthogonal, making it trivial to build tiny binaries for edge devices.

---

## License

Dual-licensed under either:

* Apache-2.0 OR
* MIT

© 2024 Toka Contributors 