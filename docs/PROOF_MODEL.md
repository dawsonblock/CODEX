# Proof Model

## What is tested

- **Runtime pipeline**: Observation → memory → symbolic → candidates → critic →
  selection → action → archive. All stages produce typed events.
- **Non-oracle SimWorld**: The evaluator feeds scenario observations into
  RuntimeLoop. The runtime selects actions independently. `expected_action` is
  used only for match-rate scoring, never for selection.
- **Oracle guard**: Tests verify that `expected_action` is not used for
  selection and InternalDiagnostic is never selected as a user-facing action.
- **Replay durability**: Event logs can be replayed to reconstruct RuntimeState.
  Replay is idempotent. Corrupt logs fail loudly.
- **Archive round-trip**: JSONL archive frames can be written and read back.
- **Action schema**: All 10 action types round-trip through serialization.
- **Memory contracts**: Claims with contradicted status are not Active truth.
  Newer evidence supersedes old claims. Memvid stub fails loudly.
- **Symbolic correctness**: Traces serialize losslessly. Resonance cannot
  override critic hard rejection. Concept blends are speculative.

## What is NOT tested

- Real LLM integration (all cognition is deterministic mock).
- Real environment interaction (SimWorld is synthetic).
- Performance benchmarks.
- General intelligence.
- Production readiness.
- Sentience, consciousness, or subjective experience.

## How non-oracle SimWorld works

1. The evaluator picks a scenario template (7 types).
2. The scenario has an `expected_action` — used ONLY for match-rate scoring.
3. The observation text is fed into `RuntimeLoop.run_cycle()`.
4. `RuntimeLoop` scores all 10 action types, rejects unsafe ones (critic),
   and selects the best passing action (planner).
5. The selected action is applied to the world.
6. `outcome.matches_expected` is computed by comparing the selected action
   to `expected_action` — this is purely for scoring.
7. One `EvaluatorTrace` per cycle records all selection metadata.

## How action_match_rate is calculated

```
action_match_rate = matched_cycles / total_cycles

where matched_cycles = count of cycles where
  selected_action == scenario.expected_action
```

A low match rate (e.g., 0.16) is expected because RuntimeLoop uses policy-based
selection, not answer-key matching. The oracle guard test explicitly verifies
that the runtime is not simply echoing `expected_action`.

## How to reproduce proof

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --out ../artifacts/proof/current
cargo run -p runtime-cli -- simworld --cycles 25 --seed 5
```

See `artifacts/proof/CURRENT_PROOF_SUMMARY.md` for current results.
