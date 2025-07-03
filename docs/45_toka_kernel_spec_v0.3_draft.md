# Toka-Kernel – v0.3 DRAFT (Namespace Split)

> Status: **Draft** | Version: **0.3.0-alpha** | Last-updated: 2025-07-03  
> This draft captures the *namespace split* proposal removing **all domain
> opcodes** from the core and introducing a minimal, system-level surface.

---

## 1 Motivation

The kernel must remain **domain-agnostic** so that higher-level toolkits
(e.g. `toka-agents`, `toka-finance`) can evolve independently.  By distilling
the contract down to generic system primitives we gain:

1. Stable surface → fewer breaking changes across the ecosystem.
2. Smaller audit scope → easier verification & formal methods.
3. Explicit extension mechanism → clear boundaries for side-band logic.

---

## 2 Operation Enumeration (`toka_types::Operation`)

```rust
/// Core, domain-agnostic primitives.
///
/// *All* families (agents, finance, identity …) are provided by external
/// crates implementing `OpcodeHandler`.
#[non_exhaustive]
enum Operation {
    /* 0x01 */ CreateEntity     { template: Hash256 },
    /* 0x02 */ DeleteEntity     { id: EntityId },
    /* 0x03 */ GrantCapability  { to: PubKey, cap: Capability },
    /* 0x04 */ RevokeCapability { cap_id: Hash256 },
    /* 0x05 */ SubmitBatch      { ops: Vec<Message> },
    /* 0x06 */ EmitEvent        { topic: String, data: Vec<u8> },
    /* 0x07 */ RegisterHandler  { range: Range<u8>, entry: HandlerRef },
}
```

### 2.1 World-State

```rust
struct WorldState {
    entities: HashSet<EntityId>,
    grants:   HashMap<PubKey, Vec<Capability>>,
}
```

### 2.2 Event Enumeration (`toka_events::Event`)

```rust
enum Event {
    EntityCreated      { template: Hash256, id: EntityId },
    EntityDeleted      { id: EntityId },
    CapabilityGranted  { to: PubKey, cap: Capability },
    CapabilityRevoked  { cap_id: Hash256 },
    BatchSubmitted     { count: usize },
    EventEmitted       { topic: String, data: Vec<u8> },
    HandlerRegistered  { range_start: u8, range_end: u8, entry: HandlerRef },
}
```

---

## 3 Execution Flow (unchanged)

1. **Dispatch** external handlers → built-in core handlers.
2. **Authorise** capability token.
3. **Execute** handler ⇒ mutate `WorldState`.
4. **Emit** `KernelEvent` onto bus.

---

## 4 Extension Crates

A handler that wants to claim the *agent* opcode range registers itself:

```rust
register_handler("agent", Box::new(AgentOpcodeHandler::default()));
```

The claimed byte range is negotiated out-of-band (TBD registry service).

---

© 2025 Toka Project — Apache-2.0 / MIT