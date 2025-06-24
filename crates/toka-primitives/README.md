# Toka Primitives

Zero-dependency building blocks (IDs & Currency) shared by all other Toka crates.

---

## Why a Separate Crate?

* **`no_std` friendly** – Can be used on embedded targets or WASM without dragging in the full platform.
* **Single Source of Truth** for fundamental types referenced by the canonical event store (`toka-vault`) and higher domain logic (`toka-core`).
* **Stable release cadence** – `primitives` can evolve independently of fast-moving application crates.

---

## Feature Flags

Both features are enabled by default but can be turned off to shave bytes:

| Feature | Provides |
|---------|----------|
| `ids` | Type-safe identifiers like `UserID`, `AgentID`, `VaultID`, … |
| `currency` | `MicroUSD` fixed-precision money type |

Example `Cargo.toml` when you only need IDs:

```toml
[dependencies]
toka-primitives = { version = "0.1", default-features = false, features = ["ids"] }
```

---

## Quick Demo

```rust
use toka_primitives::{AgentID, MicroUSD};
use rust_decimal_macros::dec;

let id = AgentID::new();
let price = MicroUSD::from_usd_decimal(dec!(0.99)).unwrap();
println!("Agent {id} paid {price}");
```

---

## Relationship to the Vault

Events persisted in `toka-vault` reference these types so that every crate speaks the **same language**.  Keeping primitives decoupled prevents accidental cyclic dependencies.

---

## License

Apache-2.0 OR MIT

© 2024 Toka Contributors 