# Capability Tokens – v0.2 Specification

> Status: **Stable** | Version: 0.2 | Last-updated: 2025-07-04

---

## 1  Purpose

This document standardises how *capability tokens* are minted, transported and validated across the **Toka** platform. It supersedes the informal design notes in `prompts/cap_based_security_starter.md` and becomes the single source of truth for implementers.

*Capability‐based security* grants access through possession of an **unforgeable token** that spells out *exactly* which action(s) may be taken against which resource(s). The model is identity-agnostic, supports offline delegation and enforces least authority by construction.

The specification deliberately separates two token classes:

| Class | Audience | Format | Transport | Revocation | Typical TTL |
|-------|----------|--------|-----------|------------|-------------|
| **External** | Browsers, CLIs & 3ᵖ services | **JWT** (access + ID tokens) | HTTPS request headers | RFC 7009 revocation endpoint | ≤ 15 min |
| **Internal** | Micro-services & WASM modules | **Capability Token** (this spec) | gRPC / NATS metadata | Delete row from DB (if opaque) or TTL | 5 – 30 min |

The remainder of this document focuses on the *internal* flavour.

---

## 2  Token Format

Internally we encode capabilities as JSON Web Tokens (RFC 7519) by default via the *HS256* implementation crate [`toka-capability-jwt-hs256`]. Alternative algorithms can be plugged in without affecting this spec.

### 2.1  Header

```
{
  "alg": "HS256",           // MUST match allow-list; see §3.3
  "typ": "toka.cap+jwt"     // Disambiguates from OAuth access tokens
}
```

### 2.2  Payload (Claims)

| Claim | Type | Required | Description |
|-------|------|----------|-------------|
| `sub` | `string` | ✓ | Logical principal (user, agent or service id).
| `vault` | `string` | ✓ | Vault / workspace id the principal wishes to access.
| `permissions` | `string[]` | ✓ | Ordered list of allowed actions (e.g. `read`, `write`).
| `iat` | `number` | ✓ | Seconds since Unix epoch when token was issued.
| `exp` | `number` | ✓ | Absolute expiry. MUST be ≤ **now + 30 min**.
| `jti` | `string` | – | UUIDv4 for audit correlation.

### 2.3  Signature

The initial reference implementation signs with **HMAC-SHA-256 (HS256)**.
A future *MINOR* release will introduce **EdDSA (ed25519)** while retaining HS256 for backwards compatibility.

---

## 3  Validation Rules

1. **Clock Skew** – Accept at most ±5 seconds. Tokens are considered invalid the *moment* the wall-clock reaches `exp`.
2. **Algorithm Allow-List** – Only `HS256` is accepted in v0.2. Reject `alg:"none"` and anything else.
3. **Issuer Separation** – Internal tokens carry `typ:"toka.cap+jwt"` and must be signed with the *internal* key, never the public OAuth key.
4. **Audience** – Not required. Internal services derive authorisation purely from `vault` + `permissions`.
5. **Constant-Time Comparison** – Always compare MACs in constant time.

---

## 4  Lifecycle

1. **Mint** – Auth service issues token after authenticating principal. Caller receives the raw JWT.
2. **Transport** – Sent as `Authorization: Bearer <jwt>` header or `grpc-metadata-bearer`.
3. **Validate** – Receiving service executes algorithm in §3.
4. **Propagate / Delegate** – A holder MAY forward the token *unchanged* to downstream services. Attenuation (sub-scoping) will arrive with Biscuit support in v0.3.
5. **Revoke** – Not supported for stateless JWT. Instead, issue short TTL and rotate often. Opaque-token mode (v0.3) will allow on-demand revocation.

---

## 5  Key Management

| Key | Location | Rotation | Notes |
|-----|----------|----------|-------|
| `INT_CAP_HS256` | HashiCorp Vault | 90-day | Shared symmetric secret for HS256. *Never* leaves the cluster.
| `INT_CAP_ED25519` | HSM / PKCS#11 | 180-day | Future – public half baked into container images.

All keys are identified by a short **kid** string exposed via internal JWKS at `/_internal/.well-known/jwks.json`.

---

## 6  Reference Implementation (Rust)

```rust
use toka_security_auth::{CapabilityToken, JwtValidator};

const SECRET: &str = include_str!("/run/secrets/int_cap_hs256");

// Mint – usually lives behind an HTTP handler
let token = CapabilityToken::new(
    "alice",          // sub
    "vault_123",      // vault
    vec!["read".into()],
    SECRET,
    900,               // 15-minute TTL
)?;

// Validate – inside any internal service
let validator = JwtValidator::new(SECRET);
let claims = validator.validate(token.as_str()).await?;
assert_eq!(claims.vault, "vault_123");
```

The reference Rust implementation lives in `crates/toka-capability-jwt-hs256` with the shared traits residing in `crates/toka-capability-core`.

---

## 7  Compatibility & Migration

The transition from v0.1 to v0.2 requires updating dependencies to use `toka-capability-core` and `toka-capability-jwt-hs256` directly.

New features slated for v0.3:
* EdDSA signing & verification.
* Optional Biscuit token format with offline delegation.
* Opaque-token mode backed by Postgres for instant revocation.

---

## 8  Security Considerations

* The internal secret **must** be 256-bit or stronger and stored in a secrets manager.
* Services **must not** log raw tokens.
* All communication channels carrying tokens are encrypted (TLS 1.3 or HTTP/2 + gRPC).
* Unit tests employ timing-attack regression checks (see `tests/security_tests.rs`).

---

## 9  Decision Log

| Date | Change | Author |
|------|--------|--------|
| 2025-07-04 | v0.2 specification with proper date enforcement | DS |