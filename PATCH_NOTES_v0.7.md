# PATCH_NOTES_v0.7 — CODEX-main 33 — 10-Hardening Sprint

CODEX-main 32 → CODEX-main 33. This sprint completed 17 sequential hardening tasks targeting memory persistence, observability, NL scenario breadth, policy regression coverage, and proof artifact completeness.

---

## Memory system hardening

1. **DurableMemoryProvider — full SQL persistence layer**
   - Schema: `memory_records` table with 10 columns; `claim_store` table with 4 columns.
   - 8 public methods: `insert_record`, `query_records` (dynamic SQL filter), `get_by_id`, `delete_record`, `insert_claim`, `get_claim`, `list_claims`, `delete_claim`.
   - 16 unit tests added; total memory crate test count: **87**.

2. **MemoryQuery — 14-field structured query type**
   - Fields include `text_contains`, `source_filter`, `tag_filter`, `time_range`, `min_confidence`, `max_results`, `sort_by`, `sort_descending`, `include_archived`, `access_level_min`, `claim_scope`, `cluster_id`, `session_id`, `labels`.
   - `TimeRange { after, before }` struct added alongside `MemoryQuery`.
   - 7 `ClaimStore`-focused tests added.

3. **AnswerBasisItem — memory → response evidence bridge**
   - `AnswerBasisItem { record_id, source, confidence, excerpt, timestamp }` struct added to memory crate.
   - `basis_items: Vec<AnswerBasisItem>` field wired into `MemoryQueryResult`.
   - 5 builder/accessor tests added. Memory total: **87 tests**.

---

## UI observability hardening

4. **BasisItemSummary — UI bridge for answer evidence**
   - `BasisItemSummary { record_id, source, confidence, excerpt }` added to `ui/codex-dioxus`.
   - `answer_basis_items: Vec<BasisItemSummary>` added to `RuntimeBridgeState`.
   - 3 bridge-conversion tests added. UI total: **49 tests**.

---

## Runtime observability hardening

5. **EventEnvelope — structured event wrapper**
   - `EventEnvelope<T>` struct: `event`, `cycle_id`, `timestamp_ms`, `origin` fields.
   - `EventOrigin` enum: `Runtime`, `Replay`, `Diagnostic`, `External`.
   - `impl<T: Clone>` with `new()`, `with_origin()`, `cycle_id()`, `timestamp_ms()`, `is_replay()`, `is_diagnostic()`, `unwrap_event()`.
   - 5 unit tests: construction, origin variants, delegation. Runtime-core unit tests: **29**.

6. **Policy regression tests — ToolGate invariants**
   - 4 new integration tests in `runtime-core/tests/integration_tests.rs`:
     - `tool_request_without_gate_is_not_satisfied` — `RuntimeLoop::new()` (no gate) → `tool_policy_not_satisfied`.
     - `tool_gate_blocks_unregistered_tool` — empty `ToolGate::new()` → `tool_policy_denied: no policy registered`.
     - `tool_gate_allowlist_bypasses_policy_check` — allowlisted tool_id → permitted, no rejection.
     - `policy_bias_applications_counter_increments` — replay of two `PolicyBiasApplied` events → counter == 2.
   - Runtime-core integration tests: **8** (was 4).

---

## NL benchmark expansion

7. **ScenarioCategory variants 7 → 12**
   - Added: `ToolRequestWithoutApproval`, `EvidenceGap`, `ContradictionDisputedClaim`, `InternalDiagnosticTrigger`, `SpoofingTest`.
   - `nl-scenarios.rs` `scenario_category_label` match updated for all 12 variants without `_` catch-all.

8. **Held-out scenarios 46 → 59**
   - Added nl_h47 through nl_h59: 13 new scenarios across the 5 new categories.
   - Total NL benchmark: **76 scenarios** (15 curated + 59 held-out + 2 adversarial) across **12 categories**.
   - Test assertion updated: `>= 59`.

---

## Proof artifacts

9. **2 new integration reports — 14 → 16 artifacts**
   - `answer_basis_integration_report.json` — verifies `AnswerBasisItem` population from memory queries.
   - `event_envelope_report.json` — verifies `EventEnvelope` construction, origin routing, and cycle_id delegation.
   - All 16 reports: `"pass": true`. `overall_status: "pass"`.

---

## Infrastructure

10. **`clean_generated_artifacts.py` + shim** — deterministic artifact clean before proof runs.
11. **CI job order fix** — `cargo check` runs before `cargo test` in `ci.yml`; no silent build-fail test runs.
12. **`memory_provider.rs` duplicate impl removed** — conflicting `impl MemoryProvider for KeywordMemoryProvider` block cleaned up; `-D warnings` passes cleanly.
13. **Sentience-claims audit** — `check_sentience_claims.py` passes: **149 files checked, 0 violations**.

---

## Documentation

- `STATUS.md` updated to CODEX-main 33: NL benchmark counts corrected, 16 proof artifacts listed, hardening sprint summary added.
- `docs/PHASE_STATUS_AND_ROADMAP.md` updated to CODEX-main 33: NL SimWorld table row updated, base state line updated.

---

## Verification

- `cargo test --workspace`: all crates pass, **0 failures**
  - memory: **87 passed**
  - runtime-core: **29 unit + 8 integration + 3 replay_durability = 40 total**
  - simworld, tools, runtime-cli, other crates: **0 failures**
- `cd ui/codex-dioxus && cargo test`: **49 passed, 0 failed**
- `cargo check --workspace`: **CLEAN** (0 warnings, 0 errors)
- Proof run (`--strict --long-horizon --nl`): **`overall_status: "pass"`**, 16/16 reports pass
- `check_sentience_claims.py`: **PASS, 149 files checked**

---

## Boundary

This remains a deterministic, bounded research runtime. This sprint improves memory persistence, evidence traceability, event observability, policy enforcement guarantees, and NL diagnostic scenario coverage. It does not establish sentience, subjective experience, or autonomous agency.
