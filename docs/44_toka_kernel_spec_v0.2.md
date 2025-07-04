# Toka-Kernel – v0.2.1 Specification (Official)

> Status: **Stable** | Version: **0.2.1** | Last-updated: 2025-07-04  
> This version represents the completed architectural refactor that separated deterministic kernel operations from fuzzy user-space concerns.
>
> v0.2 removes _finance_ & _user_ opcode families from the core and defines
> a **minimal, agent-centric kernel surface**.  Domain-specific functionality
> is expected to live in _extension crates_ that plug into the kernel via the
> new `OpcodeHandler` registry introduced in this release.

---

## 1 Purpose

`toka-kernel` is the deterministic nucleus of the Toka Agentic Operating
System.  Every state transition passes through it so that:

1. Mutations are **authorised** by capability tokens.
2. The resulting event log is **replayable** and auditable.
3. Sub-systems (storage, networking, tooling …) can evolve independently while
   preserving a stable, minimal contract.

v0.2 narrows the contract to **agent primitives only** and introduces an
extension mechanism for optional opcode families.

---

## 2 Glossary (Updates)

| Term | Meaning |
|------|---------|
| **Agent Process (AP)** | Logical execution context identified by an `EntityId`. |
| **Task** | Deterministic function scheduled inside an AP (akin to a thread). |
| **Opcode Handler** | Plug-in implementing extra opcodes via the registry. |
| **Kernel Interface (KI)** | Stable API exposed by the kernel (`Message`, `Operation`). |

---

## 3 Kernel Surface (v0.2)

### 3.1 Operation Enumeration

```rust
enum Operation {
    // Process / Tasking (always compiled)
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    EmitObservation   { agent: EntityId, data: Vec<u8> },
}
```

*All domain-specific families (finance, identity, etc.) are provided by
external crates implementing `OpcodeHandler`.*

### 3.2 World-State

```rust
struct WorldState {
    agent_tasks: HashMap<EntityId, Vec<TaskSpec>>,
}
```

### 3.3 Extension Architecture

The v0.2.1 refactor establishes clear architectural boundaries:

```
DETERMINISTIC CORE (replayable)
├── toka-types        # Pure data structures  
├── toka-auth         # Capability validation
├── toka-bus-core     # Event broadcasting
└── toka-kernel       # State machine core

STORAGE LAYER (pluggable)
├── toka-store-core   # Abstract traits
├── toka-store-memory # In-memory driver  
└── toka-store-sled   # Persistent driver

FUZZY/ASYNC LAYER (user-space)
└── toka-runtime      # Configuration & coordination
```

Domain-specific functionality (finance, user management, etc.) is expected to be implemented at the runtime layer or as separate application services.

### 3.4 Error Model (excerpt)

```rust
pub enum KernelError {
    CapabilityDenied,
    UnknownEntity(EntityId),
    InvalidOperation(String),
    UnsupportedOperation, // unchanged
}
```

---

## 4 Execution Flow (unchanged)

1. **Dispatch** (external handlers, then built-in).
2. **Authorise** capability token.
3. **Execute** handler ⇒ mutate `WorldState`.
4. **Emit** `KernelEvent` onto bus.

---

## 5 Roadmap

| Version | Theme | ETA |
|---------|-------|-----|
| **0.2.1** | Scheduler trait (async) | 2025-07-15 |
| **0.3.0** | Storage adapters & driver API | 2025-07-30 |

---

## 6 Backwards Compatibility

* Breaking change: snapshots created with ≤v0.1 must be migrated or replayed
  through an adapter.
* Extension crates may continue to support legacy opcodes behind their own
  semver contract.

---

© 2025 Toka Project — Apache-2.0 