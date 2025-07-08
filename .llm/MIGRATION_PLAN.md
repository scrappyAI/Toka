# Clean Architecture Migration Plan

This document outlines the step-by-step migration to resolve schema sprawl and create a clean integration between `.llm` and `toka-tools`.

## 🎯 **Migration Goals**

1. **Eliminate schema sprawl** - Clear separation between general schemas and specific instances
2. **Single source of truth** - Production tools live in `toka-tools`, schemas provide structure
3. **Strict validation** - Prevent invalid LLM output through rigorous validation
4. **Clean integration** - Seamless bridge from validated schemas to Rust implementations
5. **Scoped .llm directory** - Minimal, well-defined context for LLMs

## 📋 **Current State Analysis**

### **Problems Identified**
- ✅ Schema sprawl between general schemas and specific instances
- ✅ Duplicate tool systems (`.llm/tools` vs `toka-tools`)
- ✅ No strict validation for LLM-generated schemas
- ✅ Unclear integration path between systems
- ✅ Complex, confusing directory structure

### **Assets to Preserve**
- ✅ Date enforcement system and contracts
- ✅ Policy system with priority ordering
- ✅ Schema validation concepts
- ✅ Integration with `toka-tools` architecture

## 🏗️ **Target Architecture**

```
.llm/
├── schemas/                    # GENERAL schemas (structure definitions)
│   ├── agent.json             # ✅ DONE - What ANY agent must look like
│   ├── tool.json              # ✅ DONE - What ANY tool must look like
│   ├── policy.json            # 🔄 TODO - General policy structure
│   └── contract.json          # 🔄 TODO - General contract structure
├── instances/                 # SPECIFIC instances (concrete implementations)
│   ├── agents/               # 🔄 TODO - Move specific agents here
│   ├── tools/                # 🔄 TODO - Tool instance specifications
│   ├── policies/             # 🔄 TODO - Specific policy instances
│   ├── contracts/            # ✅ DONE - Contract instances
│   └── examples/             # ✅ DONE - Reference examples for LLMs
├── validation/               # Schema validation system
│   ├── validate.py          # ✅ DONE - Strict schema validator
│   ├── rules.json           # ✅ DONE - Validation rules
│   └── generate.py          # 🔄 TODO - Schema-compliant generator
└── integration/             # Integration with toka-tools
    ├── sync.py              # ✅ DONE - Sync validated tools to toka-tools
    ├── bridge.json          # 🔄 TODO - Bridge configuration
    └── mapping.json         # 🔄 TODO - Schema to Rust mapping
```

## 📅 **Migration Timeline**

### **Phase 1: Schema Cleanup** (Week 1)

#### Day 1-2: Complete General Schemas
- [ ] Create `schemas/policy.json` - General policy structure
- [ ] Create `schemas/contract.json` - General contract structure  
- [ ] Validate all general schemas are complete and consistent

#### Day 3-4: Migrate Specific Instances
- [ ] Move specific agents from current locations to `instances/agents/`
- [ ] Move specific policies to `instances/policies/`
- [ ] Ensure all instances validate against general schemas

#### Day 5-7: Clean Up Legacy
- [ ] Remove old sprawled schema files (`agent-spec-schema.json`, `cursor-rule-schema.json`)
- [ ] Update schema index to point to new structure
- [ ] Test validation system with new structure

### **Phase 2: Validation System** (Week 2)

#### Day 1-3: Enhance Validation
- [ ] Complete `validation/validate.py` with all schema types
- [ ] Add comprehensive business rule validation
- [ ] Create validation rules for all resource types
- [ ] Test strict validation with reject scenarios

#### Day 4-5: LLM Generation Guardrails
- [ ] Implement `validation/generate.py` for LLM guidance
- [ ] Add schema-compliant generation helpers
- [ ] Create validation hooks for real-time feedback

#### Day 6-7: Integration Testing
- [ ] Test entire validation pipeline
- [ ] Validate example instances
- [ ] Document validation requirements for LLMs

### **Phase 3: Integration Bridge** (Week 3)

#### Day 1-3: Complete Bridge System
- [ ] Finish `integration/sync.py` implementation
- [ ] Create `integration/bridge.json` configuration
- [ ] Build `integration/mapping.json` for schema-to-Rust mapping

#### Day 4-5: Rust Code Generation
- [ ] Test Rust template generation from schemas
- [ ] Validate generated code compiles in `toka-tools`
- [ ] Create YAML manifest generation for runtime

#### Day 6-7: End-to-End Testing
- [ ] Test complete flow: Schema → Validation → Rust Generation → Compilation
- [ ] Validate integration with `toka-tools` runtime
- [ ] Performance testing and optimization

### **Phase 4: Final Cleanup** (Week 4)

#### Day 1-2: Remove Duplicate Systems
- [ ] Clean up duplicate Python tools in `.llm/tools/`
- [ ] Remove schema wrapper systems
- [ ] Consolidate tool definitions

#### Day 3-4: Documentation and Examples
- [ ] Update all documentation to reflect new architecture
- [ ] Create comprehensive examples for LLMs
- [ ] Add integration guides

#### Day 5-7: Validation and Deployment
- [ ] Complete system validation
- [ ] Performance testing
- [ ] Deploy to production environment

## 🔧 **Implementation Steps**

### **Step 1: Complete General Schemas**

```bash
# Create remaining general schemas
cd .llm/schemas
# TODO: Create policy.json and contract.json based on existing patterns
```

### **Step 2: Migrate Instances**

```bash
# Create instances directory structure
mkdir -p .llm/instances/{agents,tools,policies,contracts}

# Move existing instances (need to identify and migrate)
# Example:
# mv .llm/agents/specific-agent.json .llm/instances/agents/
```

### **Step 3: Implement Validation**

```bash
# Test validation system
cd .llm
python validation/validate.py instances/ --directory --report validation_report.json

# Should show all instances validate against general schemas
```

### **Step 4: Test Integration**

```bash
# Test bridge system
cd .llm
python integration/sync.py instances/examples/file-processor-tool.json

# Should generate Rust code in toka-tools/src/generated/
```

## 🧪 **Testing Strategy**

### **Schema Validation Testing**
1. **Valid Instances** - All examples should pass validation
2. **Invalid Instances** - Create test cases that should fail validation
3. **Edge Cases** - Test boundary conditions and limits
4. **LLM Generation** - Test that LLM-generated schemas validate correctly

### **Integration Testing**
1. **Rust Generation** - Generated Rust code should compile
2. **Runtime Integration** - Tools should load in `toka-tools` runtime
3. **Performance** - Validation and generation should be fast
4. **Security** - Sandbox and capability restrictions should work

### **End-to-End Testing**
1. **LLM Workflow** - LLM generates schema → validates → syncs to Rust
2. **Error Handling** - Invalid schemas should be rejected with clear errors
3. **Production Flow** - Complete workflow from schema to deployed tool

## 📊 **Success Metrics**

### **Technical Metrics**
- [ ] All instances validate against general schemas (100% compliance)
- [ ] Generated Rust code compiles without errors (100% success rate)
- [ ] Validation completes in <5 seconds for typical schemas
- [ ] Integration bridge syncs schemas in <30 seconds

### **Quality Metrics**
- [ ] LLMs can understand schema requirements from examples
- [ ] Invalid schemas are rejected with clear, actionable errors
- [ ] No security violations in generated tools
- [ ] Production tools perform within resource constraints

### **Maintenance Metrics**
- [ ] Schema changes don't break existing instances
- [ ] Adding new capability types is straightforward
- [ ] Documentation is complete and up-to-date
- [ ] Development workflow is streamlined

## 🚨 **Risk Mitigation**

### **Migration Risks**
- **Risk**: Breaking existing tools during migration
- **Mitigation**: Maintain backward compatibility, staged rollout
- **Contingency**: Rollback plan with git branches

### **Validation Risks**
- **Risk**: Too strict validation blocks valid use cases
- **Mitigation**: Comprehensive testing, adjustable rules
- **Contingency**: Validation bypass for emergency cases

### **Integration Risks**
- **Risk**: Generated Rust code doesn't match `toka-tools` patterns
- **Mitigation**: Template validation, manual review for complex tools
- **Contingency**: Manual implementation fallback

## 🎉 **Expected Benefits**

### **For LLMs**
- Clear, minimal context for understanding system requirements
- Strict schemas prevent generation of invalid resources
- Fast validation feedback improves generation quality
- Examples guide correct schema generation

### **For Developers**
- Single source of truth in `toka-tools` for production tools
- Type-safe Rust implementations with compile-time validation
- Clean integration path from specification to implementation
- No tool sprawl or duplicate systems

### **For System**
- Consistent tool architecture across all components
- Security by design with capability-based restrictions
- Performance and reliability through Rust implementations
- Maintainable codebase with clear separation of concerns

---

**Next Actions**: Begin Phase 1 implementation by completing the remaining general schemas and setting up the instances directory structure. 