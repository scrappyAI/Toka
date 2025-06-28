# Tool Development Guidelines

This document summarises the canonical rules for authoring **Toka tools** that
can be safely invoked by autonomous agents.

> NOTE: This is a living spec.  See the project-wide `agent-tool-building`
> policy for the normative version.

## 1  Declare Capabilities

Each tool MUST declare:

* **Capability** – a short verb, e.g. `read_file`, `post_message`.
* **Input schema** – concrete struct or serde schema.
* **Output schema** – idem.

## 2  Side-effect Clarity

Tools MUST document whether they are:

* Read-only / idempotent
* Causing side effects (file writes, network calls, …)
* Privileged (requires elevated authz)

## 3  Safety Guards

* Validate all inputs.
* Enforce explicit limits (size, count, depth).
* Fail fast with rich error context.

## 4  Execution Isolation

Sub-process, network or FS-mutating tools must:

* Run in sandboxed task (Tokio, container, etc.).
* Emit audit events to the `EventBus`.

## 5  Registration & Versioning

* Register via `ToolRegistry` manifest.
* Version every tool; keep changelog.
* Deprecate old versions gracefully. 