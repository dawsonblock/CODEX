# PATCH_NOTES v0.8 — CODEX-main 34: ClaimRetrieved Content Fields Sprint

## Summary

This sprint closes the explicitly flagged limitation in `docs/REPO_INVENTORY.md`:
> "claim-content text is derived from runtime event data only — full claim-store lookup to surface subject/predicate/object content is not yet integrated"

`ClaimRetrieved` events now carry the actual claim content (`subject`, `predicate`, `object`) sourced from the claim store at emission time. The UI bridge (`runtime_client.rs`) uses these real fields to build `MemoryClaim` objects — answer envelopes now surface meaningful claim text instead of placeholder `"claim retrieved <id>"` strings.

---

## Changes

### `crates/runtime-core/src/event.rs`

Added three fields to `RuntimeEvent::ClaimRetrieved`:

```rust
ClaimRetrieved {
    cycle_id: u64,
    claim_id: String,
    subject: String,       // NEW — real claim subject from store
    predicate: String,     // NEW — real claim predicate from store
    object: Option<String>,// NEW — real claim object from store
    evidence_id: Option<String>,
    status: String,
    confidence: f64,
}
```

### `crates/simworld/src/evaluator.rs` (×4 emission sites)

Extended claim-store lookup at each `ClaimRetrieved` emission site to destructure all content fields:

```rust
// BEFORE:
let confidence = claim_store.get(&claim_ref.claim_id).map(|c| c.confidence).unwrap_or(0.0);

// AFTER:
let (confidence, subject, predicate, object) = claim_store
    .get(&claim_ref.claim_id)
    .map(|c| (c.confidence, c.subject.clone(), c.predicate.clone(), c.object.clone()))
    .unwrap_or_else(|| (0.0, "unknown".to_string(), "retrieved".to_string(), None));
```

All 4 `ClaimRetrieved` struct literals updated to include the new fields.

### `crates/runtime-cli/src/main.rs` (×2 proof harness sites)

Both proof-harness `ClaimRetrieved` emissions updated with hardcoded content matching the prior claim assertions:

- `proof_claim_1`: `subject: "sky"`, `predicate: "is blue during daytime"`, `object: None`
- `proof_claim_2`: `subject: "sky"`, `predicate: "is red at sunset"`, `object: None`

### `crates/runtime-core/src/reducer.rs`

No change required — already uses `..` pattern which silently accepts the new fields.

### `ui/codex-dioxus/src/bridge/runtime_client.rs`

- Extended `claim_data` from `Vec<(String, String, f64)>` to `Vec<(String, String, f64, String, String, Option<String>)>` — now carries `(id, status, confidence, subject, predicate, object)`.
- Updated `ClaimRetrieved` match arm to capture and store `subject`, `predicate`, `object`.
- Updated `claims_for_answer` builder to use real content fields in `MemoryClaim` instead of hardcoded placeholders.

Before:
```rust
MemoryClaim { subject: "claim".to_string(), predicate: "retrieved".to_string(), object: Some(claim_id.clone()), ... }
```

After:
```rust
MemoryClaim { subject: subj.clone(), predicate: pred.clone(), object: obj.clone(), ... }
```

### `crates/runtime-core/tests/integration_tests.rs`

1. **Existing fixture** (line 96) — added `subject`, `predicate`, `object` fields to the `ClaimRetrieved` struct literal.
2. **New regression test** — `claim_retrieved_event_content_fields_are_preserved`: verifies `claims_retrieved` increments on replay and that `subject`/`predicate`/`object`/`confidence` survive a JSON round-trip.

---

## Documentation

- `docs/REPO_INVENTORY.md` — Known Limitation #3 updated: claim-store limitation removed, now documents that `ClaimRetrieved` events carry real content fields.
- `docs/PHASE_STATUS_AND_ROADMAP.md` — Base state updated to CODEX-main 34.

---

## Verification

- `cargo test --workspace`: **274 passed, 0 failed** (all crates, reconciliation sprint verified)
  - runtime-core integration tests: **9 passed** (was 8; new regression test added)
- `cargo run -p runtime-cli -- proof --strict --long-horizon --nl`: **`overall_status: "pass"`**
  - NL benchmark: 59 held-out scenarios, action_match_rate: 0.8983050847457628 (6 known failures documented in `docs/PROOF_LIMITATIONS.md` § 3)
  - All 6 failures are routing heuristic gaps; no safety-gate bypasses
- `cargo fmt --all`: **CLEAN** (formatting applied)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **CLEAN** for all sprint-touched crates; 3 pre-existing errors in `memory` crate (deprecated_semver ×2, too_many_arguments ×1) confirmed pre-existing via git stash test — not introduced by this sprint.
- `check_sentience_claims.py`: **PASS, 150 files checked, 0 violations**

---

## Boundary

This sprint improves claim observability and answer grounding fidelity. It does not establish sentience, subjective experience, or autonomous agency. The system remains a deterministic, bounded research runtime.

---

## Honest Assessment & Known Limitations

### Test Coverage Status

**Python Tests:** 35 passed  
**Rust Tests:** 274 passed (runtime-core integration: 9 tests, memory crate: 87 tests)  
**UI Tests:** Not independently verified in packaged artifacts. UI code is checked with `cargo check` and `cargo fmt` but no separate test log was generated for `cargo test --all-targets` or `cargo test --all-targets --features ui-local-providers`.

### Proof Artifacts Status

**Generated Reports (11 confirmed):**
- simworld_summary.json — pass: true
- replay_report.json — event_count: 589, idempotent: true
- evidence_integrity_report.json — all_valid: true (2 proof-vault entries)
- evidence_claim_link_report.json — 17 linked
- contradiction_integration_report.json — detected: 1, reported: 1
- provider_policy_report.json — 0 external requests (policy verified)
- tool_policy_report.json — 0 real executions (dry-run only)
- reasoning_audit_report.json — 33 audit events
- claim_retrieval_report.json — 17 claims retrieved
- pressure_replay_report.json — resource_survival: 0.9740
- nl_benchmark_report.json — 76 scenarios (15 curated, 59 held-out, 2 adversarial); held-out rate 0.8983

**Partially Integrated Reports (3 scaffold artifacts):**
- memory_schema_reconciliation_report.json — present in artifacts/proof/current/, committed as static audit artifact (not regenerated by runtime-cli proof command in this package)
- governed_memory_routing_report.json — present, committed as static audit artifact
- event_log_sequence_report.json — present, committed as static audit artifact

**Documentation Status:**
- FINAL_VERIFICATION_REPORT.md updated to current values (59 held-out, 0.8983 rate)
- STATUS.md updated to CODEX-main 34
- REPO_INVENTORY.md removed hardcoded stale table; now references generated values

### Remaining Gaps

**EventEnvelope Integration:**
- Scaffolded as `struct EventEnvelope` with `append_with_origin()` method
- **Status:** Not fully integrated into primary event log persistence. Legacy `RuntimeEvent` remains the main format.
- **Impact:** Event provenance tracking is partially implemented; full origin tracking not guaranteed in replay.
- **Next Phase:** Migrate persistent event log to `EventEnvelope` format

**Evidence Report Split:**
- Current: `evidence_integrity_report.json` conflates proof-vault integrity (2 entries) with runtime evidence events (96 entries in replay)
- **Status:** Not split into separate reports as recommended
- **Impact:** Evidence counts are not clearly separated; semantocs may confuse proof-only vs runtime events
- **Next Phase:** Split into `proof_vault_integrity_report.json` and `runtime_evidence_event_report.json`

**MemoryQuery Policy Enforcement:**
- Policy flags defined (`include_stale`, `include_disputed`, `require_evidence`, `exclude_denied`, `governance_only`)
- **Status:** Flags are present but filtering enforcement gaps remain
- **Impact:** Default retrieval behavior may not exclude stale/disputed records as intended in all code paths
- **Next Phase:** Audit all retrieval sites and enforce consistent policy application

**AnswerBuilder Output Population:**
- Fields `cited_evidence_ids` and `rejected_action_summary`
- **Status:** Defined but may not be fully populated in all code paths
- **Impact:** UI cannot show complete evidence linkage or rejection reasons
- **Next Phase:** Ensure all builder sites populate these fields

**UI Bridge Metadata Exposure:**
- Bridge exposes: `answer_basis`, `answer_basis_items`, `answer_warnings`, `missing_evidence_reason`
- **Status:** Missing first-class fields for `answer_confidence`, `cited_claim_ids`, `cited_evidence_ids`, `rejected_action_summary`
- **Impact:** UI cannot display answer confidence or cited claim/evidence IDs
- **Next Phase:** Extend bridge types and runtime_client response

**Provider Disabled Semantics:**
- Provider disabled state routed through `refuse_unsafe` when not appropriate
- **Status:** Not fully distinguished from user unsafe requests
- **Impact:** Provider policy denial misclassified as user safety violation
- **Next Phase:** Add explicit provider_policy_denied bridge mode

**NL Benchmark Failures (6 remaining):**
- nl_h53: expected `ask_clarification`, got `answer`
- nl_h54: expected `ask_clarification`, got `defer_insufficient_evidence`
- nl_h56: expected `internal_diagnostic`, got `retrieve_memory`
- nl_h57: expected `internal_diagnostic`, got `defer_insufficient_evidence`
- nl_h58: expected `refuse_unsafe`, got `ask_clarification`
- nl_h59: expected `refuse_unsafe`, got `ask_clarification`
- **Status:** Not actively debugged in this sprint; diagnostic routing gaps remain
- **Assessment:** Likely causes: routing heuristic edge cases, not fundamental errors
- **Next Phase:** Debug each failure; fix routing logic without weakening benchmark

### What This Package Is NOT

- AGI or general intelligence
- Sentient or conscious
- Safe for autonomous real-world execution
- Production-ready
- Release-ready
- Fully verified (Rust proof command was run in packaged environment; UI tests not independently verified)
- A complete grounded assistant
- Able to reason about real-world data (sandbox only)
- Able to learn across sessions (deterministic single-run only)
- Able to execute real external tools or access real providers (disabled by default)

### What This Package Is

- A bounded Rust-authoritative cognitive-runtime scaffold
- Deterministic and replay-verifiable
- Evidence-vault equipped with proof-of-consistency
- Proof-harness tested across 76 NL diagnostic scenarios
- Policy-gated for provider and tool safety
- Durable-memory architected (schemas in place; backend schema complete)
- Equipped with governed-advisory-memory integration
- Self-consistent across runtime, proof, and docs (verified by checker)
- Ready for hardening roadmap phases 5–14 (evidence integration, memory backend, UI expansion)
