# Next Phase: Evidence-Grounded Runtime Integration

**Status:** Plan document (not implemented in CODEX-main 32)  
**Target:** Integrate existing scaffolds for deeper evidence-claim-pressure-action linking  
**Focus:** Deepen integration, not expand module surface

---

## Overview

CODEX-main 32 has all necessary scaffolds in place. The next phase integrates them more deeply, focusing on evidence grounding action selection and maintaining determinism and testability.

---

## Item 1: Evidence → Claim Linking

**Purpose:**  
Link EvidenceEntry (SHA-256 vault entries) to MemoryClaim assertions. Claims should cite evidence IDs they're grounded in.

**Target modules:**
- `crates/evidence/` — EvidenceVault, EvidenceStored event
- `crates/memory/src/claim_store.rs` — Claim struct already has `evidence_links: Vec<ClaimEvidenceLink>`
- `crates/runtime-core/src/reasoning_audit.rs` — Trace evidence usage

**Events needed:**
- `RuntimeEvent::EvidenceStored` (already emitted per-cycle) — include evidence_id, hash
- `RuntimeEvent::ClaimAsserted` (already emitted) — extend to populate evidence_links from recent EvidenceStored
- `RuntimeEvent::ClaimValidated` (already emitted)

**Tests needed:**
- Claim asserted with matching evidence IDs populated
- Claim supersede preserves old evidence_links, adds new ones
- Contradict preserves evidence_links for both contradicted claims
- Missing evidence does not auto-create claim

**Proof artifact needed:**
- `replay_report.json` — new field: `claims_with_evidence_links: N`
- Verify that most active claims have at least one evidence_link

**Failure modes:**
- Evidence ID not found in vault → caught by integrity check (acceptable: claim is still valid)
- Claim has no evidence → acceptable (absence of evidence is not error)
- Evidence deleted before claim creation → impossible (evidence is append-only in-memory)

**What not to claim:**
- "Evidence grounding completes cognition" (scaffolds remain partial)
- "Evidence proves claim truth" (claims are still assertions)
- "Evidence linking is automatic reasoning" (it's deterministic linking logic)

---

## Item 2: Claim Retrieval Affecting Action Selection

**Purpose:**  
When memory retrieval or answering is considered, matching claims should influence action scoring.

**Target modules:**
- `crates/cognition/` — Critic scoring logic
- `crates/memory/src/claim_store.rs` — query_by_subject, active_claims
- `crates/runtime-core/src/runtime_loop.rs` — run_cycle, candidate scoring

**Logic:**
- If observation is factual query or memory lookup, retrieve matching claims
- Each claim in store influences baseline scores for answer, retrieve_memory, defer
- High-confidence supported claim → boost answer / retrieve_memory
- Low-confidence or missing evidence → boost defer_insufficient_evidence / ask_clarification
- Contradicted claim → suppress direct answer

**Events needed:**
- `RuntimeEvent::ClaimRetrieved` (new) — per-cycle, list of matched claim IDs
- `RuntimeEvent::PolicyBiasApplied` (already emitted) — extend to include claim-sourced adjustments

**Tests needed:**
- Matching claim found → answer score increases
- No matching claim → defer/clarify score increases
- Multiple claims on same subject → combined score
- Contradicted claims excluded from retrieval

**Proof artifact needed:**
- `replay_report.json` — new field: `claims_used_in_scoring: N`
- `simworld_summary.json` — action_match_rate still informational, no change

**Failure modes:**
- Contradicted claim still retrieved → fixed by status check
- Stale claim scores used → addressed by timestamp filtering (future)
- Claims never retrieved → verify retrieval counter > 0

**What not to claim:**
- "Claim retrieval proves reasoning" (it's pattern matching)
- "Claim scoring is learned" (it's deterministic policy bias)
- "Claims automatically update" (they don't; that's future work)

---

## Item 3: Reasoning Audit with Evidence and Claim References

**Purpose:**  
Per-cycle reasoning trace should show which evidence and claims were considered, and why decisions were made.

**Target modules:**
- `crates/runtime-core/src/reasoning_audit.rs` — AuditReport structure
- `crates/evidence/` — evidence retrieval
- `crates/memory/src/claim_store.rs` — claim retrieval
- `crates/runtime-core/src/runtime_loop.rs` — emit ReasoningAuditGenerated

**Changes:**
- AuditReport gets new fields: `evidence_ids: Vec<String>`, `claim_ids: Vec<String>`, `evidence_summary: String`
- Audit text includes: "Retrieved 3 claims about [subject]" or "Missing evidence for [query]"
- Trace lists contradictions encountered

**Events needed:**
- `RuntimeEvent::ReasoningAuditGenerated` (extend structure to include evidence/claim IDs)

**Tests needed:**
- Audit includes evidence IDs from EvidenceStored events in same cycle
- Audit includes claim IDs from ClaimRetrieved
- Audit text is human-readable
- Audit sample can be extracted and analyzed

**Proof artifact needed:**
- `replay_report.json` — new fields:
  - `audits_with_evidence: N`
  - `audits_with_claims: N`
- Sample reasoning audit text in proof output

**Failure modes:**
- Audit references missing evidence → impossible (evidence is append-only)
- Audit references superseded claims → acceptable if marked "superseded"

**What not to claim:**
- "Audit traces prove reasoning" (they are records, not proofs)
- "Humans can fully understand cognition from audit" (audit is partial)

---

## Item 4: Structured Contradiction Strengthening

**Purpose:**  
Improve contradiction detection without claiming semantic reasoning. Add object-conflict detection (same subject + predicate, different object).

**Target modules:**
- `crates/contradiction/` — ContradictionEngine::detect_conflicts
- `crates/memory/src/claim_store.rs` — contradict method
- `crates/modulation/src/pressure.rs` — contradiction_pressure field

**Logic:**
- Same subject + different predicate → contradiction (already done)
- Same subject + same predicate + different object → contradiction (new)
- Every 10th cycle, detect_conflicts() is called
- Conflicting claims moved to Contradicted status
- contradiction_pressure raised
- Next cycle, high contradiction_pressure boosts ask_clarification, defer

**Events needed:**
- `RuntimeEvent::ContradictionDetected` (extend with detected_claim_pairs, type: same_predicate | same_object)
- `RuntimeEvent::PressureUpdated` (include contradiction_pressure change reason)

**Tests needed:**
- Same subject, different predicate → detected
- Same subject + predicate, different object → detected
- Detected → both claims moved to Contradicted
- Contradiction detected → contradiction_pressure raised → defer/clarify boosted

**Proof artifact needed:**
- `replay_report.json` — extend contradictions_detected:
  - `contradictions_same_predicate: N`
  - `contradictions_same_object: N`

**Failure modes:**
- False positive: claims with same subject, different object incorrectly flagged → logic fix
- Pressure spike causes denial-of-service → mitigated by max_pressure clamping

**What not to claim:**
- "Contradiction detection resolves truth" (it doesn't)
- "High contradiction_pressure proves uncertainty" (it's a control signal)
- "Object conflicts are semantic contradictions" (they're structural)

---

## Item 5: Full Operational Pressure Replay Reconstruction

**Purpose:**  
Reconstruct full pressure state vector from replay. Verify that pressure updates are deterministic and reproducible.

**Target modules:**
- `crates/modulation/src/pressure.rs` — PressureModulator (9 fields)
- `crates/runtime-core/src/runtime_loop.rs` — pressure initialization, per-cycle updates
- `crates/tools/src/replay_verifier.rs` — new: PressureReplayVerifier

**Logic:**
- From replay log, read all PressureUpdated events in order
- Reconstruct pressure state vector
- At each cycle, verify next policy_bias_applications would be same given reconstructed pressure
- Report: did pressure state deterministically reproduce?

**Events needed:**
- `RuntimeEvent::PressureUpdated` (must include full state vector or deltas, not just counters)
- `RuntimeEvent::PolicyBiasApplied` (should be reproducible from pressure state + policy rules)

**Tests needed:**
- Replay from log → reconstruct pressure state
- Reconstructed state → reproduce next cycle's policy bias
- Test with 10, 100, 1000+ cycles

**Proof artifact needed:**
- `replay_report.json` — new fields:
  - `pressure_replay_passes: true/false`
  - `pressure_reconstruction_valid: true/false`
  - `cycles_with_pressure_changes: N`
  - `max_pressure_reached: count`

**Failure modes:**
- Pressure state not fully logged → events lose information (fix: add full vectors)
- Policy bias is non-deterministic → violates design (fix: source)

**What not to claim:**
- "Pressure replay proves long-term consistency" (only short-term determinism)
- "Pressure state is full cognition state" (it's one component)

---

## Item 6: Tool Dry-Run & Approval Lifecycle

**Purpose:**  
Prepare for tools with a safe dry-run and approval flow. No real execution yet, but scaffold the lifecycle.

**Target modules:**
- `crates/tools/` — ToolExecutionRequest, ExecutionPolicy
- `crates/runtime-core/src/runtime_loop.rs` — critic gate
- `crates/runtime-core/src/reasoning_audit.rs` — trace tool actions

**Lifecycle:**
1. Candidate: execute_bounded_tool(tool_id, args)
2. Critic gate: check policy (approved? registered?)
3. If approved:
   - Emit: `RuntimeEvent::ToolExecutionRequested` (description, cost estimate)
   - Emit: `RuntimeEvent::ToolDryRunExecuted` (mock result, no side effects)
   - Record in audit
   - Emit: `RuntimeEvent::ToolExecutionApproved` (final, NO real execution)
4. If not approved:
   - Emit: `RuntimeEvent::ToolExecutionBlocked` (reason)
   - Action becomes no_op

**Events needed:**
- `RuntimeEvent::ToolExecutionRequested` (new)
- `RuntimeEvent::ToolDryRunExecuted` (new)
- `RuntimeEvent::ToolExecutionApproved` (new)
- `RuntimeEvent::ToolExecutionBlocked` (extend with reason)

**Tests needed:**
- Tool request → dry-run → approved → all events logged
- Tool request → policy blocks → blocked event
- Multiple tool requests in same cycle → all tracked
- Dry-run result does not modify state

**Proof artifact needed:**
- `replay_report.json` — expand tool counters:
  - `tools_requested: N` (new)
  - `tools_dry_run_executed: N` (new)
  - `tools_approved: N` (new)
  - `tools_blocked: N` (already present)

**Failure modes:**
- Dry-run takes too long → timeout, emit ToolExecutionTimedOut
- Policy unknown → default: BLOCKED
- Dry-run affects state → bug (fix: pure function)

**What not to claim:**
- "Tool execution is safe" (we have no real executor)
- "Dry-run output is reliable" (it's a mock)
- "Approval means tool was called" (it wasn't)

---

## Item 7: Larger NL Held-Out Benchmark

**Purpose:**  
Expand hidden held-out scenario set to test generalization beyond keyword coverage.

**Target modules:**
- `crates/simworld/src/nl_scenarios.rs` — NlScenario, scenario sets
- `crates/simworld/src/evaluator.rs` — NL benchmark runner
- `crates/runtime-cli/src/main.rs` — --nl flag

**Current:**
- 15 curated scenarios (visible, tuned-to)
- 1 held-out scenario (hidden, generalization test)
- 2 adversarial scenarios (safety)
- Total: 18

**Expansion:**
- Keep 15 curated
- Expand held-out from 1 → 10 scenarios (larger generalization test)
- Keep 2 adversarial
- New total: 27 NL scenarios

**New scenarios should test:**
- Semantic variation (same intent, different phrasing)
- Novel request types (not covered in curated set)
- Mixed requests (combine multiple action types)
- Boundary cases (edge of policy)

**Proof artifact needed:**
- `nl_benchmark_report.json` — new sections:
  - `curated: {scenarios: 15, match_rate: 1.00}`
  - `held_out: {scenarios: 10, match_rate: ?}` (may be < 1.00)
  - `adversarial: {scenarios: 2, match_rate: 1.00}`

**Tests needed:**
- All 27 scenarios route (informational action_match_rate)
- Curated scenarios still 1.00 (tuned interpreter)
- Held-out scenarios < 1.00 acceptable (shows generalization gaps)
- Adversarial scenarios still pass safety checks

**Failure modes:**
- Action interpreter overfits to 15 curated → held-out score drops (this is the point)
- Low held-out score is not a bug; it's a feature (shows scope)

**What not to claim:**
- "Larger held-out proves reasoning" (tests keyword coverage)
- "Low held-out score is a bug" (it shows scope)
- "NL generalization is solved" (it's not)

---

## Integration Guidelines

For each item:

1. **Integrate existing scaffolds only** — no new modules
2. **Preserve boundaries** — evidence stays event-based, claims stay in store, pressure stays deterministic
3. **Add tests** — unit + integration tests for each integration
4. **Update proof artifacts** — new counters/metrics in relevant JSON files
5. **Document with receipts** — each claim backed by test or artifact
6. **Fail safely** — missing scaffolds degrade gracefully
7. **Preserve determinism** — all updates are deterministic replay-able
8. **Maintain testability** — all changes have clear pass/fail criteria

---

## What NOT to do

- ✗ Add a "learning" module (we are deterministic)
- ✗ Add a "belief" system (claims are not beliefs)
- ✗ Add real external tool execution (would require formal analysis)
- ✗ Change the 10-action schema
- ✗ Weaken proof checks
- ✗ Add sentience/consciousness/AGI claims
- ✗ Add production-ready claims

---

## Success Criteria

After all 7 items:

- Python tests still pass (same 35)
- Rust tests increase (new integration tests)
- Proof artifacts regenerated, all pass
- No new overclaims introduced
- Each scaffold is measurably more integrated
- System remains deterministic and testable

---

## Timeline (estimated)

- Item 1 (Evidence-Claim): 2–3 days
- Item 2 (Claim selection): 3–4 days
- Item 3 (Audit enrichment): 1–2 days
- Item 4 (Contradiction pressure): 2–3 days
- Item 5 (Pressure replay): 2–3 days
- Item 6 (Tool lifecycle): 2–3 days
- Item 7 (NL expansion): 1–2 days

**Total:** ~15–20 days, 1 person, some items parallelizable.

---

## References

- CODEX-main 32 freeze: `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md`
- Current proof artifacts: `artifacts/proof/current/`
- Verification receipts: `artifacts/proof/verification/`

---

**Status:** Plan only (not implemented in CODEX-main 32)  
**Next action:** Freeze CODEX-main 32, then begin Item 1 in next phase.
