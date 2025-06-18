# Toka Meta Crate

`cargo add toka` gives you a batteries-included entry point to the Toka platform.

```rust
use toka::prelude::*;

let token = CapabilityToken::new("alice", "vault1", vec!["read".into()], "secret", 3600);
```

Disable default features if you only need a subset:

```toml
[dependencies]
toka = { version = "0.1", default-features = false, features = ["auth"] }
``` 