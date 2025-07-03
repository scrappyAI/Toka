# toka-kernel

Deterministic, capability-secured state-machine core of **Toka OS**.

The kernel is responsible for:

1. **Dispatch** – mapping incoming `toka_types::Operation` variants to handler functions.
2. **Validation** – enforcing capability checks (delegated to `toka_auth`) and business rules.
3. **Execution** – mutating in-memory `WorldState` tables in a deterministic fashion.
4. **Event Emission** – publishing typed events to the `toka_events` bus.

The **v0.2** kernel runs fully in-memory and single-threaded to maximise reproducibility.  Durable storage adapters, gas metering, and async schedulers are planned for **v0.3+**.

> **Security Note**: Every kernel invocation must pass a capability token.  Builds compiled with `--release` will `deny_missing_capability` at runtime to avoid insecure integration environments.