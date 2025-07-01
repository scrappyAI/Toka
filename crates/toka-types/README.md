# toka-types

Shared primitive data structures for the **Toka OS** workspace.

This crate defines:

* `EntityId` – secure, 128-bit identifier for any on-chain/off-chain entity (user, agent, system).
* `Operation` – kernel opcode enumeration (financial, agent, and user primitives).
* `Message` – authenticated envelope `{ origin, capability, op }` handled by the kernel.

The crate is intentionally dependency-light and re-exports only foundational traits (e.g. `serde::{Serialize, Deserialize}`).

> Changes here should remain backward-compatible. Treat additions as *additive*; avoid breaking enum variants outside a major version bump.