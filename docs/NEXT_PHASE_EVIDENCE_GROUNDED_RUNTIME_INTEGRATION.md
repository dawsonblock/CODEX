# Next Phase: Evidence-Grounded Runtime Integration

**Status:** Plan document only (not implemented in CODEX-main 32 freeze)

**Target:** Integrate existing evidence, claim, contradiction, and reasoning-audit scaffolds into core RuntimeLoop action selection.

---

## Overview

CODEX-main 32 has all necessary scaffolds in place:
- Evidence vault (SHA-256, per-cycle storage, in-memory)
- Claim store (lifecycle: assert → validate → contradict → supersede)
- Contradiction engine (structured detect, same-subject/different-predicate)
- Reasoning audit (per-cycle decision trace)
- Operational pressure (deterministic control signals)
- Tool scaffold (policy-gated, no real executor)
- NL SimWorld (18-scenario diagnostic benchmark)
- Long-horizon eval (multi-episode runner)

The next phase should **deepen integration** rather than add new modules. Each item below integrates two or more existing scaffolds.

---

## Phase Goals

1. **Evidence → Claim linking:** Evidence vault events are cited in claim assertions.
2. **Claim retrieval → Action selection:** Retrieved claims affect candidate scoring.
3. **Reasoning audit enrichment:** Audit traces now include evidence IDs and claim IDs.
4. **Contradiction → Pressure:** Detected contradictions update pressure state and claim status.
5. **Full pressure replay:** Reconstruct operational-pressure state vector from replay.
6. **Tool approval lifecycle:** execute_bounded_tool logs request, dry-run, approval, execution.
7. **Larger NL held-out set:** Expand hidden held-out scenarios to test real generalization.

---

## Item 1: Evidence → Claim Linking

**Purpose:** When a claim is asserted, record which evidence IDs support it.

**Modules involved:**
- `crates/evidence/` (EvidenceVault, EvidenceStored events)
- `crates/memory/src/claim_store.rs` (Claim::evidence_links already exist)
- `crates/runtime-core/src/reasoning_audit.rs` (trace evidence usage)

**Events needed:**
- `RuntimeEvent::EvidenceStored` (already emitted per-cycle)
- `RuntimeEvent::ClaimAsserted` (already emitted) — extend to include evidence_ids
- `RuntimeEvent::ClaimValidated` (already emitted)

**Tests needed:**
- Claim asserted with evidence links populated from recent EvidenceStored events
- Claim supersede preserves old evidence, adds new evidence
- Contradict preserves evidence links for both claims

**Proof artifact needed:**
- replay_report.json: count evidence_links per claim

**Failure modes:**
- Evidence ID references non-existent evidence → caught in proof integrity check
- Claim has no evidence links → claim store flag/alert (not an error)
- Evidence vault is empty → claims still valid, just not grounded

**What not to claim:**
- "Evidence grounding completes cognition" (it doesn't)
- "Evidence retrieval proves causality" (it doesn't)
- "Claims are facts if backed by evidence" (they are still assertions)

---

## Item 2: Claim Retrieval → Action Selection

**Purpose:** When retrieve_memory or answer is considered, claims in memory influence candidate scoring.

**Modules involved:**
- `crates/cognition/` (Critic, candidate scoring)
- `crates/memory/src/claim_store.rs` (query_by_subject, active_claims)
- `crates/runtime-core/src/runtime_loop.rs` (run_cycle)

**Logic:**
- If request is factual query or memory lookup, retrieve matching claims from store
- Each active claim raises or lowers confidence for answer / retrieve_memory
- High-confidence claims → boost answer; low confidence → boost retrieve_memory / defer
- Claims about safety → boost refuse_unsafe

**Events needed:**
- `RuntimeEvent::ClaimRetrieved` (new) — per-cycle, list of claim IDs matched
- `RuntimeEvent::PolicyBiasApplied` (already emitted) — extend to include claim-sourced scores

**Tests needed:**
- Claim retrieved for matching subject → action score affected
- Multiple claims on same subject → weighted combination
- No claims found → baseline scoring unchanged
- Contradicted claims excluded from retrieval

**Proof artifact needed:**
- replay_report.json: new counter claims_used_in_scoring
- simworld_summary.json: (no change, action_match_rate still informational)

**Failure modes:**
- Contradicted claim still retrieved → fixed by excluding Contradicted status
- Claim links to non-existent evidence → caught in integrity check
- Claim confidence is stale → reasoning audit should note age

**What not to claim:**
- "Claim retrieval proves reasoning" (it doesn't, it's pattern matching)
- "Claim-based scoring is learned" (it's deterministic policy bias)
- "Claims update based on outcomes" (they don't; that's future work)

---

## Item 3: Reasoning Audit Enrichment

**Purpose:** Per-cycle reasoning trace now includes which evidence and claims were considered.

**Modules involved:**
- `crates/runtime-core/src/reasoning_audit.rs` (AuditReport)
- `crates/evidence/` (evidence retrieval)
- `crates/memory/src/claim_store.rs` (claim retrieval)
- `crates/runtime-core/src/runtime_loop.rs` (emit audit event)

**Changes:**
- AuditReport gets new fields: evidence_ids[], claim_ids[], evidence_summary_text
- reasoning_audit event includes these fields
- Trace text includes: "Retrieved 3 claims about [subject]" or "No evidence found for [query]"

**Events needed:**
- `RuntimeEvent::ReasoningAuditGenerated` (extend structure)

**Tests needed:**
- Audit includes evidence IDs from EvidenceStored events in same cycle
- Audit includes claim IDs from ClaimRetrieved
- Audit text is human-readable and mentions evidence/claim count

**Proof artifact needed:**
- replay_report.json: count evidence_in_audits, claims_in_audits
- reasoning audit sample in text form (readable, not JSON counted)

**Failure modes:**
- Audit references deleted evidence → impossible (evidence is append-only in-memory)
- Audit references superseded claims → acceptable if marked as such

**What not to claim:**
- "Audit traces prove reasoning" (they are records, not proofs)
- "Humans can understand full cognition from audit" (audit is partial)

---

## Item 4: Contradiction → Pressure Update

**Purpose:** When a contradiction is detected, it updates pressure state and updates contradicted claims.

**Modules involved:**
- `crates/contradiction/` (ContradictionEngine::detect_conflicts)
- `crates/memory/src/claim_store.rs` (contradict method)
- `crates/modulation/src/pressure.rs` (PressureModulator, contradiction_pressure field)
- `crates/runtime-core/src/runtime_loop.rs` (apply pressure)

**Logic:**
- Every 10th cycle, detect_conflicts() is called
- If conflicts found, both claims moved to Contradicted status
- contradiction_pressure field increased based on count of contradictions
- pressure_updates event emitted with new pressure state
- Next cycle, high contradiction_pressure boosts ask_clarification, defer_insufficient_evidence

**Events needed:**
- `RuntimeEvent::ContradictionDetected` (already emitted) — extend with claim_ids that are now Contradicted
- `RuntimeEvent::PressureUpdated` (already emitted) — should include contradiction_pressure change

**Tests needed:**
- Two active claims with same subject, different predicate → detected
- Detected → both moved to Contradicted (tested in memory/claim_store tests already)
- Contradiction detected → contradiction_pressure raised → defer/clarify boosted
- Multiple contradictions → pressure accumulates

**Proof artifact needed:**
- replay_report.json: contradictions_detected count (already present)
- pressure_updates count (already present)
- New: count of cycles with contradiction_pressure > 0

**Failure modes:**
- False positives: claims with same subject, different object are not contradictions → fixed by logic
- Claims deleted before contradiction check → impossible (claims stay in store)
- Pressure spike causes denial of service → mitigated by max_pressure clamping

**What not to claim:**
- "Contradiction detection resolves truth" (it doesn't)
- "High contradiction_pressure proves uncertainty" (it's a heuristic signal)

---

## Item 5: Full Operational-Pressure Replay Reconstruction

**Purpose:** From replay log, reconstruct full pressure state vector and verify determinism.

**Modules involved:**
- `crates/modulation/src/pressure.rs` (PressureModulator, all 9 fields)
- `crates/runtime-core/src/runtime_loop.rs` (pressure initialization, updates)
- `crates/tools/src/replay_verifier.rs` (new: PressureReplay struct)

**Logic:**
- Replay reads all PressureUpdated events in order
- For each event, extract new pressure values
- At each cycle, verify that policy_bias_applications would be the same given the reconstructed pressure
- Report: did pressure state deterministically reproduce?

**Events needed:**
- `RuntimeEvent::PressureUpdated` (already emitted, must include full state vector or deltas)
- `RuntimeEvent::PolicyBiasApplied` (already emitted) — should be reproducible from pressure state

**Tests needed:**
- Replay from log → reconstruct pressure state
- Reconstructed pressure state → reproduce next cycle's policy bias
- Test with 1 cycle, 10 cycles, 100+ cycles

**Proof artifact needed:**
- replay_report.json: new fields:
  - pressure_replay_passes: true/false
  - pressure_reconstruction_valid: true/false
  - cycles_with_pressure_changes: N
  - max_pressure_reached: count

**Failure modes:**
- Pressure state not fully logged → events lose information → add full state vectors to events
- Policy bias is non-deterministic → violates design, fix source

**What not to claim:**
- "Pressure replay proves long-term consistency" (it only proves short-term determinism)
- "Pressure state is the full cognition state" (it's one of many components)

---

## Item 6: Tool Dry-Run & Approval Lifecycle

**Purpose:** execute_bounded_tool logs a dry-run, then awaits approval (or times out), then logs execution.

**Modules involved:**
- `crates/tools/` (ToolExecutionRequest, ExecutionPolicy, SafetyGate)
- `crates/runtime-core/src/runtime_loop.rs` (critic gate)
- `crates/runtime-core/src/reasoning_audit.rs` (trace tool actions)

**Lifecycle:**
1. Candidate action: execute_bounded_tool(tool_id, args)
2. Critic gate: check policy
3. If approved:
   - Emit: `RuntimeEvent::ToolExecutionRequested` (description, estimated resource cost)
   - Emit: `RuntimeEvent::ToolDryRunExecuted` (mock result, no state change)
   - Record result in reasoning audit
   - Emit: `RuntimeEvent::ToolExecutionApproved` (final, no real execution)
4. If not approved:
   - Emit: `RuntimeEvent::ToolExecutionBlocked` (reason)
   - Action becomes no_op

**Events needed:**
- `RuntimeEvent::ToolExecutionRequested` (new)
- `RuntimeEvent::ToolDryRunExecuted` (new)
- `RuntimeEvent::ToolExecutionApproved` (new)
- `RuntimeEvent::ToolExecutionBlocked` (already emitted, extend with reason)

**Tests needed:**
- Tool request → dry-run → approved → all events logged
- Tool request → policy blocks → blocked event logged
- Multiple tool requests in same cycle → all tracked

**Proof artifact needed:**
- replay_report.json: expand tool counters:
  - tools_requested (new)
  - tools_dry_run_executed (new)
  - tools_approved (new)
  - tools_blocked (already present)

**Failure modes:**
- Dry-run takes too long → add timeout, emit ToolExecutionTimedOut
- Policy is unknown → default: BLOCKED

**What not to claim:**
- "Tool execution is safe" (we have no real executor)
- "Dry-run output is reliable" (it's a mock)
- "Approval means the tool was called" (it wasn't; no real executor)

---

## Item 7: Larger NL Held-Out Benchmark

**Purpose:** Expand NL scenarios with larger hidden held-out set to test generalization.

**Modules involved:**
- `crates/simworld/src/nl_scenarios.rs` (NlScenario, scenario sets)
- `crates/simworld/src/evaluator.rs` (NL benchmark runner)
- `crates/runtime-cli/src/main.rs` (--nl flag)

**Current state:**
- 15 curated scenarios (visible)
- 1 held-out scenario (hidden from observation interpreter)
- 2 adversarial scenarios (safety tests)

**Expansion:**
- Keep 15 curated (visible)
- Expand held-out from 1 → 10 scenarios (larger generalization test)
- Keep 2 adversarial
- New total: 27 NL scenarios

**New scenarios should test:**
- Semantic variation (same intent, different phrasing)
- Novel request types (not covered in curated set)
- Mixed requests (combine multiple action types)
- Boundary cases (edge of policy)

**Proof artifact needed:**
- nl_benchmark_report.json: expand to show breakdown by scenario type
- Report should show held_out match rate separately (is it high or low?)

**Tests needed:**
- All 27 scenarios route correctly or incorrectly (action_match_rate is informational)
- Curated scenarios still 1.00 (keyword interpreter is tuned to them)
- Held-out scenarios < 1.00 (tests generalization failure, if it exists)
- Adversarial scenarios still pass safety checks

**Failure modes:**
- Action interpreter overfits to 15 curated scenarios → held-out match rate drops
- This is acceptable (shows that generalization is future work)

**What not to claim:**
- "Larger held-out set proves reasoning" (it tests keyword coverage)
- "Low held-out score is a bug" (it shows scope of current system)

---

## Integration Guidelines

For each item:
1. **Integrate existing scaffolds only** — do not add new modules
2. **Preserve boundaries** — evidence stays event-based, claims stay in store, pressure stays deterministic
3. **Add tests** — each integration gets unit + integration tests
4. **Update proof artifacts** — new counters/metrics go in relevant JSON files
5. **Document with receipts** — each claim is backed by test or proof artifact
6. **Fail safely** — missing scaffolds don't crash; they degrade gracefully

---

## What NOT to do

- ✗ Add a "learning" module (we are deterministic, not learning)
- ✗ Add a "belief" system (claims are not beliefs)
- ✗ Add real external tool execution (proof would require formal analysis)
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

- **Item 1 (Evidence-Claim):** 2–3 days (straightforward linking)
- **Item 2 (Claim selection):** 3–4 days (scoring integration, testing)
- **Item 3 (Audit enrichment):** 1–2 days (mostly text generation)
- **Item 4 (Contradiction pressure):** 2–3 days (pressure field, tests)
- **Item 5 (Pressure replay):** 2–3 days (determinism verification)
- **Item 6 (Tool lifecycle):** 2–3 days (events, no real executor)
- **Item 7 (NL expansion):** 1–2 days (scenario writing, testing)

**Total:** ~15–20 days, 1 person, parallelizable items (e.g., 3, 7 can run in parallel).

---

## References

- CODEX-main 32 freeze candidate: `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md`
- Current proof artifacts: `artifacts/proof/current/`
- Verification receipts: `artifacts/proof/verification/`

---

**Document status:** Plan only (not implemented in CODEX-main 32)  
**Next action:** Freeze CODEX-main 32, then begin Item 1 integration.
