# CODEX-main 32 Integration Kickoff Status

**Date:** 2026-05-09  
**Status:** Integration planning complete, implementation ready  
**Commit:** 0cd3eb4

---

## What Was Done

### Phase 0: Baseline Verification ✓
- Python: 35 tests pass, all 6 guards pass
- Rust: cargo 1.95.0, rustc 1.95.0, 139 tests pass
- Cargo fmt/clippy clean
- Official proof command ready

### Phase 1: Integration Contract ✓
- **docs/EVIDENCE_GROUNDED_RUNTIME_INTEGRATION.md** — End-to-end path defined
  - Runtime path explicitly specified (Observation → EvidenceStored → ClaimCreated → ... → ProofArtifactVerifies)
  - Existing modules identified (evidence, claims, contradiction, pressure, runtime-core, audit, simworld, tools)
  - Events required documented (ClaimCreated, ClaimRetrieved, ContradictionChecked, PressureUpdated, ReasoningAuditGenerated)
  - Data flow between modules detailed
  - 6 new proof artifacts specified
  - Failure modes addressed
  - Limitations clearly bounded (not claiming sentience, AGI, broad reasoning, or tool safety)

### Phase 1.5: Implementation Roadmap ✓
- **docs/INTEGRATION_IMPLEMENTATION_ROADMAP.md** — Detailed specifications for phases 2–11
  - Phase 2: Evidence → claim linking (create EvidenceClaimAdapter)
  - Phase 3: Add replayable events (ClaimCreated, ClaimRetrieved, ContradictionChecked, etc.)
  - Phase 4: Claim retrieval in action selection (create ClaimRetrievalAdapter)
  - Phase 5: Contradiction check integration (extend ContradictionEngine, raise pressure)
  - Phase 6: Operational pressure full replay reconstruction (stateful pressure state)
  - Phase 7: Reasoning audit with evidence/claim/pressure references (extend AuditReport)
  - Phase 8: Tool dry-run / approval lifecycle (formalize ToolPolicy, no real execution)
  - Phase 9: Bigger held-out NL benchmark (expand to 28+ scenarios)
  - Phase 10: Proof artifact expansion (generate 6 new reports)
  - Phase 11: Final verification (Python + Rust + proof)

---

## Current State: CODEX-main 32

**Authority:** Rust-authoritative (global-workspace-runtime-rs/)  
**Tests:** 139 Rust + 35 Python  
**Scaffolds:** All 8 present (evidence, claims, contradiction, pressure, runtime-core, audit, simworld, tools)  
**Proof:** Strict proof passes (--strict --long-horizon --nl)

---

## Integration Target

**Path to prove:**
```
Observation
  → EvidenceStored (SHA-256 hash, real content)
  → ClaimCreated (evidence-backed, evidence_id linked)
  → ClaimRetrieved (during observation processing)
  → ContradictionChecked (on active claims)
  → PressureUpdated (contradiction/evidence_gap fields)
  → ActionScored (claims + pressure influence scores)
  → ActionSelected (from 10-action vocab)
  → ReasoningAuditGenerated (evidence_ids, claim_ids, contradiction_ids, pressures)
  → RuntimeEventEmitted (all events logged)
  → ReplayReconstructs (full state path deterministically)
  → ProofArtifactVerifies (6 new integration reports confirm)
```

---

## Key Integration Documents

1. **docs/EVIDENCE_GROUNDED_RUNTIME_INTEGRATION.md** (9.6 KB)
   - Purpose, path, modules, events, data flow, artifacts, failure modes, success criteria

2. **docs/INTEGRATION_IMPLEMENTATION_ROADMAP.md** (11.8 KB)
   - Detailed phase-by-phase specifications
   - File locations, methods to implement, tests needed
   - Proof artifacts per phase
   - Critical blockers, unknowns to resolve
   - Estimated effort: 12–18 days

---

## Implementation Readiness

**Ready to implement:**
- Phase 2 (evidence→claim linking) — straightforward adapter
- Phase 3 (events) — event enum + reducer counters
- Phase 4 (claim retrieval) — query existing claims, return results
- Phase 5 (contradiction integration) — extend existing engine
- Phase 6 (pressure replay) — add full state vector to events
- Phase 7 (audit references) — extend AuditReport structure
- Phase 8 (tool lifecycle) — formalize existing policy
- Phase 9 (NL expansion) — add scenarios to nl_scenarios.rs
- Phase 10 (proof artifacts) — generate reports from reducer state
- Phase 11 (verification) — run existing verification suite

**Blockers to resolve before Phase 2:**
1. Does MemoryClaim support subject-predicate-object or only subject-predicate?
2. Does ContradictionEngine already support object-conflict or only predicate-conflict?
3. Are all 9 pressure fields correctly named in codebase?
4. Is tool policy already integrated into Critic?
5. Can ReasoningAudit be extended without breaking serialization?

---

## Boundaries Explicitly Preserved

✓ No new major modules (adapters only)  
✓ 10-action schema unchanged  
✓ Proof checks not weakened  
✓ Python not made authoritative  
✓ No sentience/consciousness/AGI/emotion claims  
✓ No real external tool execution enabled  
✓ No broad natural-language reasoning claims  
✓ No full evidence-grounded cognition claims  
✓ Deterministic replay preserved  
✓ Proof command remains official truth  

---

## What This Integration Proves

After completion:
- Evidence entries can create evidence-backed claims
- Claims affect action selection measurably
- Contradictions update claim status and operational pressure
- Reasoning audits reference evidence/claims/contradictions/pressure
- Full path is deterministically replayable
- Existing scaffolds work together

---

## What This Integration Does NOT Prove

- Sentience, consciousness, or AGI
- Broad natural-language reasoning
- Full evidence-grounded cognition across arbitrary data
- Safe autonomous external tool execution
- Production-readiness
- Semantic contradiction truth-resolution
- Emotional reasoning

---

## Next Steps

1. **Review blockers** — Inspect crates to resolve unknowns (1 day)
2. **Create tracking** — Set up PR/branch structure (1 day)
3. **Begin Phase 2** — Implement EvidenceClaimAdapter (2 days)
4. **Iterate phases** — Complete phases 3–11 in order (10–16 days)
5. **Verify & document** — Run full suite, update proof artifacts (1–2 days)

**Total estimated:** 15–21 days with dedicated Rust environment

---

## References

- **Integration contract:** docs/EVIDENCE_GROUNDED_RUNTIME_INTEGRATION.md
- **Implementation roadmap:** docs/INTEGRATION_IMPLEMENTATION_ROADMAP.md
- **Rust workspace:** global-workspace-runtime-rs/
- **Current proof:** artifacts/proof/current/
- **Verification receipts:** artifacts/proof/verification/

---

**Status:** CODEX-main 32 is prepared and planned for evidence-grounded runtime integration.

The scaffolds exist. The path is defined. The proof framework is ready.

Next work: Connect the systems and prove they work together.

**Ready to proceed to Phase 2 implementation.**
