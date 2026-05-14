# Phase 8: AnswerBuilder Field Enhancement & Proof Integration

**Status:** Planning  
**Predecessor:** Phase 7: MemoryQuery Policy Enforcement (1a1e66b)  
**Base:** CODEX-main 35 (Phases 5-7 complete)  
**Next:** Phase 9 onwards  

---

## Phase 8: Objectives

Phase 8 enhances AnswerBuilder integration by:

1. **Validate AnswerBasisItem population** — Ensure all claim fields properly serialized
2. **Test evidence linking in answers** — Verify evidence_ids propagated correctly
3. **Generate answer_basis_integration_report** — Track answer quality metrics
4. **Validate answer envelope JSON** — Ensure schema compliance
5. **Document answer grounding** — Link claims to evidence in generated answers

---

## Current AnswerBuilder State

### Existing Implementation (v0.8+)

✓ `crates/memory/src/answer_builder.rs`:
- `AnswerBasisItem` struct with claim fields (subject, predicate, object, confidence, evidence_ids)
- `AnswerEnvelope` response contract
- Claim lifecycle policy (Active/Contradicted/Superseded)
- Build methods with context support

✓ `ui/codex-dioxus/src/bridge/runtime_client.rs`:
- UI bridge uses AnswerBuilder
- Surfaces `MemoryClaim` objects with real content fields
- Builds `AnswerEnvelope` for frontend display

✓ Tests: `crates/memory/tests/answer_builder_tests.rs`
- Claim lifecycle filtering tests
- Evidence linking tests
- Confidence computation tests

### Phase 8 Work Items

1. **Add Answer Generation Tracking Event** — New proof harness event
2. **Generate Answer Quality Report** — Aggregate statistics
3. **Validate Field Serialization** — JSON round-trip tests
4. **Document Answer Grounding** — Show evidence-to-claim-to-answer path

---

## Implementation Plan

### Task 1: Create Answer Quality Tracking

**File:** New Python script `scripts/generate_answer_quality_report.py`

Extract from proof run:
- Total answers generated
- Average basis items per answer
- Evidence coverage rate
- Claimed count vs evidence count ratio
- Answer confidence distribution

### Task 2: Add Integration Test

**File:** `crates/memory/tests/answer_builder_tests.rs`

New test: `answer_builder_with_evidence_links_serializes_correctly`
- Create claims with evidence backing
- Build answer envelope
- Verify evidence_ids in AnswerBasisItem
- JSON round-trip validation

### Task 3: Generate Proof Artifact

**File:** `artifacts/proof/current/answer_quality_report.json`

Report format:
```json
{
  "answer_generation": {
    "total_answers_generated": N,
    "basis_items_per_answer_avg": X.Y,
    "evidence_coverage_rate": 0.95,
    "answer_confidence_avg": 0.85
  },
  "claim_grounding": {
    "total_claims_cited": 17,
    "total_evidence_links": 17,
    "evidence_per_claim_avg": 1.0
  },
  "schema_validation": {
    "json_schema_version": "1.0",
    "answer_envelope_fields": [...],
    "validation_status": "pass"
  }
}
```

### Task 4: Update Proof Manifest

Add `answer_quality_report.json` to proof artifacts list

---

## Success Criteria

- [ ] New integration test passes
- [ ] Python script generates valid report
- [ ] Answer envelope schema valid
- [ ] No regressions in existing 270+ tests
- [ ] Proof command still passes with `overall_status: "pass"`

---

## Tasks Remaining

1. Write `generate_answer_quality_report.py`
2. Add integration test to answer_builder_tests.rs
3. Run proof and generate artifact
4. Update proof manifest
5. Commit Phase 8

---

## Estimated Effort

**Time:** 30-45 minutes  
**Risk:** Very Low (mostly reporting/validation)  
**Blockers:** None identified
