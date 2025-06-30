# Security Crates

This directory groups **capability-based security** crates that together
implement the authentication & authorisation model defined in
[../../docs/40_capability_tokens_spec_v0.1.md].  The split follows the
three-tier architecture described in [../../docs/41_capability_tokens_architecture.md].

| Crate | Tier | Purpose |
|-------|------|---------|
| `toka-capability-core` | Core | `no_std` Claims struct & traits (`CapabilityToken`, `TokenValidator`). |
| `toka-capability-jwt-hs256` | Impl | HS256 JWT encoder / validator (default). |
| `toka-revocation` | Adapter | RFC 7009 revocation primitives (in-memory store + trait). |
| `toka-cvm` | Adapter | Placeholder *Capability Validation Module* for verifying tokens inside WASM guests.

Crates are intentionally **decoupled** so you only depend on what you really
need (e.g. embedded targets can pull in `core` only).

All crates forbid `unsafe_code` and are covered by MIT OR Apache-2.0.

**Note**: The deprecated `toka-capability` shim crate has been removed in v0.2. Depend directly on `toka-capability-core` and the desired implementation crate (e.g. `toka-capability-jwt-hs256`). 