# Phase 9 Plan: UI Integration & Final Verification

**Phase:** 9  
**Predecessor:** Phase 8 (052dcb7) — AnswerBuilder Field Enhancement  
**Status:** PLANNING  
**Target:** Complete UI integration and final verification for Phases 5-9  
**Test Baseline:** 229+ tests passing (no regressions allowed)  

---

## Phase 9 Objectives

After Phases 5-8 completing the event sourcing, evidence tracking, retrieval policy, and answer grounding pipelines, Phase 9 focuses on **UI integration and comprehensive proof** of the complete runtime.

### Primary Goals

1. **UI Dashboard Enhancement**
   - Integrate AnswerEnvelope field data into runtime_client.rs responses
   - Display claim basis items with evidence references
   - Show confidence scores and policy warnings
   - Render lifecycle state (Active, Contradicted, Superseded)

2. **Timeline Visualization**
   - Show claim creation sequence (ordered by timestamp)
   - Display evidence collection timeline
   - Illustrate contradiction and supersession events
   - Mark retrieval decision points

3. **Pressure Dynamics Display**
   - Show action score trends across cycles
   - Display pressure modulation effects
   - Illustrate policy blocking/allowing decisions
   - Trace confident vs. deferring behavior

4. **Long-Horizon Trace Inspection**
   - Enhanced trace with all 15 cycles visible
   - Per-cycle breakdown: claims, evidence, actions taken
   - Streaming view of reasoning chain
   - Cross-reference between cycles

5. **Final Proof Generation**
   - Create `ui_integration_report.json` artifact
   - Document UI bridge compliance with all phases
   - Generate complete proof suite for all 9 phases
   - Produce final validation summary

---

## Discovery Phase: Current UI State

### Existing UI Infrastructure

**Location:** `/Users/dawsonblock/CODEX-1/ui/codex-dioxus/`

**Current Components:**
- Dioxus 0.7+ web frontend
- `runtime_client.rs` - Bridge to Rust runtime
- Existing answer display (likely basic text only)

**Current Data Flow:**
```
Rust Runtime
  ↓ EventEnvelope
Claim Store (17 claims, 16 validated)
Evidence Vault (96 entries, all hashed)
Answer Builder (18 answers, grounded)
  ↓ AnswerEnvelope
runtime_client.rs (via HTTP/JSON)
  ↓
Dioxus Components
  ↓
Browser UI
```

### What Phase 9 Must Discover

1. **Current AnswerEnvelope usage in UI**
   - Which fields are currently displayed?
   - Which fields are ignored/missing?
   - Architecture of runtime_client response

2. **Existing visualization components**
   - Chart libraries available
   - Timeline patterns in use
   - Data structure expectations

3. **Integration points for new data**
   - AnswerBasisItem rendering requirements
   - Confidence score visualization
   - Claim lifecycle state badges

---

## Phase 9 Implementation Plan

### Step 1: Analyze Current UI State (Discovery)

**Tasks:**
- [ ] Review runtime_client.rs to understand current API contract
- [ ] Identify which AnswerEnvelope fields are currently used
- [ ] Check UI component structure in Dioxus
- [ ] Locate visualization library declarations

**Deliverable:** `PHASE_9_DISCOVERY.md` with findings

### Step 2: Enhance Answer Display Component

**Tasks:**
- [ ] Create enhanced answer component showing:
  - Main answer text
  - Basis items table (claim_id, subject, predicate, object, confidence)
  - Evidence references (evidence_ids as links)
  - Policy warnings section
- [ ] Add styling for confidence levels (color gradient)
- [ ] Add lifecycle state badge (Active=green, Contradicted=orange, Superseded=gray)

**Deliverable:** Updated UI component with full AnswerEnvelope support

### Step 3: Add Timeline Visualization

**Tasks:**
- [ ] Create claim timeline component
- [ ] Show claims ordered by EventEnvelope timestamp
- [ ] Mark evidence collection points
- [ ] Highlight contradiction/supersession events
- [ ] Implement interactive zoom (all 15 cycles visible)

**Deliverable:** Timeline component with all phases represented

### Step 4: Add Pressure Dynamics Display

**Tasks:**
- [ ] Extract pressure metrics from replay_report.json
- [ ] Create action score trend chart
- [ ] Display policy blocking decisions
- [ ] Show cycle-by-cycle behavior (confident vs. deferring)

**Deliverable:** Dynamics dashboard showing pressure modulation

### Step 5: Implement Long-Horizon Trace Viewer

**Tasks:**
- [ ] Create per-cycle trace viewer
- [ ] Show claims active in each cycle
- [ ] Display evidence used in that cycle
- [ ] Indicate action taken and outcome
- [ ] Add navigation between cycles

**Deliverable:** Trace inspection interface for all 15 cycles

### Step 6: Generate Final UI Integration Report

**Tasks:**
- [ ] Run updated UI with all enhancements
- [ ] Extract metrics from integration:
  - Fields displayed per component
  - Visualization coverage
  - User interactions supported
- [ ] Create `ui_integration_report.json`
- [ ] Document compliance with Phases 5-8

**Deliverable:** `ui_integration_report.json` artifact

### Step 7: Final Proof Suite

**Tasks:**
- [ ] Update proof command to include UI report generation
- [ ] Run complete proof for all 9 phases
- [ ] Generate final `PROOF_SUMMARY.md`
- [ ] Verify:
  - All 229+ tests pass
  - All artifacts generated
  - overall_status = "pass"

**Deliverable:** Complete Phase 9 proof suite

---

## Technical Dependencies

### Required Changes

**Dioxus UI Changes:**
- Update components to handle new AnswerEnvelope fields
- Add timeline and pressure visualization
- Integrate chart library if not present

**Rust Runtime Changes (Minimal):**
- Ensure AnswerEnvelope fully serialized to JSON
- No EventEnvelope changes (completed Phase 5)
- No claim/evidence changes (established Phase 5-8)

**Python Reports:**
- Create `generate_ui_integration_report.py`
- Parse UI component outputs
- Generate final summary

**Git Commands:**
- Commit Phase 9 work incrementally
- Final Phase 9 commit with all proof artifacts

---

## Success Criteria

**End of Phase 9:**

- ✅ All AnswerEnvelope fields displayed in UI
- ✅ Timeline shows all claims with timestamps
- ✅ Pressure dynamics visible in dashboard
- ✅ Long-horizon trace viewer operational
- ✅ UI Integration report generated
- ✅ All 229+ tests passing
- ✅ Complete proof suite passes
- ✅ `overall_status: "pass"` confirmed
- ✅ Ready for Phases 10-14 production hardening

---

## Phase 9 Timeline

1. **Discovery** (~15 min) — Analyze current UI state
2. **Answer Display** (~30 min) — Implement enhanced component
3. **Timeline** (~30 min) — Add visualization
4. **Pressure Display** (~20 min) — Dashboard metrics
5. **Trace Viewer** (~25 min) — Per-cycle inspection
6. **Report Generation** (~15 min) — Create artifacts
7. **Final Proof** (~10 min) — Validate all systems
8. **Documentation** (~10 min) — PHASE_9_SUMMARY.md

**Total:** ~2.5 hours

---

## Integration Points

**Previous Phases Feed Into Phase 9:**

| Phase | Output | Phase 9 Usage |
|-------|--------|---------------|
| Phase 5 | EventEnvelope provenance | Timeline timestamps |
| Phase 6 | Evidence coverage | Evidence references in UI |
| Phase 7 | Retrieval policies | Pressure dynamics display |
| Phase 8 | AnswerBuilder fields | Basis items table + confidence |

---

## Risk Mitigation

**Risk:** UI component incompatibilities  
**Mitigation:** Minimal changes to existing components; new components encapsulated

**Risk:** Missing JSON serialization fields  
**Mitigation:** Verify AnswerEnvelope → JSON via cargo test before UI integration

**Risk:** Timeline complexity  
**Mitigation:** Use existing chart library; fallback to ASCII timeline if needed

---

## Proof Artifacts Target

**Phase 9 will generate:**

1. `ui_integration_report.json` — UI compliance metrics
2. `PHASE_9_SUMMARY.md` — Implementation summary
3. Final `proof_manifest.json` update
4. Complete `PROOF_SUMMARY.md` for phases 1-9

**Official proof command (Phase 9):**
```bash
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current && \
python3 scripts/generate_retrieval_policy_report.py artifacts/proof/current && \
python3 scripts/generate_answer_quality_report.py artifacts/proof/current && \
python3 scripts/generate_ui_integration_report.py artifacts/proof/current
```

---

## Next Action

**Begin Phase 9 Discovery:**
1. Examine current UI state (runtime_client.rs, Dioxus components)
2. Document findings in PHASE_9_DISCOVERY.md
3. Proceed with Step 1 of implementation plan

**Completion Trigger:** All 7 implementation steps complete + final proof validates
