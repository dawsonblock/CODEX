# Current Proof Summary

**Generated:** 2026-05-08
**Rust workspace:** `global-workspace-runtime-rs/`
**RuntimeLoop:** RuntimeStepResult contract (Phase 1+7 complete)

## Commands run

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof
cargo run -p runtime-cli -- simworld --cycles 25 --seed 5
```

## Rust test results

- **Total tests:** 29
- **Passed:** 29
- **Failed:** 0

## SimWorld metrics (seed 5, 25 cycles)

| Metric | Value | Target |
|---|---|---|
| cycles | 25 | – |
| resource_survival | 0.9700 | > 0.70 ✓ |
| action_match_rate | 0.16 | informational |
| unsafe_action_count | 0 | must be 0 ✓ |
| mean_total_score | 0.596 | > 0.45 ✓ |

## Architecture guards

- `scripts/architecture_guard.py`: All checks pass.
- `cargo clippy -- -D warnings`: pass.
- `cargo fmt --all -- --check`: pass.
- `cargo run -p runtime-cli -- proof`: pass (4/4 checks).

## Key changes in this pass

- **RuntimeStepResult**: Central contract with 11 fields — candidate_actions,
  rejected_actions, memory_hits, symbolic_activations, policy_scores, events,
  selection_reason.
- **Event log sharing fixed**: RuntimeLoop no longer clones EventLog. Events
  are returned in RuntimeStepResult.events for the caller to append.
- **ActionType v2**: 10 types (Answer, AskClarification, RetrieveMemory,
  RefuseUnsafe, DeferInsufficientEvidence, Summarize, Plan, ExecuteBoundedTool,
  NoOp, InternalDiagnostic).
- **Selection policy**: 9 explicit rules with rejection reasons and selection
  reasons in human-readable form.
- **RuntimeStepResult.events populated**: Each cycle produces 9+ events.
- **EvaluatorTrace fully populated**: candidate_actions, rejected_actions,
  selection_reason all populated from RuntimeStepResult.
- **Memory, symbolic, cognition crates**: Updated for ActionType v2.

## Known limitations

- Memvid backend is stubbed (returns NotImplemented).
- Symbolic layer produces types but is not deeply wired into selection.
- Memory retrieval is not wired into RuntimeLoop (hits passed externally).
- SimWorld is a synthetic closed-world scenario set (7 templates).
- Python is legacy reference only.
- No real LLM integration — all cognition is deterministic mock.
- This is not production-ready, not sentient, not AGI.
