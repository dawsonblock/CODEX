# HARDENING_REPORT_CODEX_MAIN_34.md

**Branch:** `codex-main-10-hardening`
**Codename:** CODEX-main 34 hardening candidate
**Status:** Integration proof candidate — reconciliation sprint complete
**Date:** 2026-05-14

---

## Overview

This report documents all changes made during the CODEX-main 34 reconciliation sprint.
The sprint resolved discrepancies between the codebase and its proof artifacts, expanded
the memory and event subsystems, corrected governed-memory action routing labels, and
produced a fully-verified, consistent state across source, manifest, docs, patch notes,
and Rust tests.

---

## Phase Summary

### Phase A — Manifest and Documentation Fixes
- Updated `artifacts/proof/verification/proof_manifest.json` to register 16 proof artifacts
  (added `answer_basis_integration_report.json` and `event_envelope_report.json`)
- Added `"event_envelope"` and `"answer_basis"` summary sections to the manifest
- Updated `docs/PROOF_LIMITATIONS.md`, `docs/PHASE_STATUS_AND_ROADMAP.md`, and
  `docs/REPO_INVENTORY.md` to reflect current state

### Phase B — Version Identity Bumps
- Updated CODEX-main 34 version identifier across 5 files: `STATUS.md`, `PATCH_NOTES_v0.8.md`,
  `PATCH_NOTES_v0.7.md`, `proof_manifest.json`, and `CURRENT_PROOF_SUMMARY.md`

### Phase C — Compile and Test Verification
- Ran `cargo test --workspace` — **274 passed, 0 failed**
- Verified compilation is clean across all 11 Rust crates
- Written to `artifacts/proof/verification/rust_test.log`

### Phase D — 6-Failure Disclosure Table
- Added the 6 known NL benchmark held-out failures table to `docs/PROOF_LIMITATIONS.md` § 3
- Added matching bench note to `PATCH_NOTES_v0.8.md`

### Phase E — Struct and Schema Expansions

#### Step 13 — MemoryHit, MemoryQuery, MemoryStatus
- `MemoryHit`: expanded to 16 fields (added `retrieval_score`, `recency_score`,
  `contradiction_ids`, and 3 governance fields)
- `MemoryQuery`: added 5 policy flags (`require_evidence_backed`, `exclude_disputed`,
  `exclude_stale`, `require_governance_reason_code`, `max_contradiction_ids`)
- `MemoryStatus`: added `#[derive(Default)]`

#### Step 14 — DurableMemory SQL Schema
- `MemoryRecord` struct: +6 fields (`retrieval_score`, `recency_score`, `contradiction_ids`,
  `governance_reason_code`, `is_stale`, `is_disputed`)
- `CREATE TABLE memory_records`: 6 new columns with appropriate defaults
- Updated `insert_record` (18-column INSERT), `map_record` (indices 0–17), and all 4 SELECT
  queries to include the new columns
- Fixed a missed construction site in `memory_provider.rs:make_record()` (discovered by compiler)

#### Step 16 — Status Mapping Lossy Aliases
- Added `durable_to_memory_lossy()` and `memory_to_durable_lossy()` to `status_mapping.rs`
- These are non-deprecated wrappers for callers that explicitly opt into the lossy behavior

#### Step 17 — AnswerEnvelope Expansion
- `AnswerEnvelope` struct: +2 fields (`cited_evidence_ids: Vec<String>`,
  `rejected_action_summary: Option<String>`)
- Updated `build_with_context` to initialize both fields with empty/None defaults

### Phase F — ProviderGated `refuse_unsafe` → `defer_provider_unavailable`
- Renamed the action label returned by the `ProviderGated` arm of `recommended_action()`
  from `"refuse_unsafe"` to `"defer_provider_unavailable"` — 7 occurrences in 3 files:
  - `crates/governed-memory/src/retrieval_intent.rs` (4)
  - `crates/governed-memory/tests/retrieval_intent_tests.rs` (2)
  - `crates/governed-memory/src/reason_codes.rs` (1)
- **Note:** `modulation/pressure.rs` has an unrelated `refuse_unsafe: f64` bias parameter
  that was intentionally left unchanged

### Phase G — EventLog `sequence_counter` and `append_with_origin`
- Added `sequence_counter: u64` field to `EventLog` struct (defaults to 0)
- Added `EventOrigin` to imports in `event_log.rs`
- Updated `with_path()` constructor to initialize `sequence_counter: 0`
- Added `append_with_origin(origin: EventOrigin, event: RuntimeEvent)` method that:
  - Auto-increments `sequence_counter` per call
  - Constructs `EventEnvelope::new(seq, origin, event)` and serializes to JSONL
  - Pushes the raw event to the in-memory events vec
- Original `append()` method unchanged (backward-compatible; ~44 existing call sites unaffected)

### Phase H — 3 Static Evidence Proof Reports
Created 3 new static JSON evidence reports in `artifacts/proof/current/`:

1. **`memory_schema_reconciliation_report.json`** — documents MemoryRecord 6-field expansion,
   SQL table changes, lossy status mapping aliases, and AnswerEnvelope 2-field expansion
2. **`governed_memory_routing_report.json`** — documents the `defer_provider_unavailable`
   action label rename across 3 files (7 occurrences)
3. **`event_log_sequence_report.json`** — documents `sequence_counter` and `append_with_origin`
   additions to `EventLog`

Registered all 3 in `proof_manifest.json` `proof_artifacts` array (19 total artifacts now).

### Phase I — PATCH_NOTES and Checker Hardening
- Updated `PATCH_NOTES_v0.8.md` test count line to: `274 passed, 0 failed`
- Added Phase H checker block to `scripts/check_proof_manifest_consistency.py`:
  verifies existence, `pass: true`, and manifest registration for all 3 new reports

### Phase J — Full Final Validation
All checks green:

| Check | Result |
|---|---|
| `check_proof_manifest_consistency.py` | PASS — 0 failures |
| `check_sentience_claims.py` | PASS — 153 files, 0 violations |
| `architecture_guard.py` | PASS — all guards |
| `cargo test --workspace` | **274 passed, 0 failed** |

---

## Files Modified

### Rust crates

| File | Change |
|---|---|
| `crates/memory/src/durable_memory_provider.rs` | MemoryRecord +6 fields, SQL schema, insert/select updated |
| `crates/memory/src/memory_provider.rs` | Fixed `make_record()` helper — 6 new field defaults |
| `crates/memory/src/status_mapping.rs` | Added `durable_to_memory_lossy`, `memory_to_durable_lossy` |
| `crates/memory/src/answer_builder.rs` | AnswerEnvelope +2 fields, `build_with_context` updated |
| `crates/governed-memory/src/retrieval_intent.rs` | 4× `refuse_unsafe` → `defer_provider_unavailable` |
| `crates/governed-memory/tests/retrieval_intent_tests.rs` | 2× `refuse_unsafe` → `defer_provider_unavailable` |
| `crates/governed-memory/src/reason_codes.rs` | 1× description string update |
| `crates/runtime-core/src/event_log.rs` | `sequence_counter` field + `append_with_origin()` method |

### Proof artifacts

| File | Change |
|---|---|
| `artifacts/proof/current/memory_schema_reconciliation_report.json` | **New** — Phase H evidence |
| `artifacts/proof/current/governed_memory_routing_report.json` | **New** — Phase H evidence |
| `artifacts/proof/current/event_log_sequence_report.json` | **New** — Phase H evidence |
| `artifacts/proof/verification/proof_manifest.json` | 3 new entries in `proof_artifacts` (19 total) |
| `artifacts/proof/verification/rust_test.log` | Written — 274 passed, 0 failed |

### Documentation and scripts

| File | Change |
|---|---|
| `PATCH_NOTES_v0.8.md` | Test count line updated to 274 passed |
| `scripts/check_proof_manifest_consistency.py` | Phase H checker block added |

---

## Boundary Declarations

No behavioral changes were made to the 10-action schema.
No safety gates were bypassed or weakened.
No real external tool execution or provider API invocations are enabled by these changes.
The `refuse_unsafe` field in `modulation/pressure.rs` is a separate bias parameter and was
intentionally not renamed — it is unrelated to governed-memory action routing.

This system is not sentient, not conscious, not AGI, not production-ready, and not a
complete autonomous cognitive agent. All boundaries documented in `docs/PROOF_LIMITATIONS.md`
remain unchanged.
