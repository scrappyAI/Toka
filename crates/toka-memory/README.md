# toka-memory

## Toka Memory – lightweight, in-process key–value cache

Slice 3 of the SIMPLECT refactor introduces a small, **async** cache
abstraction living in its own crate so other subsystems (agents, runtime,
projections) can share a pluggable memory model without pulling heavy
databases.

The API is intentionally minimal: single get/put with a TTL.  Callers that
need more complex semantics (atomic counters, CAS etc.) should wrap their
own newtype around [`MemoryAdapter`].

License: MIT OR Apache-2.0
