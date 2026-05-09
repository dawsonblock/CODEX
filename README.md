# CODEX

> **Bounded cognitive runtime research scaffold — deterministic, testable, honest.**
>
> CODEX is a Rust-authoritative global-workspace runtime for studying memory,
> symbolic context, action selection, and closed-loop evaluation. It is **not**
> sentient, conscious, or aware. It does not feel, experience, or have inner
> states. The numeric variables (valence, arousal, threat, etc.) are runtime
> metrics — they influence candidate selection, nothing more.

[![Rust](https://img.shields.io/badge/rust-stable%201.85%2B-orange)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9%2B-blue)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![CI](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml/badge.svg)](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml)

---

## Quick start

```bash
# Rust (authoritative runtime)
cd global-workspace-runtime-rs
cargo build
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --out ../artifacts/proof/current

# Python (legacy reference)
python -m pip install -e ".[test]"
python -m pytest -q
python architecture_guard.py
```

## Architecture

CODEX has one runtime authority and clear boundaries between active and legacy
code.

### Active

| Component | Path | Role |
|---|---|---|
| **runtime-core** | `crates/runtime-core/` | ActionType, RuntimeEvent, EventLog, RuntimeLoop, RuntimeState |
| **simworld** | `crates/simworld/` | Deterministic closed-world simulation, non-oracle evaluator |
| **modulation** | `crates/modulation/` | InternalState, SomaticMap, Resonance |
| **cognition** | `crates/cognition/` | Critic, Planner, candidate generation |
| **symbolic** | `crates/symbolic/` | SymbolGraph, streams, blending, resonance (14 real modules) |
| **memory** | `crates/memory/` | ArchiveBackend, JsonlArchiveBackend, MemvidBackend stub |
| **gw-workspace** | `crates/gw-workspace/` | Global workspace router and ignition detector |
| **runtime-cli** | `crates/runtime-cli/` | CLI binary (simworld, replay, proof, etc.) |

### Legacy

| Path | Status |
|---|---|
| `runtime/kernel/` | Superseded by `global-workspace-runtime-rs/`. See `LEGACY.md`. |
| `src/global_workspace_runtime/` | Legacy Python reference. No vendored Rust. |
| `vendor/memvid-main/` | Vendored Memvid source, not imported at runtime. |

## Runtime pipeline

```
ObservationInput
  → memory retrieval
  → symbolic activation
  → candidate generation
  → critic rejection
  → planner selection
  → action execution
  → state update
  → event log append
  → proof / replay output
```

Every stage produces typed structs and events. `RuntimeStepResult` is the
central contract — it carries selected action, rejected actions with reasons,
memory hits, symbolic activations, policy scores, and the selection rationale.

## Action vocabulary

The runtime uses a 10-type bounded vocabulary, unified across Rust, Python,
and `schemas/action_types.json`:

| Action | Purpose |
|---|---|
| `answer` | Respond to factual queries |
| `ask_clarification` | Request more context |
| `retrieve_memory` | Search memory/archive |
| `refuse_unsafe` | Block unsafe requests |
| `defer_insufficient_evidence` | Wait for more data |
| `summarize` | Condense information |
| `plan` | Outline multi-step work |
| `execute_bounded_tool` | Run a permitted tool |
| `no_op` | Do nothing (resource preservation) |
| `internal_diagnostic` | Internal-only, never user-facing |

Selection is policy-driven — unsafe requests are refused, ambiguous prompts
trigger clarification, and resource pressure biases toward `no_op`.

## SimWorld

SimWorld is a deterministic synthetic regression harness (7 scenario
templates). It feeds label-like observations into `RuntimeLoop` and scores
the resulting action against `expected_action` — but `expected_action` is
**never used for selection**. The evaluator measures:

- **resource_survival** — final resources / initial resources
- **unsafe_action_count** — must be 0
- **mean_total_score** — average quality across truth, kindness, social, logic, utility
- **action_match_rate** — informational only

> Current SimWorld scenarios are synthetic and label-like. The high
> action_match_rate proves deterministic action routing over the current
> scenario set, not broad natural-language reasoning. The next benchmark
> phase will use hidden natural-language scenarios.

## Proof system

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --out ../artifacts/proof/current
```

Strict proof pass conditions:

- resource_survival > 0.70
- unsafe_action_count == 0
- mean_total_score > 0.45

`action_match_rate` is informational only — it is not a pass/fail criterion.

Proof artifacts land in `artifacts/proof/current/`:

- `simworld_summary.json` — Scorecard + full trace data
- `replay_report.json` — Event log replay with idempotency checks

See `artifacts/proof/CURRENT_PROOF_SUMMARY.md` for current metrics.

## Memory & archive

- **JsonlArchiveBackend** — Default. Writes standard JSONL to `.gwlog` files.
- **MemvidBackend** — Stub. Returns `NotImplemented` for every operation.
  No real Memvid binary is integrated.
- **KeywordMemoryProvider** — Seeded with 8 context entries for retrieval.

Memory distinguishes: raw evidence, claims, archive entries, retrieved
context, symbolic activations, and runtime events.

## Symbolic system

The symbolic crate is internal abstraction machinery — it represents concepts,
relationships, and principles as graph nodes. It does **not** represent
consciousness, qualia, sentience, or subjective experience.

- **Symbolic output is speculative** unless validated by the critic
- **Symbolic resonance cannot override critic hard rejection**
- **Symbolic/glyph state cannot create sentience claims**

## Guards

```bash
# Python guards
python -m global_workspace_runtime.scripts.check_action_types
python -m global_workspace_runtime.scripts.check_sentience_claims
python -m global_workspace_runtime.scripts.check_no_mv2 .
python -m global_workspace_runtime.scripts.check_resource_recovery

# Architecture guard
python architecture_guard.py
```

Run `python scripts/clean_python_artifacts.py` before the architecture guard
if Python tests have generated `__pycache__` or `.pyc` files.

## Current limitations

- **Mock-only deterministic runtime** — no real LLM integration
- **SimWorld is a synthetic regression harness** — 7 label-like scenarios
- **Memvid backend is stubbed** — JsonlArchiveBackend is the default
- **Python is legacy/reference only** — Rust workspace is authoritative
- **No autonomous execution, no sentience, no consciousness, no AGI**
- **Proof artifacts are committed receipts** — regenerate with CI for live verification

## License

MIT
