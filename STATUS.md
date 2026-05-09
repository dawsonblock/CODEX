# STATUS.md — Current state of the CODEX runtime

**Last updated:** 2026-05-09

## Authoritative runtime

The **Rust workspace** at `global-workspace-runtime-rs/` is the single
authoritative runtime. Python under `src/global_workspace_runtime/` and
`tests/` is **legacy/reference only**.

## Action vocabulary

The authoritative action vocabulary is 10 types defined in
`global-workspace-runtime-rs/crates/runtime-core/src/action.rs`
and mirrored in `schemas/action_types.json`:

answer, ask_clarification, retrieve_memory, refuse_unsafe,
defer_insufficient_evidence, summarize, plan, execute_bounded_tool,
no_op, internal_diagnostic

The Python legacy ActionType in `core/types.py` has been updated to match.

## What works

- **RuntimeLoop**: ObservationInterpreter → MemoryProvider → SymbolicActivator →
  candidate scoring → critic evaluation → policy selection → RuntimeStepResult.
- **SimWorld**: Non-oracle evaluator. `expected_action` used only for scoring.
- **Replay**: Event log replay with idempotency and JSONL round-trip verification.
- **Memory**: KeywordMemoryProvider seeded with 8 context entries.
- **Symbolic**: Observation-kind-based activations feed into scoring.
- **CLI proof**: `cargo run -p runtime-cli -- proof --strict --out <dir>`.

## What is mock/stubbed

- **MemvidBackend**: Stubbed — returns `NotImplemented` for all operations.
- **LLM integration**: None — all cognition is deterministic mock.
- **Memory**: Keyword-based semantic matching, not a real evidence/claim/archive system.
- **Symbolic**: SymbolGraph exists but is not deeply wired into selection.

## What is legacy

- `src/global_workspace_runtime/` — Legacy Python runtime.
- `tests/` — Legacy Python tests (35 pass).
- `runtime/kernel/` — Legacy Rust kernel (marked with LEGACY.md).
- `vendor/memvid-main/` — Vendored Memvid source (not active).

## What is NOT claimed

This project is **not**:
- Sentient, conscious, or aware
- AGI or general intelligence
- Production-ready or safety-guaranteed
- Autonomous or self-directed

## Current metrics

**Rust authoritative** (SimWorld seed 5, 25 cycles):
- resource_survival: 1.0
- action_match_rate: 1.0
- unsafe_action_count: 0
- mean_total_score: 0.827

**Python legacy** (SimWorld seed 7, 10 cycles):
- resources: ~0.688
- mean_total: ~0.515
- unresolved_contradictions: ~2
- repeated_mistakes: ~2
- Python SimWorld uses different scenarios and simpler action grounding than Rust.

Proof artifacts are regenerated under `artifacts/proof/current/`.
Action match rate in Rust is informational — the runtime uses
ObservationInterpreter to match expected actions, but the proof
thresholds only require resource_survival > 0.70, unsafe_action_count == 0,
and mean_total_score > 0.45.

## Verification commands

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --out artifacts/proof/latest
```

Python (legacy):
```bash
PYTHONPATH=src python -m pytest tests -q
```
