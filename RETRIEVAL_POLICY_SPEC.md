# Retrieval Policy Enforcement Specification

## Executive Summary

The CODEX retrieval routing system uses **configured advisory flags** that inform decision-making but do not block retrieval—the runtime always performs route evaluation. Flags serve as guidelines for which action type to emit given the retrieval intent, not hard-blocking gates.

**Policy Model**: Deterministic routing based on `RetrievalIntentCategory` + optional policy flags for future extensibility.

---

## Policy Flags (Advisory Scope)

### Flag: `defer_unsupported_factual`
**Default**: `true`  
**Type**: Advisory  
**Scope**: Routes `RetrievalIntentCategory::UnsupportedFactual` queries → `defer_insufficient_evidence` action

**Specification**:
```rust
pub defer_unsupported_factual: bool,
// "Route unsupported factual queries to defer?"
```

**Behavior**:
- When `true` (default): `UnsupportedFactual` intent → emit `defer_insufficient_evidence` action
- Never blocks retrieval attempt itself
- Informs action selection post-retrieval
- Governed-memory router *always* processes the query regardless of this flag

**Usage Context**:
| Intent | Action | Reason Code | Confidence |
|--------|--------|-------------|-----------|
| UnsupportedFactual | defer_insufficient_evidence | retrieval_unsupported_factual | 0.9 |

**Enforcement**: ✅ HARD-ENFORCED at action emission time (always defers unsupported factual)

---

### Flag: `ask_on_ambiguous`
**Default**: `true`  
**Type**: Advisory  
**Scope**: Routes `RetrievalIntentCategory::Ambiguous` queries → `ask_clarification` action

**Specification**:
```rust
pub ask_on_ambiguous: bool,
// "Route ambiguous queries to ask clarification?"
```

**Behavior**:
- When `true` (default): `Ambiguous` intent → emit `ask_clarification` action
- Never blocks retrieval attempt itself
- Informs action selection post-retrieval
- Governed-memory router *always* evaluates the ambiguous query

**Usage Context**:
| Intent | Action | Reason Code | Confidence |
|--------|--------|-------------|-----------|
| Ambiguous | ask_clarification | retrieval_ambiguous_match | 0.8 |

**Enforcement**: ✅ HARD-ENFORCED at action emission time (always asks on ambiguous)

---

### Flag: `refuse_provider_gated`
**Default**: `true`  
**Type**: Advisory  
**Scope**: Routes provider-gated queries → `refuse_unsafe` action (in experimental mode)

**Specification**:
```rust
pub refuse_provider_gated: bool,
// "Refuse provider-gated queries?"
```

**Behavior**:
- When `true` (default): Provider-gated retrieval attempts → log as advisory refused
- Does NOT stop retrieval evaluation
- Provider output is always marked non-authoritative regardless
- Feature-gated behind `ui-local-providers` flag (disabled in default build)

**Note**: In default build, `refuse_provider_gated=true` is informational only (providers already disabled).

**Enforcement**: ✅ ADVISORY (provider already non-authoritative, so effective enforcement via provider policy, not retrieval policy)

---

## Core Routing Decision Logic

The router uses **deterministic, intent-based routing**, NOT advisory flags, as primary decision mechanism:

```rust
match &query.intent_category {
    RetrievalIntentCategory::MemoryLookup => {
        action = "retrieve_memory"            // Always emit this for valid lookups
        confidence = 0.95
    }
    RetrievalIntentCategory::UnsupportedFactual => {
        action = "defer_insufficient_evidence"  // Always defer unsupported factual
        confidence = 0.9
    }
    RetrievalIntentCategory::HighStakesLowEvidence => {
        action = "defer_insufficient_evidence"  // Always defer high-stakes, low-evidence
        reason = high_stakes_threshold_check
        confidence = 0.85
    }
    RetrievalIntentCategory::Ambiguous => {
        action = "ask_clarification"          // Always ask on ambiguous
        confidence = 0.8
    }
}
```

**Key Point**: Intent category *directly determines* action. Policy flags are **not consulted in default routing**—they exist for future policy extensions or logging.

---

## Enforcement vs Advisory: Clarification

### Clearly Enforced (Hard Boundaries)
1. ✅ **Unsupported factual queries ALWAYS defer** (intent-based)
2. ✅ **Ambiguous queries ALWAYS ask clarification** (intent-based)
3. ✅ **High-stakes + low-evidence ALWAYS defer** (confidence check)
4. ✅ **Provider output ALWAYS non-authoritative** (provider_policy_report.pass=true)
5. ✅ **External provider requests ALWAYS 0** (tool_policy enforcement)

### Advisory/Future-Extensible (Not Currently Enforced)
1. ⚠️ `defer_unsupported_factual` flag value not checked in current routing (always defers via intent)
2. ⚠️ `ask_on_ambiguous` flag value not checked in current routing (always asks via intent)
3. ⚠️ `refuse_provider_gated` flag value not checked (providers disabled in default build anyway)

---

## Specification vs Implementation Gap

**Gap Category**: The `RetrievalPolicy` flags are **structurally defined but not actively consulted** in v36 routing logic.

**Why**:
- Routing is intent-based and deterministic
- Flags were designed for extensible multi-policy scenarios (future phases)
- Current build has single policy, so flags are informational

**Implication for Hardening**:
- **No security gap**: Intent-based routing is stricter than flag-based routing would be
- **No behavioral gap**: Current action emission matches intended policy
- **Clarification needed**: Update comments/docs to reflect that flags are advisory/future placeholders

---

## Recommended Action (Priority 6 Fix)

### 1. Update policy.rs Documentation
```rust
/// Retrieval policy decision rules.
///
/// NOTE: Policy flags are currently **advisory and informational only**.
/// The active retrieval routing logic is intent-based (see retrieval_intent.rs).
/// Policy flags are structured for future extensibility but not consulted
/// in v36 routing decisions.
///
/// Intent categories drive action selection directly:
/// - MemoryLookup: emit "retrieve_memory"
/// - UnsupportedFactual: emit "defer_insufficient_evidence"
/// - HighStakesLowEvidence: emit "defer_insufficient_evidence"
/// - Ambiguous: emit "ask_clarification"
impl RetrievalPolicy {
    ...
}
```

### 2. Update PROOF_MODEL.md
Add section:
```markdown
## Retrieval Policy Scope

**Design**: Intent-based deterministic routing with advisory policy flags.

**Flags (Advisory)**:
- `defer_unsupported_factual`: Future extensibility hook (currently always true behavior)
- `ask_on_ambiguous`: Future extensibility hook (currently always true behavior)
- `refuse_provider_gated`: Future extensibility hook (provider disabled in default build)

**Active Enforcement**: Query intent category determines action emission via hard-coded routing table.
```

### 3. Update retrieval_policy_enforcement_report.json
Add validation field:
```json
{
    "flag_status": "advisory_informational",
    "intent_based_routing": true,
    "flags_actively_consulted": false,
    "reasoning": "Intent categories drive deterministic action selection; flags reserved for future multi-policy scenarios",
    "security_impact": "NONE: intent-based routing is stricter than advisory flags would allow"
}
```

---

## Current Proof Status

**Assertion**: All retrieval intent routes are correctly implemented and enforced.

**Evidence**:
- [governed-memory/src/retrieval_intent.rs](../global-workspace-runtime-rs/crates/governed-memory/src/retrieval_intent.rs): Deterministic routing table
- [artifacts/proof/current/retrieval_policy_enforcement_report.json](../artifacts/proof/current/retrieval_policy_enforcement_report.json): Pass=true validation

**Test Coverage**:
- Unit tests: `governed_memory_route_* tests in integration_tests.rs`
- Proof: `nl_benchmark_report.json` includes ambiguous/unsupported/high-stakes scenarios
- Security: `governed_memory_integration_report.json` confirms routing integrity

---

## Changelog

**For PHASE_16**:
- [ ] Update policy.rs doc comments to clarify advisory nature
- [ ] Extend `retrieval_policy_enforcement_report.json` with flag consultation status
- [ ] Add PROOF_MODEL.md section on flag scope vs active routing
- [ ] Consider implementing policy-flag-driven routing as opt-in feature (future)

**For Future**:
- [ ] Multi-policy scenarios where flags become actively consulted
- [ ] Configuration-driven routing engine (Phase 17+)
- [ ] A/B testing of policy variants against benchmark

---

## FAQ

**Q: Are the policy flags enforced?**  
A: Intent categories drive enforcement; flags are advisory. Current behavior matches intended design (all unsupported factual defers, all ambiguous asks clarification).

**Q: Does this impact security?**  
A: No. Intent-based hard-coded routing is equivalent to or stricter than advisory flag-based routing.

**Q: Can I turn off defer_unsupported_factual?**  
A: In v36, no—it's intent-based. In future phases (v37+), if flag-driven routing is implemented, yes.

**Q: Why keep the flags if they're not used?**  
A: Schema preservation for future multi-policy extension. Defined but not actively consulted is safer than adding flags later.

---

**Codename**: CODEX-main 36 hardening specification  
**Priority**: 6 (Documentation/Clarification)  
**Status**: Documented ✅
