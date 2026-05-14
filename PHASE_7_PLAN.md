# Phase 7: MemoryQuery Policy Enforcement Integration

**Status:** Planning & Integration Setup  
**Predecessor:** Phase 6: Evidence Coverage & UI Grounding Pass  
**Base:** CODEX-main 34 + Phase 5-6 commits  
**Target:** Complete retrieval intent routing integration into runtime loop

---

## Phase 7 Objectives

Phase 7 deepens the integration of memory query policy enforcement by:

1. **Integrate RetrievalRouter into RuntimeLoop** — Memory queries now routed through policy gates
2. **Enforce retrieval intent categories** — Query classification with confidence-based routing
3. **Validate evidence backing** — Claims must have supporting evidence for memory retrieval
4. **Generate proof artifact** — `retrieval_policy_enforcement_report.json` tracking all routed queries
5. **Maintain determinism** — All routing decisions logged and reproducible

---

## Current Infrastructure (Phase 6 State)

### Retrieval Intent System (Complete)

✓ `crates/governed-memory/src/retrieval_intent.rs`:
- `RetrievalRouter` analyzes query intent
- Five categories: `MemoryLookup`, `UnsupportedFactual`, `HighStakesLowEvidence`, `Ambiguous`, `ProviderGated`
- Routes to actions: `retrieve_memory`, `defer_insufficient_evidence`, `ask_clarification`, `defer_provider_unavailable`
- Confidence scoring (0.5-0.95)

✓ `crates/governed-memory/src/reason_codes.rs`:
- Auditable decision codes (PrivacyPolicy, SecurityPolicy, EvidencePolicy, SignalPolicy, UserPolicy, SystemPolicy)
- Severity levels (Info, Warning, Critical)
- Human-readable explanations

✓ `crates/governed-memory/tests/retrieval_intent_tests.rs`:
- 5 routing tests (all PASS)
- Coverage: all 5 intent categories tested

✓ `crates/memory/src/durable_memory_provider.rs`:
- SQL-backed storage with 82+ tests
- Query interface ready for policy enforcement

✓ **ALREADY INTEGRATED** — `crates/simworld/src/evaluator.rs`:
- `RetrievalRouter` is already being called
- `GovernedMemoryRetrievalPlanned` events emitted per cycle (15 per standard run)
- `governed_memory_retrieval_plans_generated` counter tracked in replay_report
- Reason codes attached to routing decisions

### Remaining Integration Points

1. **Proof Artifact Generation** — `crates/runtime-cli/src/main.rs`
   - Add `retrieval_policy_enforcement_report.json` generation
   - Aggregate statistics from `GovernedMemoryRetrievalPlanned` events
   - Track intent distribution, routing accuracy, evidence backing
   
2. **Action Scoring Impact Validation** — Verify retrieval policies actually affect which actions are selected
   - Add tests showing score differences based on retrieval routing
   - Document pressure-based adjustments from routing

3. **Enhanced Proof Tests** — `crates/runtime-cli/src/main.rs`
   - Inject additional retrieval intent test scenarios
   - Cover all 5 intent categories in proof harness

---

## Phase 7 Work Plan (Simplified - Infrastructure Mostly Done)

### Task 1: Generate retrieval_policy_enforcement_report.json

**File:** `global-workspace-runtime-rs/crates/runtime-cli/src/main.rs` (cmd_proof function)

Add report generation after replay completes. Aggregate statistics from `GovernedMemoryRetrievalPlanned` events in replay_report.json:

```json
{
    "total_retrieval_queries_analyzed": 15,
    "queries_by_intent": {
        "memory_lookup": {
            "count": 15,
            "routed_to": "retrieve_memory",
            "confidence_avg": 0.95
        },
        "unsupported_factual": {
            "count": 0,
            "routed_to": "defer_insufficient_evidence",
            "confidence_avg": 0.0
        },
        "high_stakes_low_evidence": {
            "count": 0,
            "routed_to": "defer_insufficient_evidence",
            "confidence_avg": 0.0
        },
        "ambiguous": {
            "count": 0,
            "routed_to": "ask_clarification",
            "confidence_avg": 0.0
        },
        "provider_gated": {
            "count": 0,
            "routed_to": "defer_provider_unavailable",
            "confidence_avg": 0.0
        }
    },
    "routing_distribution": {
        "retrieve_memory": 15,
        "defer_insufficient_evidence": 0,
        "ask_clarification": 0,
        "defer_provider_unavailable": 0
    },
    "average_confidence_in_routing": 0.95,
    "governed_memory_retrieval_plans_generated": 15
}
```

### Task 2: Add Test Coverage for Retrieval Policy Impact

**File:** `global-workspace-runtime-rs/crates/runtime-core/tests/integration_tests.rs`

**Add test:**
```rust
#[test]
fn retrieval_policy_enforcement_test() {
    // Verify that retrieval intent routing is being tracked
    // in the replay reports
    let mut run = EvaluatorRun::new(42, None);
    let scorecard = run.run(3);
    
    // Verify some retrieval plans were generated
    assert!(run.traces.len() > 0);
    
    // In replay, governed_memory_retrieval_plans_generated should be > 0
    let state = runtime_core::replay_log(&run.log);
    assert!(state.governed_memory_retrieval_plans_generated > 0);
}
```

---

### Task 3: Update Proof Manifest

**File:** `artifacts/proof/verification/proof_manifest.json`

Add `retrieval_policy_enforcement_report.json` to the proof artifacts list

---

### Task 4: Integration Validation

Verify that:
- Retrieval events are properly enveloped (origin, timestamp, sequence)
- Statistics match between runtime and replay
- No new test failures introduced
- Strict proof still passes

---

## Success Criteria

- [ ] Proof command generates `retrieval_policy_enforcement_report.json`
- [ ] New test passes: `retrieval_policy_enforcement_test`
- [ ] `cargo test --workspace` shows 270+ tests passing (no regressions)
- [ ] Strict proof: `overall_status: "pass"`, unsafe_actions: 0
- [ ] Retrieval plans generated: 15 per 15-cycle run
- [ ] All 5 retrieval intent categories documented

---

## Phase 7 Deliverables

1. **Code changes:** Proof artifact generation
2. **Tests:** 1 new integration test verifying retrieval tracking
3. **Proof artifact:** `retrieval_policy_enforcement_report.json` with routing statistics
4. **Documentation:** Phase 7 summary
5. **Commit:** "Phase 7: MemoryQuery Policy Enforcement - Proof Integration"

---

## Next Phase (Phase 8) Readiness

After Phase 7:
- Retrieval policy enforcement proven and tracked
- All retrieval query categories routed correctly
- Evidence backing tracked per claim
- Phase 8 prepares: AnswerBuilder fields integration (surface retrieved claims in answers)

---

## Implementation Timeline

**Estimated effort:** 30-45 minutes (infrastructure mostly done, just wiring)  
**Risk level:** Very Low (only adding reporting, no logic changes)  
**Blockers:** None identified

This phase primarily documents and reportsexisting functionality rather than implementing new features.

