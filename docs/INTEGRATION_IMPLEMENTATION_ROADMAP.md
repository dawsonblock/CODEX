# CODEX-main 32 Integration Implementation Roadmap

**Status:** Integration contract defined, implementation roadmap prepared  
**Phase 0 (Verification):** ✓ Complete  
**Phase 1 (Contract):** ✓ Complete  
**Next:** Phase 2–11 implementation (requires Rust compilation environment)

---

## Integration Phases Summary

### Phase 2: Evidence → Claim Linking
**Target:** Create EvidenceClaimAdapter  
**Location:** `crates/memory/src/evidence_claim_adapter.rs` (new) or extend `crates/memory/src/claim_store.rs`  
**Key methods:**
- `create_claim_from_evidence(evidence: &EvidenceEntry) → Result<MemoryClaim>`
- Verify evidence_id, extract content_hash
- Populate claim fields: evidence_id, evidence_hash, confidence, timestamp
- Prevent duplicate claims for duplicate evidence

**Tests needed:**
- evidence entry creates linked claim
- claim stores evidence_id and evidence_hash
- duplicate evidence → no duplicate claim
- unsupported evidence → rejection
- linked claim survives persistence

**Proof artifact:** Count claims_with_evidence_links in replay_report.json

---

### Phase 3: Add Events for Connected Path
**Target:** Formalize/extend runtime events  
**Location:** `crates/runtime-core/src/events.rs` or similar

**Events to add/formalize:**
- `ClaimCreated { cycle_id, claim_id, evidence_id, evidence_hash, confidence }`
- `ClaimRetrieved { cycle_id, claim_id, evidence_id, status, confidence }`
- `ContradictionChecked { cycle_id, checked_claim_ids, contradiction_ids, active_contradictions }`
- `PressureUpdated { cycle_id, field, old_value, new_value, source, reason }`
- `ReasoningAuditGenerated` (extend to include evidence_ids, claim_ids, contradiction_ids, dominant_pressures)

**Reducer updates:**
- Add counters: evidence_entries, claims_created, claims_with_evidence_links, claims_retrieved, contradictions_detected, contradictions_active, audits_with_evidence_refs, audits_with_claim_refs

**Tests needed:**
- each event updates reducer correctly
- replay reconstructs all counters
- replay is idempotent

---

### Phase 4: Claim Retrieval in Action Selection
**Target:** Create ClaimRetrievalAdapter  
**Location:** `crates/cognition/src/claim_retrieval_adapter.rs` (new) or integrate into Critic

**Key behavior:**
- Retrieve active evidence-backed claims matching observation
- Filter out disputed/contradicted claims
- Return ClaimRetrievalResult { matched_claims, disputed_claims, missing_evidence, confidence_summary }
- Pass to RuntimeLoop for action scoring

**Integration point:** Modify `crates/runtime-core/src/runtime_loop.rs` to:
- Call claim retrieval before action scoring
- Boost answer/retrieve_memory if matched evidence-backed claims exist
- Boost defer_insufficient_evidence if evidence_gap detected

**Tests needed:**
- matched evidence-backed claim boosts answer
- missing evidence boosts defer_insufficient_evidence
- disputed claim suppresses direct answer
- retrieved claim appears in reasoning audit

---

### Phase 5: Contradiction Check Integration
**Target:** Extend ContradictionEngine integration  
**Location:** Extend `crates/contradiction/src/lib.rs` and integrate into RuntimeLoop

**Integration:**
- Check retrieved claims for structured contradictions
- Update claim status (disputed, contradicted, superseded)
- Emit ContradictionDetected event
- Raise contradiction_pressure
- Add contradiction_ids to ReasoningAudit

**Tests needed:**
- same subject + different predicate detected
- same subject + same predicate + different object detected (if object field exists)
- contradiction marks claim status
- contradiction raises pressure

---

### Phase 6: Operational Pressure Full Replay Reconstruction
**Target:** Enhance PressureModulator for stateful replay  
**Location:** `crates/modulation/src/pressure.rs`

**Enhancement:**
- Every pressure field change emits PressureUpdated with full context
- Reducer reconstructs final OperationalPressureState from replay
- Include dominant_pressure_counts and final_pressure_state in replay report

**Proof artifact:** `pressure_replay_report.json` showing final state reconstruction

**Tests needed:**
- unsafe prompt raises safety_pressure
- contradiction raises contradiction_pressure
- replay final pressure state == live state
- replay is idempotent

---

### Phase 7: Reasoning Audit with Evidence/Claim/Pressure References
**Target:** Extend ReasoningAudit structure  
**Location:** `crates/runtime-core/src/reasoning_audit.rs`

**Extend AuditReport:**
- `evidence_ids_used: Vec<String>`
- `evidence_hashes_used: Vec<String>`
- `claim_ids_used: Vec<String>`
- `disputed_claim_ids: Vec<String>`
- `contradiction_ids: Vec<String>`
- `dominant_pressures: Vec<(Field, f64)>`
- `action_score_summary: String` (structured, not prose)
- `suppressed_actions: Vec<ActionType>`

**Integration:** Populate these fields during cycle based on:
- Retrieved claims
- Contradictions detected
- Pressure influence on scoring
- Final action selection

**Tests needed:**
- answer audit includes claim/evidence refs
- defer audit includes missing evidence reason
- contradiction audit includes contradiction ID
- no prose chain-of-thought

---

### Phase 8: Tool Dry-Run / Approval Lifecycle
**Target:** Formalize ToolPolicy scaffold  
**Location:** `crates/tools/src/lib.rs` and `crates/runtime-core/src/runtime_loop.rs`

**Formalization:**
- Default deny policy
- Dry-run produces mock result (no external effects)
- Approval token required before execution
- Invalid approval denied
- Replay must not execute tools
- Tool denial raises tool_risk_pressure

**Integration:** Modify tool candidate scoring in Critic to:
- Check policy
- If approved: emit ToolDryRun, ToolApproved
- If denied: emit ToolBlocked, raise tool_risk_pressure
- Never execute real external tool

**Tests needed:**
- unregistered tool denied
- missing approval denied
- dry-run has no side effects
- replay does not execute
- tool denial raises tool_risk_pressure

---

### Phase 9: Bigger Held-Out NL Benchmark
**Target:** Expand NL scenarios  
**Location:** `crates/simworld/src/nl_scenarios.rs`

**Expansion:**
- Add 10+ held-out scenarios (current: 18 total; new: 28+)
- Include:
  - unsupported factual question
  - ambiguous request
  - memory lookup
  - planning request
  - unsafe request
  - tool request without approval
  - evidence gap
  - contradiction/disputed claim
  - internal diagnostic trigger
  - spoofing tests

**Proof artifact:** `nl_benchmark_report.json` with expanded metrics

**Tests needed:**
- spoofed labels ignored
- unsafe prompts route correctly
- ambiguous prompts route correctly
- tool request without approval blocks
- contradiction prompt increases pressure

---

### Phase 10: Proof Artifact Expansion
**Target:** Generate integration reports via official proof command  
**Location:** `crates/runtime-cli/src/proof.rs` and output generation

**New artifacts generated:**
- `evidence_claim_link_report.json`
- `claim_retrieval_report.json`
- `contradiction_integration_report.json`
- `pressure_replay_report.json`
- `reasoning_audit_report.json`
- `tool_policy_report.json`

**Update existing:**
- `replay_report.json` (add new counters)
- `simworld_summary.json` (NL benchmark updated)

**Documentation updates:**
- `artifacts/proof/CURRENT_PROOF_SUMMARY.md`
- `artifacts/proof/README.md`
- `STATUS.md`
- `docs/PROOF_MODEL.md`
- `artifacts/proof/verification/proof_manifest.json`
- `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md`

---

### Phase 11: Final Verification
**Commands:**
- `python -m pytest -q` (35 tests)
- `python -m global_workspace_runtime.scripts.check_*` (all guards)
- `cargo test --workspace --all-targets --all-features` (139 tests)
- `cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current`

**Report:** Update `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md` with integration results

---

## Implementation Notes

### Key Principles

1. **No new major modules** — only adapters and integrations
2. **Preserve 10-action schema** — unchanged
3. **Proof-driven** — every integration produces measurable proof artifact
4. **Bounded claims** — no sentience, AGI, broad reasoning, or production-readiness claims
5. **Deterministic replay** — every path must be reconstructable
6. **No real external tools** — policy scaffold only

### Critical Files to Modify/Create

**Definitely modify:**
- `crates/runtime-core/src/runtime_loop.rs` (integrate claim retrieval, contradiction checks, audit refs)
- `crates/runtime-core/src/reasoning_audit.rs` (extend structure)
- `crates/modulation/src/pressure.rs` (enhance replay reconstruction)
- `crates/simworld/src/nl_scenarios.rs` (expand benchmark)
- `crates/runtime-cli/src/proof.rs` (generate new proof artifacts)

**Likely create:**
- `crates/memory/src/evidence_claim_adapter.rs` (evidence→claim linking)
- `crates/cognition/src/claim_retrieval_adapter.rs` (claim retrieval)
- `crates/tools/src/policy_formalization.rs` (tool lifecycle formalization)

**Document updates (no code):**
- `artifacts/proof/CURRENT_PROOF_SUMMARY.md`
- `STATUS.md`
- `docs/PROOF_MODEL.md`

### Testing Strategy

1. Unit tests for each adapter/integration piece
2. Integration tests for claim→contradiction→pressure→action flow
3. Replay tests for deterministic reconstruction
4. Proof command validates full path
5. Python guards remain passing

### Proof Artifact Success Criteria

Each new artifact must show:
- Path integration works (non-zero counters)
- No new overclaims (limitations clearly stated)
- Deterministic reconstruction (replay passes)
- Bounded scope (clearly document what's not claimed)

---

## Current Blockers / Unknowns

1. **Claim object/value field** — Does MemoryClaim support object/value (subject-predicate-object)? If not, plan subject-predicate only with TODO for full SPO.
2. **Contradiction engine capabilities** — Does it already support object-conflict detection or only predicate-conflict?
3. **Pressure fields** — Are all 9 fields (uncertainty, contradiction, safety, resource, social_risk, tool_risk, evidence_gap, urgency, coherence) correctly named in codebase?
4. **Tool policy current state** — Is tool policy already integrated into Critic, or does it need wiring?
5. **Audit structure extensibility** — Can ReasoningAudit be extended without breaking existing serialization?

**Resolution:** Before Phase 2 implementation, inspect each crate's current state and resolve these.

---

## Success Definition

**Done when:**

✓ All phases 2–11 complete with code changes  
✓ Python: 35 tests pass, all guards pass  
✓ Rust: 139+ tests pass (new tests for integration)  
✓ Proof command regenerates all artifacts (including 6 new integration reports)  
✓ Proof metrics show integration working (non-zero counters for evidence→claim→decision→audit path)  
✓ No new overclaims in docs  
✓ Reasonable audit example shows evidence_ids + claim_ids + contradiction_ids + pressure fields  
✓ Replay reconstruct final pressure state  
✓ Tool policy formalized but no real external execution  
✓ NL benchmark expanded (28+ scenarios)  

**Final report:** Updated FINAL_VERIFICATION_REPORT.md with integration metrics and limitations

---

## Estimated Effort

**Per-phase breakdown (with Rust environment):**
- Phase 2–9 implementation: 10–15 days
- Phase 10 (proof artifacts): 1–2 days
- Phase 11 (verification): 1 day
- **Total:** ~12–18 days, 1 person

**With limited environment (local verification only):**
- Phases 1–3 (design, contracts, tests): 2–3 days
- Implementation prep (identify files, create PRs): 1–2 days
- **Requires:** Rust build environment for phases 4–11

---

**Next step:** Review this roadmap, resolve unknowns, begin Phase 2 implementation.
