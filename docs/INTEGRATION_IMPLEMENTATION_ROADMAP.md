# CODEX-main 32 Integration Implementation Roadmap

> **Historical Note:** This document outlines the integration roadmap from CODEX-main 32. Current version is CODEX-main 34. This document remains available as a reference for the implementation sequence and completion status of phases 2–11.

<!-- markdownlint-disable -->


**Status:** Integration contract defined, implementation roadmap prepared  
**Phase 0 (Verification):** ✓ Complete  
**Phase 1 (Contract):** ✓ Complete  
**Next:** Phase 2–11 implementation (requires Rust compilation environment)


## Integration Phases Summary

### Phase 2: Evidence → Claim Linking
**Target:** Create EvidenceClaimAdapter  
**Location:** `crates/memory/src/evidence_claim_adapter.rs` (new) or extend `crates/memory/src/claim_store.rs`  
**Key methods:**

**Tests needed:**

**Proof artifact:** Count claims_with_evidence_links in replay_report.json


### Phase 3: Add Events for Connected Path
**Target:** Formalize/extend runtime events  
**Location:** `crates/runtime-core/src/events.rs` or similar

**Events to add/formalize:**

**Reducer updates:**

**Tests needed:**


### Phase 4: Claim Retrieval in Action Selection
**Target:** Create ClaimRetrievalAdapter  
**Location:** `crates/cognition/src/claim_retrieval_adapter.rs` (new) or integrate into Critic

**Key behavior:**

**Integration point:** Modify `crates/runtime-core/src/runtime_loop.rs` to:

**Tests needed:**


### Phase 5: Contradiction Check Integration
**Target:** Extend ContradictionEngine integration  
**Location:** Extend `crates/contradiction/src/lib.rs` and integrate into RuntimeLoop

**Integration:**

**Tests needed:**


### Phase 6: Operational Pressure Full Replay Reconstruction
**Target:** Enhance PressureModulator for stateful replay  
**Location:** `crates/modulation/src/pressure.rs`

**Enhancement:**

**Proof artifact:** `pressure_replay_report.json` showing final state reconstruction

**Tests needed:**


### Phase 7: Reasoning Audit with Evidence/Claim/Pressure References
**Target:** Extend ReasoningAudit structure  
**Location:** `crates/runtime-core/src/reasoning_audit.rs`

**Extend AuditReport:**

**Integration:** Populate these fields during cycle based on:

**Tests needed:**


### Phase 8: Tool Dry-Run / Approval Lifecycle
**Target:** Formalize ToolPolicy scaffold  
**Location:** `crates/tools/src/lib.rs` and `crates/runtime-core/src/runtime_loop.rs`

**Formalization:**

**Integration:** Modify tool candidate scoring in Critic to:

**Tests needed:**


### Phase 9: Bigger Held-Out NL Benchmark
**Target:** Expand NL scenarios  
**Location:** `crates/simworld/src/nl_scenarios.rs`

**Expansion:**
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


### Phase 10: Proof Artifact Expansion
**Target:** Generate integration reports via official proof command  
**Location:** `crates/runtime-cli/src/proof.rs` and output generation

**New artifacts generated:**

**Update existing:**

**Documentation updates:**


### Phase 11: Final Verification
**Commands:**

**Report:** Update `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md` with integration results


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

**Likely create:**

**Document updates (no code):**

### Testing Strategy

1. Unit tests for each adapter/integration piece
2. Integration tests for claim→contradiction→pressure→action flow
3. Replay tests for deterministic reconstruction
4. Proof command validates full path
5. Python guards remain passing

### Proof Artifact Success Criteria

Each new artifact must show:


## Current Blockers / Unknowns

1. **Claim object/value field** — Does MemoryClaim support object/value (subject-predicate-object)? If not, plan subject-predicate only with TODO for full SPO.
2. **Contradiction engine capabilities** — Does it already support object-conflict detection or only predicate-conflict?
3. **Pressure fields** — Are all 9 fields (uncertainty, contradiction, safety, resource, social_risk, tool_risk, evidence_gap, urgency, coherence) correctly named in codebase?
4. **Tool policy current state** — Is tool policy already integrated into Critic, or does it need wiring?
5. **Audit structure extensibility** — Can ReasoningAudit be extended without breaking existing serialization?

**Resolution:** Before Phase 2 implementation, inspect each crate's current state and resolve these.


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


## Estimated Effort

**Per-phase breakdown (with Rust environment):**

**With limited environment (local verification only):**


**Next step:** Review this roadmap, resolve unknowns, begin Phase 2 implementation.
<!-- markdownlint-enable -->
