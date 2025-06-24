# Toka v0.3 – "SIMPLECT" Refactor Plan

_This document is the authoritative blueprint for the upcoming architectural clean-up.  Each numbered slice must land independently with tests and docs before we move on to the next._

---

## Golden Six Principles

1. **Single-Responsibility Crates** – bus, vault, auth, agents, toolkit stay orthogonal.  
2. **Capability-First Security** – every privileged action is expressed in a token.  
3. **Explicit Persistence** – disk access crosses a dedicated trait (`EventSink`).  
4. **Zero Secret Leakage** – secrets live in `secrecy` wrappers and are zeroised on drop.  
5. **LLM-Readable APIs** – short names, few generics, one trait per file.  
6. **Docs Next to Code** – module rustdoc + ≤120-line README for every crate.

---

## 10 Slices (merge order)

| # | Title | Kind | Goal |
|---|-------|------|------|
| 0 | **Bootstrap – deps & ADR-001** | 📚 | Adopt `pasetors`, `jsonwebtoken`, `secrecy`. Write Security Tokens ADR. |
| 1 | **Extract `toka-bus`** | ⚙📚 | New crate with `EventBus` trait + `MemoryBus`. All crates depend on bus, not vault. |
| 2 | **Vault = Persistence Only** | ⚙📚 | Remove bus from vault; add `EventSink` & `QueryApi`. Runtime drains bus → vault. |
| 3 | **Memory Model** | ⚙ | New `toka-memory` crate (in-proc agent memory + TTL). |
| 4 | **Auth Refactor** | ⚙🔐 | Replace custom token with Paseto/JWT. Add `TokenValidator`. |
| 5 | **Runtime Security Envelope** | ⚙🔐 | Secret rotation, validator hook, redaction layer. |
| 6 | **Proc-macro QoL (`toka-derive`)** | ⚙📚 | `#[derive(Event)]`, `#[budget]`. |
| 7 | **Projections API** | ⚙ | `Projector` trait in bus; add sample BalanceProjector. |
| 8 | **Agents 2.0** | ⚙📚 | Agent shell + ctx, budgeting accountant, typetag engines. |
| 9 | **Clean-up & Docs Sprint** | 📚 | Update READMEs, generate cargo-doc, write ADR-002 & ADR-003.

Each slice ≤ 500 LOC diff and keeps CI green.

---

## Key Interfaces (sketch)

```rust
// toka-bus
#[async_trait]
pub trait EventBus {
    async fn publish<P: EventPayload>(&self, payload: &P, kind: &str);
    fn subscribe(&self) -> broadcast::Receiver<EventHeader>;
}

// toka-vault
#[async_trait]
pub trait EventSink {
    async fn commit(&self, header: &EventHeader, payload: &[u8]);
}

pub trait QueryApi {
    async fn header(&self, id: &EventId) -> Option<EventHeader>;
    async fn payload<P: EventPayload>(&self, digest: &CausalDigest) -> Option<P>;
}

// toka-memory
pub trait MemoryAdapter {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn put(&self, key: &str, val: Vec<u8>, ttl_secs: u64);
}
```

---

## Migration Checklist

- [ ] Update workspace `Cargo.toml` with new crates & features.  
- [ ] Deprecate Blake3 token path after Paseto/JWT prove stable.  
- [ ] Remove any direct vault writes from agents/tests.  
- [ ] Add fuzz targets for vault read/write & token parsing.

---

## Progress

- [x] Slice 1 – **toka-bus crate scaffolded** (`EventBus`, `MemoryBus`, workspace member)
- [x] Slice 1 – Migrate existing crates to depend on `toka-bus`
- [x] Slice 1 – Removed legacy `toka_secrets` dependency; introduced lightweight `MemoryAdapter` in `toka-agents`
- [ ] Slice 2 – Vault persistence-only refactor (pending)

_Last updated: 2025-06-24_ 