# Toka CVM (`toka-cvm`)

> **Status**: Placeholder – not yet implemented

`cvm` stands for **Capability Validation Module**.  The goal is to offer a
host-side helper that verifies capability tokens *inside* guest WASM modules
(exec'd via Wasmtime).  This allows untrusted user code to safely interact
with Toka services while the host guarantees strict least authority.

The crate currently ships *no code*; it merely reserves the namespace and
lays down the groundwork for future post-quantum and agent-native security
work.

---

## Planned Features

* `validate(token: &str)` – minimal FFI boundary for guests.
* **EdDSA** and **Biscuit** support once upstream.
* Tight integration with `toka-capability-core` and `toka-revocation`.

---

## License

Apache-2.0 OR MIT