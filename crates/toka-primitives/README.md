# Toka Primitives

This crate provides fundamental, building-block types used throughout the Toka ecosystem. These primitives are designed to be lightweight and free of any high-level business logic, making them easy to compose in other parts of the platform.

## Features

This crate is feature-gated to allow you to only include the primitives you need.

- `ids`: Provides strongly-typed identifiers for various entities in the Toka ecosystem. This helps prevent bugs caused by mixing up different types of IDs.
- `currency`: Provides a `MicroUSD` type for handling monetary values with high precision.

By default, both the `ids` and `currency` features are enabled.

## Usage

Add `toka-primitives` to your `Cargo.toml`:

```toml
[dependencies]
toka-primitives = "0.1.0"
```

If you only need specific primitives, you can disable the default features and select the ones you need:

```toml
[dependencies]
toka-primitives = { version = "0.1.0", default-features = false, features = ["ids"] }
```

### Example: Using `AgentID`

```rust
use toka_primitives::AgentID;

let agent_id = AgentID::new();
println!("New Agent ID: {}", agent_id);
```

### Example: Using `MicroUSD`

```rust
use toka_primitives::MicroUSD;
use rust_decimal_macros::dec;

let cost = MicroUSD(dec!(0.0015)); // $0.0015
println!("Cost: {}", cost);
``` 