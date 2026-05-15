# AnswerBuilder Provenance Implementation

## Executive Summary

The `AnswerBuilder` component in CODEX-main 36 **fully populates** all provenance fields required for complete answer traceability:

- ✅ `cited_claim_ids`: Active claims that ground the answer
- ✅ `cited_evidence_ids`: Evidence IDs linked to grounding claims
- ✅ `rejected_action_summary`: Actions considered but not selected

**Status**: COMPLETE implementation verified ✅

---

## AnswerEnvelope Structure

### Provenance Fields (Completeness Verified)

```rust
pub struct AnswerEnvelope {
    // **Basis fields** (complete traceability)
    pub basis: String,                              // "grounded_active_claims" or "insufficient_grounded_claims"
    pub basis_items: Vec<AnswerBasisItem>,         // Full claim records with evidence links
    pub confidence: f64,                            // Aggregated confidence (0.0-1.0)
    
    // **Citation fields** (provenance)
    pub cited_claim_ids: Vec<String>,              // ✅ Active claims included in answer
    pub cited_evidence_ids: Vec<String>,           // ✅ Evidence backing cited claims
    pub rejected_action_summary: Option<String>,   // ✅ Actions NOT selected (via context)
    
    // **Status fields**
    pub action_type: String,                        // "answer", "defer_insufficient_evidence", etc.
    pub warnings: Vec<String>,                      // Contradicted claims, missing evidence links
    pub missing_evidence_reason: Option<String>,   // Why answer was deferred if applicable
    
    // **Content fields**
    pub text: String,                               // User-facing answer text
    pub evidence_ids: Vec<String>,                  // Context evidence IDs (from build context)
}

pub struct AnswerBasisItem {
    pub claim_id: String,                           // Claim ID (for tracing back to archive)
    pub subject: String,                            // Triple subject
    pub predicate: String,                          // Triple predicate
    pub object: Option<String>,                     // Triple object
    pub confidence: f64,                            // Per-claim confidence
    pub evidence_ids: Vec<String>,                  // Evidence backing THIS claim
}
```

---

## Provenance Population Implementation

### 1. Cited Claim IDs (Active Claims)

**Location**: [memory/src/answer_builder.rs:71-75](../global-workspace-runtime-rs/crates/memory/src/answer_builder.rs#L71-L75)

```rust
let cited_claim_ids = active_claims
    .iter()
    .map(|c| c.id.clone())
    .collect::<Vec<_>>();

// Included in AnswerEnvelope:
AnswerEnvelope {
    cited_claim_ids,  // ✅ All active claims that ground the answer
    ...
}
```

**Semantics**:
- Contains claim IDs for all claims with `status == ClaimStatus::Active`
- Used to trace which claims the answer was built from
- Excludes superseded, contradicted, and unverified claims from citation

**Example**:
```
Query: "What color is the sky?"
Claims:
  - c1 (Active, confidence 0.9): "sky is blue"
  - c2 (Contradicted, confidence 0.8): "sky is green"
  - c3 (Superseded): "sky is grey"
  
cited_claim_ids: ["c1"]  // ✅ Only active claim
warnings: ["disputed_claims_present:c2"]  // ⚠️ Contradiction noted
```

---

### 2. Cited Evidence IDs (Evidentiary Chain)

**Location**: [memory/src/answer_builder.rs:157-161](../global-workspace-runtime-rs/crates/memory/src/answer_builder.rs#L157-L161)

```rust
let cited_evidence_ids: Vec<String> = active_claims
    .iter()
    .flat_map(|c| c.evidence_ids.iter().cloned())
    .collect();

// Included in AnswerEnvelope:
AnswerEnvelope {
    cited_evidence_ids,  // ✅ All evidence linked to active claims
    ...
}
```

**Semantics**:
- Flat list of evidence IDs from all active claims
- Allows direct lookup of supporting materials in evidence vault
- Supports full provenance chain: Answer → Claims → Evidence

**Example**:
```
Query: "What color is the sky?"
Claims:
  - c1: {"evidence_ids": ["e1", "e2"]}  (blue-sky observation)
  - c3: {"evidence_ids": ["e4"]}         (contradicted, not included)
  
cited_evidence_ids: ["e1", "e2"]  // ✅ Evidence from active claims only
```

---

### 3. Rejected Action Summary (Decision Context)

**Location**: [memory/src/answer_builder.rs:163-171](../global-workspace-runtime-rs/crates/memory/src/answer_builder.rs#L163-L171)

```rust
let rejected_action_summary = if ctx.rejected_actions.is_empty() {
    None
} else {
    Some(format!(
        "rejected_actions:{}",
        ctx.rejected_actions.join("|")
    ))
};

// Included in AnswerEnvelope:
AnswerEnvelope {
    rejected_action_summary,  // ✅ Actions NOT selected
    ...
}
```

**Semantics**:
- Records alternative actions evaluated but not selected during candidate generation
- Passed via `AnswerBuildContext::rejected_actions`
- Supports full decision audit trail (why action X was chosen over Y)

**Typical Context**:
- From runtime-core's `CandidatePool.filtered_rejected()` 
- From modulation phase outcome exclusions
- From safety gates (e.g., refused_unsafe blocks possible actions)

**Example**:
```
Runtime evaluation:
  Candidates generated: [answer, ask_clarification, defer_insufficient_evidence]
  Candidate filter outcome: 
    - refuse_unsafe: REJECTED answer (unsafe claim)
    - high_confidence: ACCEPTED defer_insufficient_evidence (safe)
    - ask_clarification: REJECTED (evidence sufficient)
  
rejected_action_summary: "rejected_actions:answer|ask_clarification"
action_type: "defer_insufficient_evidence"
```

---

## Completeness Verification

### Test Coverage

**File**: [memory/src/answer_builder.rs Tests](../global-workspace-runtime-rs/crates/memory/src/answer_builder.rs#L170-250)

| Test | Validates |
|------|-----------|
| `includes_active_claim_citations` | ✅ `cited_claim_ids` populated for active claims |
| `contradicted_claims_emit_warning_only` | ✅ Contradicted claims excluded from citations |
| `superseded_and_unverified_excluded_from_answer_body` | ✅ Excludes Superseded/Unverified from citations |
| `active_claims_supply_evidence` | ✅ `cited_evidence_ids` extracted from active claims |
| `confidence_aggregation` | ✅ `confidence` calculated from active claims |

**Result**: All 5+ unit tests PASSING ✅

### Proof Artifacts

**File**: [artifacts/proof/current/answer_quality_report.json](../artifacts/proof/current/answer_quality_report.json)

```json
{
    "pass": true,
    "answer_envelope_fields": {
        "cited_claim_ids": {
            "populated": true,
            "count": 14,
            "mean_per_answer": 2.1
        },
        "cited_evidence_ids": {
            "populated": true,
            "count": 28,
            "mean_per_answer": 4.2
        },
        "rejected_action_summary": {
            "populated": true,
            "non_empty_count": 11,
            "average_rejection_count": 1.8
        }
    }
}
```

### Integration Proof

**File**: [artifacts/proof/current/answer_basis_integration_report.json](../artifacts/proof/current/answer_basis_integration_report.json)

```json
{
    "pass": true,
    "claim_to_evidence_chain": true,
    "evidence_integrity": true,
    "claim_status_filtering": true,
    "basis_item_completeness": true,
    "rejected_action_accounting": true
}
```

---

## Usage Examples

### Example 1: Simple Answer with Ground Truth

```rust
let builder = AnswerBuilder::new();
let claims = vec![
    MemoryClaim {
        id: "c1".to_string(),
        subject: "earth".to_string(),
        predicate: "orbits".to_string(),
        object: Some("sun".to_string()),
        status: ClaimStatus::Active,
        confidence: 0.99,
        evidence_ids: vec!["e_copernicus".to_string()],
        // ...other fields...
    }
];

let answer = builder.build("What does earth orbit?", &claims);

assert_eq!(answer.cited_claim_ids, vec!["c1"]);
assert_eq!(answer.cited_evidence_ids, vec!["e_copernicus"]);
assert_eq!(answer.text, "earth orbits sun");
assert_eq!(answer.confidence, 0.99);
```

### Example 2: Answer with Contradictions

```rust
let claims = vec![
    MemoryClaim {
        id: "c_current".to_string(),
        status: ClaimStatus::Active,
        confidence: 0.95,
        evidence_ids: vec!["e_new_observation"],
        // ...
    },
    MemoryClaim {
        id: "c_old_theory".to_string(),
        status: ClaimStatus::Contradicted,
        confidence: 0.80,
        evidence_ids: vec!["e_historical"],
        // ...
    }
];

let answer = builder.build("Is X true?", &claims);

// Cited claims only include active:
assert_eq!(answer.cited_claim_ids, vec!["c_current"]);
assert_eq!(answer.cited_evidence_ids, vec!["e_new_observation"]);

// Contradicted claims noted in warnings:
assert_eq!(answer.warnings[0], "disputed_claims_present:c_old_theory");
```

### Example 3: Deferred Answer with Rejection Context

```rust
let ctx = AnswerBuildContext {
    action_type: "".to_string(),
    evidence_ids: vec![],
    rejected_actions: vec!["answer".to_string(), "ask_clarification".to_string()],
};

let answer = builder.build_with_context("Complex query", &[], ctx);

assert_eq!(answer.action_type, "defer_insufficient_evidence");
assert_eq!(answer.rejected_action_summary.unwrap(), 
           "rejected_actions:answer|ask_clarification");
assert_eq!(answer.confidence, 0.0);
assert!(answer.cited_claim_ids.is_empty());
```

---

## Data Flow: Query → Answer → UI

```
1. RuntimeStepResult (claim evidence from memory retrieval)
   ↓
2. AnswerBuilder.build_with_context(
     query="What color is the sky?",
     claims=[Active(c1, confidence 0.9, evidence=[e1,e2]), Contradicted(c2,...)],
     context={rejected_actions: [answer_unsafe, ask_clarification]}
   )
   ↓
3. AnswerEnvelope {
     text: "sky is blue",
     basis: "grounded_active_claims",
     basis_items: [AnswerBasisItem{claim_id:"c1", evidence_ids:[e1,e2], ...}],
     cited_claim_ids: ["c1"],  ← Provenance citation
     cited_evidence_ids: ["e1", "e2"],  ← Evidence chain
     confidence: 0.9,
     action_type: "answer",
     rejected_action_summary: "rejected_actions:answer_unsafe|ask_clarification",  ← Decision context
     warnings: ["disputed_claims_present:c2"]
   }
   ↓
4. UI Display (via bridge_trace_cycle.rs):
   - Answer text: "sky is blue"
   - Confidence: 90%
   - Disclaimers: "Contradicted view: c2 (80% confidence)"
   - Citation link: [c1] → [e1, e2]
```

---

## Codename & Status

**CODEX Version**: CODEX-main 36 hardening  
**Priority**: 7 (Verification of existing implementation)  
**Status**: COMPLETE ✅ No changes required

### Reasoning
The AnswerBuilder provenance fields are fully implemented, tested, and integrated into the proof pipeline. All citations flow through to answer quality reports and basis integration reports. No additional work needed for Priority 7.

---

## References

- **Implementation**: [crates/memory/src/answer_builder.rs](../global-workspace-runtime-rs/crates/memory/src/answer_builder.rs)
- **Integration**: [crates/simworld/src/evaluator.rs](../global-workspace-runtime-rs/crates/simworld/src/evaluator.rs) (calls AnswerBuilder.build())
- **Proof**: [artifacts/proof/current/answer_quality_report.json](../artifacts/proof/current/answer_quality_report.json)
- **Tests**: cargo test -p memory --lib

---

**Date**: May 15, 2026  
**Verification**: Automated CI + proof artifacts  
**Review**: CODEX-main 36 hardening assessment
