# Toka Capability (`toka-capability`)

> **Status**: 0.2.0-alpha – experimental

Lightweight **capability-token** primitives shared by the runtime, agents and
internal tooling.  This crate supersedes the historical
`toka-security-auth` crate and brings a cleaner modularisation ahead of the
post-quantum roadmap.

---

## Why Capability Tokens?

Toka's security model is **capability-based**: possession of an
*unforgeable token* grants access to a specific vault or resource instead of
relying on ambient identity.  The token format is a compact [JWT](https://datatracker.ietf.org/doc/html/rfc7519)
containing the *minimal* set of claims required for authorisation.

| Claim | Purpose |
|-------|---------|
| `sub` | Logical principal (user, agent, service) |
| `vault` | Workspace / vault identifier |
| `permissions` | Ordered list of allowed actions |
| `iat` | Issued-at (seconds) |
| `exp` | Expiry – **≤ now + 30 min** |
| `jti` | UUIDv4 for audit correlation |

---

## Features

* `CapabilityToken` wrapper for minting signed JWTs.
* Constant-time validation with zero allocations in the happy path.
* Strict expiry enforcement – tokens are invalid *the moment* `exp` is hit.
* Trait-based validator so you can plug in Biscuit or Paseto later without
  refactors.
* `#![forbid(unsafe_code)]` – memory safety first.

---

## Example

```rust
use toka_capability::token::CapabilityToken;

let secret = "my-32-byte-server-secret"; // store securely!
let token = CapabilityToken::new(
    "user_…",
    "vlt_…",
    vec!["TOOL_USE".into(), "VAULT".into()],
    secret,
    3600, // 1h TTL
).unwrap();
assert!(token.is_valid(secret));
```

Embed the serialised token into HTTP headers or gRPC metadata and verify it
inside any service with the complementary `JwtValidator`.

---

## Relationship to Sibling Crates

* **`toka-revocation`** – pluggable revocation stores & RFC 7009 endpoints.
* **`toka-cvm`** (optional) – high-level *Capability Validation Module* for
  WASM guest modules.

Each crate is laser-focused so you only pull what you truly need.

---

## License

Apache-2.0 OR MIT

© 2025 Toka Contributors