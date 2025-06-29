# Toka Security-Auth

Lightweight **capability-token** primitives shared by the runtime, agents and tools.

---

## Why Capability Tokens?

The platform's security model is _capability-based_: every actor receives an unforgeable token that spells out **exactly** which vault, tools or resources it may access.  This crate implements those tokens using compact [JSON Web Tokens (JWT)](https://datatracker.ietf.org/doc/html/rfc7519).  By default we sign with symmetric **HS256** for maximum portability, while the public-key **EdDSA** variant will land in a future release (tracked in the roadmap).

---

## Features

* `CapabilityToken` struct wrapping an RFC 7519 JWT.
* Built-in TTL — tokens expire automatically via the `exp` claim.
* Fast runtime validation with zero allocations in the happy-path.
* Constant-time signature checks to mitigate timing attacks.
* `#![forbid(unsafe_code)]` – security starts with memory safety.

---

## Example

```rust
use toka_security_auth::token::CapabilityToken;

let secret = "my-32-byte-server-secret"; // store securely!
let token = CapabilityToken::new(
    "user_…",
    "vlt_…",
    vec!["TOOL_USE".into(), "VAULT".into()],
    secret,
    3600, // 1h TTL
);
assert!(token.is_valid(secret));
```

Embed the serialised token into HTTP headers or gRPC metadata and verify it inside the runtime.

---

## Relationship to the Event Store

Authorisation is **orthogonal** to the event system, but tokens often appear in event payloads (e.g. `agent.auth_used`).  Keeping the type in its own crate avoids pulling hashing deps into `toka-events`.

---

## Status

Stable API – no breaking changes expected before v1.0 unless a security issue is uncovered.

---

## License

Apache-2.0 OR MIT

© 2024 Toka Contributors 