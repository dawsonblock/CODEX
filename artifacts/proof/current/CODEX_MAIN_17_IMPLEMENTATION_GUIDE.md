# CODEX-main 17 Analysis — Implementation Guide & Remediation

**Analysis Date:** May 15, 2026  
**Current Status:** CODEX-main 36 with strong hardening candidate profile  
**Proof Checker Status:** ✅ PASSING (including identity drift verification)

---

## Status Summary

### ✅ Completed Improvements (Verified)
1. **Identity Drift Fixed** — All source/UI/artifact files consistently use CODEX-main 36
   - runtime-cli/src/main.rs: ✅ "CODEX-main 36 hardening candidate"
   - ui/codex-dioxus/src/app.rs: ✅ "Codex-main 36"
   - ui/codex-dioxus/src/components/runtime_status.rs: ✅ "Codex-main 36"
   - provider_policy_report.json: ✅ "CODEX-main 36 hardening candidate"
   - Proof checker validates: ✅ PASSED

2. **Proof Manifest Consistency** — All 22 proof artifacts registered and verified
   - check_proof_manifest_consistency.py: ✅ PASSING
   - Active codename check implemented and active
   - Stale marker detection working

3. **Core Boundary Enforcement** — Security assertions active
   - real_external_executions: ✅ 0 (verified strict)
   - provider_can_execute_tools: ✅ false
   - provider_can_write_memory: ✅ false
   - provider_can_override_codex_action: ✅ false

4. **NL Benchmark Failures Documented** — Comprehensive analysis complete
   - 5 failures analyzed with root cause and remediation paths
   - Phase 11 implementation plan provided
   - Acceptance criteria defined

### ⚠️ Remaining Improvements (Actionable)

These items provide architectural completeness and policy enforcement:

1. **[HIGH] Add UI Provider Feature Test Log**
   - Status: Not run in current proof cycle
   - Impact: Incomplete feature verification
   - Solution: Either run `cargo test --all-targets --features ui-local-providers` or document absence

2. **[HIGH] Enforce MemoryQuery Policy Flags**
   - Status: Fields exist but not enforced
   - Impact: Policy framework advisory, not authoritative
   - Solution: Implement actual filter logic or rename report to "routing diagnostics"

3. **[MEDIUM] Populate AnswerBuilder Metadata**
   - Status: cited_evidence_ids and rejected_action_summary remain empty
   - Impact: Answer provenance contract incomplete
   - Solution: Wire evidence reference collection and action rejection reasons

4. **[MEDIUM] Expose Answer Metadata in UI Bridge**
   - Status: Basis items visible, but provenance fields missing
   - Impact: UI cannot display complete answer justification
   - Solution: Export answer_confidence, cited_claim_ids, cited_evidence_ids

5. **[MEDIUM] Add EventOrigin Variants**
   - Status: Only 5 variants, many subsystems use default RuntimeLoop
   - Impact: Coarse provenance attribution
   - Solution: Add Ui, TestFixture, MemoryStore, EvidenceStore, ProviderGate, AnswerBuilder, SimWorld

6. **[LOW] Complete Durable Memory Schema**
   - Status: Recent table has 16 fields, target design had 24
   - Impact: Some query optimization opportunities lost
   - Solution: Add entities_json, tags_json, salience, valid_from/valid_to, etc.

---

## Implementation Roadmap

### Phase 1 (This Week): High-Impact Fixes
**Focus:** Enforcement & provenance completeness

**Item 1A: Add UI Provider Feature Test Log**
- **Goal:** Generate proof evidence for ui-local-providers build
- **Command:** `cargo test --all-targets --features ui-local-providers --out artifacts/proof/verification/ui_provider_feature_tests.log`
- **Acceptance:** Log shows test count, pass/fail rate
- **Effort:** 1-2 hours (depends on feature build health)

**Item 1B: MemoryQuery Policy Enforcement**
- **Goal:** Enforce or document actual policy implementation
- **Decision Point:**
  - **Option A (Recommended):** Implement filtering logic for include_stale, include_disputed, require_evidence, etc.
  - **Option B:** Rename retrieval_policy_enforcement to "QueryRoutingDiagnostics" with disclaimer
- **Files:** `src/memory_query.rs`, `src/governed_memory.rs`
- **Effort:** 3-5 days (Option A), 1 day (Option B)

**Item 2: AnswerBuilder Metadata Population**
- **Goal:** Wire cited_evidence_ids and rejected_action_summary
- **Files:**
  - `src/answer_builder.rs` — Collect evidence references from claims
  - `src/retrieval_policy.rs` — Track rejection reasons
- **Integration Points:**
  - Claim resolution: collect evidence_ids
  - Action filtering: log rejection reasons
- **Effort:** 2-3 days

### Phase 2 (Next Week): Integration & Visibility
**Focus:** User-facing provenance, instrumentation

**Item 3: UI Bridge Metadata Exposure**
- **Goal:** Export answer_confidence, cited_claim_ids, cited_evidence_ids in UI response
- **Files:**
  - `src/ui_bridge.rs` — Add fields to AnswerResponse
  - `ui/codex-dioxus/src/components/answer_display.rs` — Render metadata
- **New UI Elements:**
  - Answer confidence badge (0-100%)
  - "Based on claims:" list with links
  - "Evidence references:" expandable section
- **Effort:** 2-3 days

**Item 4: EventOrigin Expansion**
- **Goal:** Add subsystem provenance variants, wire at boundaries
- **New Variants:**
  ```rust
  enum EventOrigin {
      RuntimeLoop,        // ✅ existing
      Evaluator,          // ✅ existing
      ClaimStore,         // ✅ existing
      ToolGate,           // ✅ existing
      ProofHarness,       // ✅ existing
      // NEW:
      Ui,                 // User input from UI
      TestFixture,        // Unit/integration test scenarios
      MemoryStore,        // Durable memory operations
      EvidenceStore,      // Evidence repository updates
      ProviderGate,       // Provider request/response
      AnswerBuilder,      // Answer composition
      SimWorld,           // Simulation environment
      RetrievalPolicy,    // Query policy decisions
  }
  ```
- **Integration:** Update all subsystem callsites: `event_log.append_with_origin(origin, event)`
- **Files:** `src/event_log.rs`, subsystem edges
- **Effort:** 2-3 days

### Phase 3 (Following Week): Schema Completion
**Focus:** Optional future-proofing

**Item 5: Durable Memory Schema Completion**
- **Missing Fields:**
  - entities_json: JSON array of related entity references
  - tags_json: JSON array of classification tags
  - content: Full text for full-text search
  - salience: Frequency/importance score
  - valid_from, valid_to: Temporal scope
  - evidence_links_json: Direct evidence references
  - last_retrieved_at: Query optimization hint
- **Migration:** Backfill historical records or create as optional
- **Effort:** 2-3 days + testing

---

## Detailed Implementation: Item 1A (UI Provider Feature Test Log)

### Current State
- Feature flag `ui-local-providers` compiles zero provider HTTP paths in default build
- UI tests pass (76/82) in default build
- Feature-gated build NOT separately tested

### Goal
Generate proof evidence that feature-gated provider UI code passes tests

### Steps

**Step 1:** Add test log generation to Cargo manifest
```toml
# In Cargo.toml [dev-dependencies]
[dev-dependencies]
# ... (add any missing test dependencies)
```

**Step 2:** Create test wrapper script
```bash
#!/bin/bash
# scripts/test_ui_provider_features.sh

set -e

echo "Testing UI provider features..."
cargo test --all-targets --features ui-local-providers 2>&1 | tee artifacts/proof/verification/ui_provider_feature_tests.log

# Parse results
PASSED=$(grep -c "test result: ok" artifacts/proof/verification/ui_provider_feature_tests.log || echo 0)
FAILED=$(grep -c "test result: FAILED" artifacts/proof/verification/ui_provider_feature_tests.log || echo 0)

echo "{\"feature\": \"ui-local-providers\", \"passed\": $PASSED, \"failed\": $FAILED, \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" > artifacts/proof/verification/ui_provider_feature_test_summary.json

if [ $FAILED -ne 0 ]; then
  echo "FAIL: Feature tests failed"
  exit 1
fi

echo "PASS: All feature tests passed"
exit 0
```

**Step 3:** Integrate into proof harness
```rust
// In runtime-cli proof command
fn cmd_proof(...) {
    // ... existing proof tests ...
    
    // NEW: UI provider feature test if --ui-features flag
    if args.contains(&"--ui-features") {
        run_ui_provider_feature_tests()?;
    }
}
```

### Acceptance Criteria
- ✅ Log file at `artifacts/proof/verification/ui_provider_feature_tests.log`
- ✅ Contains test count and pass/fail summary
- ✅ All feature-gated tests pass OR documented failure reasons
- ✅ Summary JSON with test metrics
- ✅ Integrated into proof cycle (optional flag)

---

## Detailed Implementation: Item 1B (MemoryQuery Enforcement)

### Current State
```rust
pub struct MemoryQuery {
    pub include_stale: bool,
    pub include_disputed: bool,
    pub require_evidence: bool,
    pub exclude_denied: bool,
    pub governance_only: bool,
    // ... other fields
}
```

**Problem:** These fields exist but are not actually used in filtering logic.

### Goal
Either enforce them or document them as advisory

### Option A: Implement Actual Enforcement

**Step 1:** Update MemoryStore query logic
```rust
fn execute_query(&self, query: &MemoryQuery) -> Vec<Claim> {
    let mut results = self.records.iter();
    
    // Apply include_stale filter
    if !query.include_stale {
        results = results.filter(|r| !r.is_stale);
    }
    
    // Apply include_disputed filter
    if !query.include_disputed {
        results = results.filter(|r| !r.is_disputed);
    }
    
    // Apply require_evidence filter
    if query.require_evidence {
        results = results.filter(|r| !r.evidence_ids.is_empty());
    }
    
    // Apply exclude_denied filter
    if query.exclude_denied {
        results = results.filter(|r| r.governance_reason_code != "DENIED");
    }
    
    // Apply governance_only filter
    if query.governance_only {
        results = results.filter(|r| !r.governance_reason_code.is_empty());
    }
    
    results.collect()
}
```

**Step 2:** Add tests for each filter
```rust
#[test]
fn test_query_exclude_stale() {
    // Create claim marked stale, verify excluded
}

#[test]
fn test_query_include_stale() {
    // Create claim marked stale, verify included when flag set
}
// ... more tests
```

**Step 3:** Update retrieval_policy_enforcement_report
- Change from advisory language to enforcement language
- Add metrics: "filters applied", "records filtered out per criterion"
- Document enforcement boundaries

### Option B: Downgrade Report Language (Quick Fix)
**Rename:** retrieval_policy_enforcement_report.json → retrieval_policy_routing_diagnostics.json
**Update language:** "Advisory query routing information" instead of "Enforcement"

**Effort:** Option A = 3-5 days. Option B = 1 day.
**Recommendation:** **Option A** (full enforcement) for completeness.

---

## Detailed Implementation: Item 2 (AnswerBuilder Metadata)

### Current State
```rust
pub struct AnswerEnvelope {
    pub cited_claim_ids: Vec<String>,        // Always empty []
    pub cited_evidence_ids: Vec<String>,     // Always empty []
    pub rejected_action_summary: Option<String>,  // Always None
    pub missing_evidence_reason: Option<String>,
    pub confidence: Option<f32>,
    pub basis_items: Vec<AnswerBasisItem>,
}
```

### Goal
Populate these fields with actual data during answer composition

### Implementation Steps

**Step 1:** Collect claim references during evidence retrieval
```rust
fn compose_answer(&self, query: &Query, context: &CompositionContext) -> AnswerEnvelope {
    let mut cited_claim_ids = Vec::new();
    let mut cited_evidence_ids = Vec::new();
    
    // During claim retrieval:
    for claim in retrieved_claims {
        cited_claim_ids.push(claim.id.clone());
        cited_evidence_ids.extend(claim.evidence_ids.clone());
    }
    
    // ... rest of composition
    
    AnswerEnvelope {
        cited_claim_ids,
        cited_evidence_ids,
        rejected_action_summary: format!("Considered {} actions, selected {}", 
            candidates.len(), selected_action),
        missing_evidence_reason: self.detect_evidence_gaps(query),
        confidence: Some(self.compute_confidence(&cited_claim_ids)),
        basis_items,
    }
}
```

**Step 2:** Track rejection reasons during action filtering
```rust
fn filter_actions(&self, candidates: &[ActionOption]) -> (ActionOption, Option<String>) {
    let mut rejection_reasons = Vec::new();
    
    for candidate in candidates {
        if self.violates_policy(candidate) {
            rejection_reasons.push(format!("Policy violation: {}", candidate.action));
        }
        if !self.has_sufficient_evidence(candidate) {
            rejection_reasons.push(format!("Insufficient evidence for: {}", candidate.action));
        }
    }
    
    let selected = ... ; // best remaining candidate
    let summary = if rejection_reasons.is_empty() {
        None
    } else {
        Some(format!("Rejected: {}", rejection_reasons.join("; ")))
    };
    
    (selected, summary)
}
```

**Step 3:** Add tests
```rust
#[test]
fn test_answer_includes_cited_claims() {
    let answer = builder.compose(&query);
    assert!(!answer.cited_claim_ids.is_empty());
}

#[test]
fn test_answer_includes_evidence_references() {
    let answer = builder.compose(&query);
    assert!(!answer.cited_evidence_ids.is_empty());
}

#[test]
fn test_answer_rejection_summary_populated() {
    let answer = builder.compose(&conflicting_query);
    assert!(answer.rejected_action_summary.is_some());
}
```

---

## Detailed Implementation: Item 3 (UI Bridge Metadata Exposure)

### Current State
```rust
pub struct AnswerResponse {
    pub answer: String,
    pub action: String,
    pub answer_basis: Option<Vec<(String, String)>>,  // claim_id, evidence summary
    pub answer_basis_items: Option<Vec<BasisItem>>,
    pub answer_warnings: Option<Vec<String>>,
    pub missing_evidence_reason: Option<String>,
}
```

### Goal
Add first-class fields for complete provenance

### Changes

**Step 1:** Extend AnswerResponse
```rust
pub struct AnswerResponse {
    pub answer: String,
    pub action: String,
    pub answer_confidence: Option<f32>,               // NEW
    pub cited_claim_ids: Option<Vec<String>>,         // NEW
    pub cited_evidence_ids: Option<Vec<String>>,      // NEW
    pub rejected_action_summary: Option<String>,      // NEW
    pub answer_basis: Option<Vec<(String, String)>>,
    pub answer_basis_items: Option<Vec<BasisItem>>,
    pub answer_warnings: Option<Vec<String>>,
    pub missing_evidence_reason: Option<String>,
}

impl AnswerResponse {
    fn from_envelope(envelope: &AnswerEnvelope) -> Self {
        Self {
            answer: envelope.answer.clone(),
            action: envelope.action.clone(),
            answer_confidence: envelope.confidence,
            cited_claim_ids: Some(envelope.cited_claim_ids.clone()),
            cited_evidence_ids: Some(envelope.cited_evidence_ids.clone()),
            rejected_action_summary: envelope.rejected_action_summary.clone(),
            // ... rest of fields
        }
    }
}
```

**Step 2:** Update UI display
```jsx
// ui/codex-dioxus/src/components/answer_display.rs

rsx! {
    div {
        // Answer text
        p { "{response.answer}" }
        
        // NEW: Confidence badge
        if let Some(confidence) = response.answer_confidence {
            div { class: "confidence-badge",
                "Confidence: {(confidence * 100).round()}%"
            }
        }
        
        // NEW: Cited claims
        if let Some(claims) = &response.cited_claim_ids {
            if !claims.is_empty() {
                div { class: "citation-section",
                    h4 { "Based on claims:" }
                    ul {
                        for claim_id in claims {
                            li { "{claim_id}" }
                        }
                    }
                }
            }
        }
        
        // NEW: Evidence references
        if let Some(evidence) = &response.cited_evidence_ids {
            if !evidence.is_empty() {
                details {
                    summary { "Evidence ({} references)", evidence.len() }
                    ul {
                        for ev in evidence {
                            li { "{ev}" }
                        }
                    }
                }
            }
        }
        
        // Existing basis items
        if let Some(basis) = &response.answer_basis_items {
            display_basis_items(basis)
        }
    }
}
```

---

## Detailed Implementation: Item 4 (EventOrigin Expansion)

### Current State
```rust
pub enum EventOrigin {
    RuntimeLoop,
    Evaluator,
    ClaimStore,
    ToolGate,
    ProofHarness,
}
```

### Goal
Add 8+ subsystem origins for fine-grained provenance

### Expansion
```rust
pub enum EventOrigin {
    // Existing (5)
    RuntimeLoop,
    Evaluator,
    ClaimStore,
    ToolGate,
    ProofHarness,
    
    // New Subsystems (8)
    Ui,                 // User input, UI interactions
    TestFixture,        // Test harness, unit/integration test setup
    MemoryStore,        // Durable memory operations
    EvidenceStore,      // Evidence repository updates
    ProviderGate,       // Provider request/response boundary
    AnswerBuilder,      // Answer composition engine
    SimWorld,           // Simulation environment
    RetrievalPolicy,    // Retrieval policy decision point
}
```

### Integration Points

**In UI (Dioxus):**
```rust
event_log.append_with_origin(EventOrigin::Ui, RuntimeEvent {
    event_type: "user_query",
    payload: query_text,
});
```

**In MemoryStore:**
```rust
pub fn store_claim(&mut self, claim: &Claim) {
    self.events.append_with_origin(EventOrigin::MemoryStore, RuntimeEvent {
        event_type: "claim_stored",
        payload: format!("{:?}", claim),
    });
}
```

**In AnswerBuilder:**
```rust
fn compose_answer(&self, ...) -> AnswerEnvelope {
    self.event_log.append_with_origin(EventOrigin::AnswerBuilder, RuntimeEvent {
        event_type: "answer_composed",
        payload: format!("action={}, confidence={}", action, confidence),
    });
    // ...
}
```

### Acceptance Criteria
- ✅ All 13 variants defined
- ✅ At least 10 callsites use `append_with_origin` with specific origin
- ✅ EventLog tests verify origin is preserved
- ✅ Proof report shows origin distribution

---

## Effort Summary & Timeline

| Item | Effort | Priority | Timeline |
|------|--------|----------|----------|
| Item 1A: UI feature test log | 1-2 hrs | HIGH | Day 1 |
| Item 1B: MemoryQuery enforcement | 1-5 days | HIGH | Days 2-4 |
| Item 2: AnswerBuilder metadata | 2-3 days | MEDIUM | Days 5-7 |
| Item 3: UI bridge exposure | 2-3 days | MEDIUM | Days 8-10 |
| Item 4: EventOrigin expansion | 2-3 days | MEDIUM | Days 11-13 |
| Item 5: Memory schema completion | 2-3 days | LOW | Optional |
| **TOTAL** | **10-19 days** | — | 2-4 weeks |

---

## Testing & Validation

### Unit Tests
- All new metadata fields in answer composition
- Event origin preservation through lifecycle
- Query filter application in memory store
- UI bridge serialization/deserialization

### Integration Tests
- End-to-end answer with complete provenance
- Feature-gated UI tests passing
- Event log contains all subsystem origins
- Proof manifest consistency passing

### Manual Validation
- UI displays confidence, claims, evidence
- Proof logs show origin distribution
- No regressions in existing functionality
- Metadata fields populated in all answer paths

---

## Next Steps

1. **This week:** Create PR for Item 1A (UI feature test log)
2. **Next week:** Items 1B & 2 (enforcement + metadata)
3. **Week 3:** Items 3 & 4 (UI + origins)
4. **Week 4+:** Item 5 (optional) + comprehensive testing

---

**Status:** Ready for phased implementation  
**Owner:** CODEX-main 36 team  
**Review:** Weekly governance checkpoint
