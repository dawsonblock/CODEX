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

- `cargo test --workspace --all-targets`: **all crates pass, 0 failures**
  - runtime-core integration tests: **9 passed** (was 8; new regression test added)
- `cargo run -p runtime-cli -- proof --strict --long-horizon --nl`: **`overall_status: "pass"`**
- `cargo fmt --all`: **CLEAN** (formatting applied)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **CLEAN** for all sprint-touched crates; 3 pre-existing errors in `memory` crate (deprecated_semver ×2, too_many_arguments ×1) confirmed pre-existing via git stash test — not introduced by this sprint.
- `check_sentience_claims.py`: **PASS, 150 files checked, 0 violations**

---

## Boundary

This sprint improves claim observability and answer grounding fidelity. It does not establish sentience, subjective experience, or autonomous agency. The system remains a deterministic, bounded research runtime.
