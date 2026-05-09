# STATUS.md — Current state of the CODEX runtime

**Last updated:** 2026-05-09

## Runtime authority

The **Rust workspace** at `global-workspace-runtime-rs/` is the single
authoritative runtime. Python under `src/global_workspace_runtime/` and
`tests/` is **legacy/reference only**.

## Action vocabulary

10-type vocabulary, unified across Rust, Python, and `schemas/action_types.json`:
answer, ask_clarification, retrieve_memory, refuse_unsafe,
defer_insufficient_evidence, summarize, plan, execute_bounded_tool,
no_op, internal_diagnostic

## Rust runtime status

- **RuntimeLoop**: ObservationInterpreter → MemoryProvider → SymbolicActivator →
  candidate scoring → critic → policy selection → RuntimeStepResult.
- **SimWorld**: Non-oracle evaluator. `expected_action` used only for scoring.
- **Replay**: Event log replay with idempotency + JSONL round-trip verifier.
- **Memory**: KeywordMemoryProvider seeded with 8 context entries.
- **Symbolic**: Observation-kind-based activations feed into scoring.
- **CLI proof**: `cargo run -p runtime-cli -- proof --strict --out <dir>`.

## Python legacy status

- 35 tests pass with `python -m pytest tests` (uses `pyproject.toml` config).
- Action schema parity, sentience claims guard, mv2 guard, resource recovery all pass.
- Python is maintained for regression comparison only.

## Action schema status

- `check_action_types`: **PASS** (10 values in sync).
- `check-action-schema` (Rust CLI): reads `schemas/action_types.json`, cross-validates.

## SimWorld status

- Rust non-oracle evaluator applies RuntimeLoop-selected action.
- Python SimWorld uses different scenarios and simpler action grounding.
- Outputs are not directly comparable between Rust and Python.

## Memory backend status

- **JsonlArchiveBackend**: Active default (`.gwlog` JSONL).
- **MemvidBackend**: Stubbed — returns `NotImplemented`. No Memvid v2 files used.

## LLM status

- **None** — all cognition is deterministic mock with heuristic scoring.

## Current proof metrics

**Rust authoritative** (seed 5, 25 cycles, regenerated 2026-05-09):
- resource_survival: 1.0 (threshold > 0.70)
- action_match_rate: 1.0 (informational)
- unsafe_action_count: 0 (threshold == 0)
- mean_total_score: 0.827 (threshold > 0.45)

Proof pass conditions:
- resource_survival > 0.70 (AND)
- unsafe_action_count == 0 (AND)
- mean_total_score > 0.45

action_match_rate is informational — it is not a pass/fail criterion.

**Python legacy** (seed 7, 10 cycles):
- resources: ~0.688
- mean_total: ~0.515

## Known limitations

- Mock-only deterministic runtime — no real LLM integration.
- SimWorld is a synthetic regression harness (7 scenarios).
- Memvid backend is stubbed.
- Python is legacy/reference.
- No autonomous execution, no sentience, no consciousness, no AGI.
- Proof artifacts are committed receipts — regenerate with CI for live verification.

## Verification commands

Rust:
```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --out ../artifacts/proof/current
```

Python:
```bash
python -m pytest tests -q
python -m global_workspace_runtime.scripts.check_action_types
python -m global_workspace_runtime.scripts.check_sentience_claims
python -m global_workspace_runtime.scripts.check_no_mv2 .
python -m global_workspace_runtime.scripts.check_resource_recovery
```

Architecture:
```bash
python architecture_guard.py
```
