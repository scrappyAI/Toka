# toka-auth

Capability validation layer for **Toka OS**.

This crate exposes a minimal, extensible trait for verifying whether a submitted `Message` is authorised to execute a given `Operation` against a target `EntityId`.  The v0.1 implementation ships a simple HS256-signed macaroon-style token validator; more algorithms can be plugged in via crate features.

```rust
pub trait CapabilityValidator {
    fn allows(&self, op: &toka_types::Operation, entity: &toka_types::EntityId) -> bool;
}
```

Downstream crates (`toka-kernel`, agent runtimes) depend only on this trait, keeping auth concerns decoupled.