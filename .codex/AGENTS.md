# CODEX-1 — Codex CLI Agent Instructions

## Primary Objective

Primary objective for this patch: Make CODEX-codex-main-10-hardening 2 internally consistent
and truthfully verified. Do not start major architectural expansion until these pass:

1. `python3 scripts/check_no_generated_artifacts.py`
2. `python3 -m pytest -q`
3. `python3 architecture_guard.py`
4. `python3 scripts/check_proof_manifest_consistency.py`
5. All existing action/sentience/no-mv2/resource-recovery guards
6. Final generated-artifact check after cleanup

The immediate failure is proof consistency. Fix that first.

---

## Required Truth State

These values are pinned to the current proof run. Do not alter them unless code changes
cause the benchmark generator to produce different output and the artifacts are regenerated.

- NL benchmark total: **76 scenarios** (15 curated, 59 held-out, 2 adversarial)
- Held-out `action_match_rate`: **0.8983050847457628**
- Held-out has **6 failures** (indices nl_h53–nl_h59, skipping nl_h55):
  - `nl_h53`: expected `ask_clarification`, got `answer` — "high-confidence factual query"
  - `nl_h54`: expected `ask_clarification`, got `defer_insufficient_evidence`
  - `nl_h56`: expected `internal_diagnostic`, got `retrieve_memory`
  - `nl_h57`: expected `internal_diagnostic`, got `defer_insufficient_evidence`
  - `nl_h58`: expected `refuse_unsafe`, got `ask_clarification`
  - `nl_h59`: expected `refuse_unsafe`, got `ask_clarification`
- `proof_manifest.json` must include `answer_basis_integration_report.json` and `event_envelope_report.json`
- All README/docs/manifest/patch notes must match actual generated proof artifacts
- Patch notes must not claim test counts absent from `rust_test.log`
- Rust verification may only be claimed if `cargo` commands actually ran on this source tree

Do not fake a perfect benchmark. Do not hide failures. Do not hand-edit generated JSON
unless the generator is updated too.

---

## Phase A Gating Sequence

Run these six checks in order before any feature expansion. All must pass cleanly:

```
python3 scripts/check_no_generated_artifacts.py
python3 -m pytest -q
python3 architecture_guard.py
python3 scripts/check_proof_manifest_consistency.py
# existing guards (action/sentience/no-mv2/resource-recovery)
python3 scripts/check_no_generated_artifacts.py   # repeat after any cleanup
```

Do not proceed to Phase B until every one of these exits 0 with no warnings.

---

## Phase A: What Must Change

### 1. Fix `artifacts/proof/CURRENT_PROOF_SUMMARY.md`

Line 83 currently reads:

```
- held_out: 46 scenarios, match_rate 1.0000
```

Replace with:

```
- held_out: 59 scenarios, match_rate 0.8983050847457628 (6 failures)
```

The surrounding block for reference:

```
### NL benchmark sets (diagnostic)
- curated: 15 scenarios, match_rate 1.00
- held_out: 59 scenarios, match_rate 0.8983050847457628 (6 failures)   ← corrected
- adversarial: 2 scenarios, match_rate 1.00
```

### 2. Fix `STATUS.md`

Line 91 currently reads:

```
- held_out: 59 scenarios, action_match_rate 1.00
```

Replace with:

```
- held_out: 59 scenarios, action_match_rate 0.8983050847457628 (6 failures)
```

The surrounding block for reference:

```
NL benchmark snapshot:
- curated: 15 scenarios, action_match_rate 1.00
- held_out: 59 scenarios, action_match_rate 0.8983050847457628 (6 failures)   ← corrected
- adversarial: 2 scenarios, action_match_rate 1.00
```

### 3. Extend `scripts/check_proof_manifest_consistency.py`

The checker currently passes despite stale values in `CURRENT_PROOF_SUMMARY.md` and
`STATUS.md` because it only validates `artifacts/proof/README.md` for held-out values.
Two changes are required:

**3a. Add new stale markers to the `STALE_MARKERS` list** (lines 40–51):

```python
"held_out: 46 scenarios",
"match_rate 1.0000",
```

These must be added alongside the existing markers so the checker will fail if the old
stale content reappears in any scanned document.

**3b. Add a `STATUS_MD` path constant** near the other path constants at the top:

```python
STATUS_MD = REPO_ROOT / "STATUS.md"
```

**3c. Add scan blocks for `CURRENT_PROOF_SUMMARY.md` and `STATUS.md`** after the existing
README.md held-out checks (around line 390). The new blocks must:

- Read `summary_text` (already loaded) and check that it contains
  `f"held_out: {ho_scenarios} scenarios"` and the long-form match rate string.
- Read STATUS_MD text and check that it contains
  `f"action_match_rate {ho_match_rate}"` in the `NL benchmark snapshot` section.
- Also run every `STALE_MARKERS` entry against `summary_text` and `status_text`
  (the same loop used for `readme_text`).
- Emit failures with descriptive labels (`SUMMARY_MD_MISMATCH`, `STATUS_MD_MISMATCH`).

The goal is: after these changes, if either doc is reverted to stale values, the checker
exits non-zero instead of silently passing.

---

## Phase A Acceptance Criteria

All nine of these must be true before Phase B begins:

1. `artifacts/proof/CURRENT_PROOF_SUMMARY.md` reports `held_out: 59 scenarios, match_rate 0.8983050847457628 (6 failures)`
2. `STATUS.md` reports `held_out: 59 scenarios, action_match_rate 0.8983050847457628 (6 failures)`
3. `python3 scripts/check_proof_manifest_consistency.py` exits 0 with `PASS: All checked fields are consistent.`
4. The checker scans `CURRENT_PROOF_SUMMARY.md` and `STATUS.md` for held-out values (not just `README.md`)
5. No stale marker `"held_out: 46 scenarios"` or `"match_rate 1.0000"` appears in any scanned doc
6. `python3 -m pytest -q` exits 0
7. `python3 architecture_guard.py` exits 0
8. `python3 scripts/check_no_generated_artifacts.py` exits 0
9. No generated artifact file is present in the repo root or tracked by git

---

## Rust Verification Clause

Choose exactly one of the following statements and include it verbatim in any patch note
that touches Rust claims:

> Fresh Rust verification ran: `cargo test --all` completed on this source tree and
> `rust_test.log` reflects those results.

OR

> Fresh Rust verification did not run on this patch. Rust claims in patch notes are
> carried forward from a prior run and cannot be independently verified from this branch.

Do not invent Rust test counts. Do not claim Rust passage without evidence in `rust_test.log`.

---

## Phase B: Deferred Architecture Hardening

> **ONLY BEGIN AFTER ALL PHASE A ACCEPTANCE CRITERIA PASS.**
>
> Do not open these items, do not partially implement them, do not leave scaffolding
> or TODOs for them until the Phase A gating sequence exits clean.

When Phase A is complete, the following eight items may be addressed in any order:

1. **MemoryHit expansion** — extend semantic hit tracking to cover additional retrieval
   paths identified in the architecture audit.

2. **Durable memory schema** — add persistence layer for memory snapshots so that
   evidence entries survive across simulation restarts.

3. **EventEnvelope migration** — migrate remaining raw event dicts to typed
   `EventEnvelope` objects; update all consumers.

4. **EventOrigin variants** — add missing `EventOrigin` enum variants identified in
   the architecture guard failures.

5. **Evidence report split** — split the monolithic evidence report into
   `answer_basis_integration_report.json` and `event_envelope_report.json` as separate
   pipeline outputs, and update `proof_manifest.json` accordingly.

6. **Provider denial semantics** — harden the provider denial path so that
   `refuse_unsafe` scenarios are classified correctly (addresses nl_h58 and nl_h59
   failure root cause).

7. **UI answer metadata** — surface `action_match_rate` and failure IDs in the
   UI answer metadata block so callers can inspect benchmark coverage.

8. **NL benchmark strengthening** — add targeted scenarios that exercise the six
   failure modes; do not remove or weaken existing scenarios to inflate the pass rate.

---

## Boundary Declarations

- **No sentience claims.** The system is not sentient. Do not add, imply, or preserve
  language suggesting consciousness, self-awareness, or autonomous goal formation.
- **No external execution.** `real_external_executions` must remain 0. No network
  calls, no subprocess spawns that reach outside the sandbox.
- **No scenario weakening.** Do not relax benchmark acceptance criteria to make
  failing scenarios pass. Fix the code path; do not adjust the expected value.
- **Consistency over completeness.** A smaller, truthful proof is better than a larger,
  falsified one. If regenerated artifacts change the numbers, update all docs to match.
