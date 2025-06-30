# toka

## `toka` – Meta Crate

Batteries-included entry-point that re-exports the **most common types** from
the Toka ecosystem (agents, event store, auth, toolkit …).  Think of it as
the *standard library* for quickly prototyping on the platform.

If you want fine-grained control over dependencies simply depend on the
individual crates (`toka-agents`, `toka-events`, …) and disable default
features.

### Feature Flags
| Flag       | Re-exported Sub-system | Extra crates pulled in |
|------------|------------------------|------------------------|
| **default** | `agents`, `auth`, `events`, `toolkit` | see below |
| `agents`   | `toka-agents`        | `tokio`, `anyhow`, … |
| `auth`     | `toka-capability-jwt-hs256` | `jsonwebtoken` |
| `events`   | `toka-events`        | `sled`, `blake3`, … |
| `toolkit`  | `toka-toolkit-core` + `toka-tools` | `wasmtime` (optional) |

Example with _only_ the auth helpers enabled:
```toml
[dependencies]
toka = { version = "0.1", default-features = false, features = ["auth"] }
```

### Quick Example
```rust
use toka::prelude::*;

// Create a signed capability token (auth feature)
let token = CapabilityToken::new(
    "alice",          // subject
    "vault1",         // vault id
    vec!["read".into()],
    "my-32-byte-secret",
    3600,              // 1 h TTL
).unwrap();
assert!(token.is_valid("my-32-byte-secret"));
```

---
This crate is `#![forbid(unsafe_code)]` and merely re-exports – it contains
*no* runtime logic.

License: MIT OR Apache-2.0
