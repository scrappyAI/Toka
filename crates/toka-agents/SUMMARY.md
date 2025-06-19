# toka-agents – Summary
 
* `symbolic.rs` – SymbolicAgent implementation & related types
* `prelude.rs` – one-stop import of common agent items 
* `metadata.rs` – AgentMetadata struct and Capability bitflags [NEW]
* `reasoning.rs` – ReasoningEngine trait and base implementations [NEW]

## Current Progress (Phase 1-2 Complete)

### ✅ Phase 1: Metadata & Capabilities
- Added `AgentMetadata` struct with id, name, description, version, capabilities
- Added `Capability` bitflags for TOOL_USE, VAULT, MEMORY, REASONING
- Added required dependencies: `semver` (with serde), `bitflags` (with serde)

### ✅ Phase 2: ReasoningEngine Foundation
- Created `ReasoningEngine` trait with single `reason()` method
- Added `ReasonOutcome` enum (BeliefUpdates, Plans, ToolCalls, NoOp)
- Added `AgentContext` for dependency injection (currently holds EventBus)
- Implemented `NoOpReasoner` as default reasoning engine

### 🚧 Next: Phase 3 - Extract SymbolicReasoner
- Move Bayesian logic from `SymbolicAgent` into `reasoning/symbolic.rs`
- Update `SymbolicAgent` to hold `Box<dyn ReasoningEngine>`
- Maintain backward compatibility with existing tests

### 📋 Remaining Phases
- Phase 4: LLMReasoner (feature "reason-llm")
- Phase 5: CodeSandboxReasoner (feature "reason-sandbox") 
- Phase 6: CompositeReasoner & policies
- Phase 7: Runtime integration & budgets
- Phase 8: Observability & metrics
- Phase 9: Documentation & examples 