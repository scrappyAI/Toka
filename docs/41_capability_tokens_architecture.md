# Capability Tokens – Three-Tier Architecture (v0.2)

> This document complements `40_capability_tokens_spec_v0.2.md` and explains **how** the Rust crates map onto the spec.

---

## 1  Overview

```
          ┌────────────┐
          │  Services  │  ← Agents, Runtime, Auth-svc, CVM …
          └────┬───────┘
               │ uses
               ▼
        ┌──────────────────┐
        │  Implementation  │  (`toka-capability-jwt-hs256`, `jwt-eddsa`, …)
        └────┬─────────────┘
             │ implements
             ▼
     ┌──────────────────────┐
     │       Core           │  (`toka-capability-core`)
     └──────────────────────┘
```

* **Core** – pure, `no_std`, owns `Claims` & the two fundamental traits (`CapabilityToken`, `TokenValidator`).
* **Implementation** – brings crypto & std, produces concrete structs (e.g. `JwtHs256Token`).
* **Adapters / Services** – I/O, storage, gRPC, Wasmtime, etc. Never touch crypto.

---

## 2  Crate Responsibilities

| Crate | Tier | `#![no_std]`? | Responsibility |
|-------|------|-------------|----------------|
| `toka-capability-core` | Core | ✅ (`alloc`) | Canonical data-model & traits. |
| `toka-capability-jwt-hs256` | Impl | 🚫 (needs `SystemTime`, `jsonwebtoken`) | HS256 JWT encoder / validator. |
| `toka-revocation` | Adapter | 🚫 | RFC 7009 stores (memory / redis / pg). |
| `toka-cvm` | Adapter | 🚫 | Host helper for validating tokens inside WASM guests. |

---

## 3  Feature Flags

```
[toka-capability-core]
default = ["std"]   # disable for embedded targets

[toka-capability-jwt-hs256]
default = ["std"]   # alloc-only mode coming in v0.3
```

---

## 4  Migration Cheatsheet

1. **Library / embedded code** → depend on `toka-capability-core` *only* and accept a generic `impl TokenValidator`.
2. **Micro-services** → add `toka-capability-jwt-hs256` (or another impl) and construct the concrete validator.
3. **Old `toka-security-auth` users** → update `Cargo.toml`:

```toml
# Before
[dependencies]
toka-security-auth = "0.1"

# After (temporary shim still works but shows warning)
[dependencies]
toka-capability = "0.2"           # deprecated shim
# or migrate directly
toka-capability-jwt-hs256 = "0.2"
```

---

## 5  Roadmap

* v0.3 – `toka-capability-jwt-eddsa` (EdDSA/Ed25519) + alloc-only HS256 build.
* v0.4 – Biscuit support with attenuation and offline delegation.
* v1.0 – Stabilise public API and freeze wire-format for long-term compatibility.