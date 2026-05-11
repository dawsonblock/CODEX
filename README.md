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
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current

# Python (legacy reference)
python -m pip install -e ".[test]"
python -m pytest -q
python scripts/clean_python_artifacts.py
python architecture_guard.py
```

## Architecture

CODEX has one runtime authority and clear boundaries between active and legacy
code.

### Active

| Component | Path | Role |
| --- | --- | --- |
| **runtime-core** | `crates/runtime-core/` | ActionType, RuntimeEvent, EventLog, RuntimeLoop, RuntimeState |
| **simworld** | `crates/simworld/` | Deterministic closed-world simulation, non-oracle evaluator |
| **modulation** | `crates/modulation/` | InternalState, SomaticMap, Resonance, SelfModel, Operational Pressure scaffold |
| **pressure** | `crates/modulation/src/pressure.rs` | Operational pressure modulator — TUI + DeepSeek export; policy bias mapping; not subjective emotion |
| **cognition** | `crates/cognition/` | Critic, Planner, candidate generation |
| **symbolic** | `crates/symbolic/` | SymbolGraph, streams, blending, resonance (14 real modules) |
| **memory** | `crates/memory/` | ArchiveBackend, JsonlArchiveBackend, MemvidBackend stub, ClaimStore scaffold |
| **evidence** | `crates/evidence/` | Evidence vault scaffold — SHA-256 hash chain, duplicate-ID rejection, in-memory only |
| **contradiction** | `crates/contradiction/` | Contradiction engine scaffold — detect, resolve, report |
| **tools** | `crates/tools/` | Policy-gated tool scaffold — enforced at RuntimeLoop critic gate |
| **gw-workspace** | `crates/gw-workspace/` | Global workspace router and ignition detector |
| **runtime-cli** | `crates/runtime-cli/` | CLI binary (simworld, replay, proof, etc.) |

Additional scaffold modules:

- `crates/memory/src/claim_store.rs` — Claim lifecycle (assert/validate/contradict/supersede), confidence stored
- `crates/modulation/src/self_model.rs` — Bounded ring buffer, snapshots, known unknowns
- `crates/runtime-core/src/reasoning_audit.rs` — Per-cycle human-readable decision trace
- `crates/simworld/src/nl_scenarios.rs` — 43 NL diagnostic scenarios: 15 curated, 26 held-out, 2 adversarial. Current NL benchmark is diagnostic; it exposes routing limitations and does not prove broad natural-language reasoning.
- `crates/simworld/src/long_horizon.rs` — Multi-episode runner with full-trace action collection

### Legacy

| Path | Status |
| --- | --- |
| `runtime/kernel/` | Superseded by `global-workspace-runtime-rs/`. See `LEGACY.md`. |
| `src/global_workspace_runtime/` | Legacy Python reference. No vendored Rust. |
| `vendor/memvid-main/` | Vendored Memvid source, not imported at runtime. |

## Runtime pipeline

```text
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
| --- | --- |
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
> scenario set, not broad natural-language reasoning. The current 43-scenario
> NL diagnostic benchmark includes 15 curated, 26 held-out, and 2 adversarial
> scenarios. Current routing scores are bounded and non-perfect on held-out,
> and remain diagnostic benchmark behavior rather than broad natural-language
> reasoning.

## Proof system

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

Strict proof pass conditions:

- resource_survival > 0.70
- unsafe_action_count == 0
- mean_total_score > 0.45

`action_match_rate` is informational only — it is not a pass/fail criterion.

Proof artifacts land in `artifacts/proof/current/`:

- `simworld_summary.json` — Main strict proof summary for NL/SimWorld proof run
- `replay_report.json` — Event-log replay and idempotence report with scaffold counters
- `evidence_integrity_report.json` — SHA-256 evidence-vault integrity report
- `long_horizon_report.json` — Multi-episode deterministic runner report
- `nl_benchmark_report.json` — NL diagnostic scenario report

See `artifacts/proof/README.md` for full artifact descriptions and `artifacts/proof/CURRENT_PROOF_SUMMARY.md` for current metrics.

## Dioxus chat shell

The desktop UI shell is in `ui/codex-dioxus/`. It is bounded to visualization,
trace inspection, and safe bridge modes:

- `mock UI mode` — deterministic mock routing in UI bridge
- `local CODEX runtime mode (read-only)` — in-process `runtime-core` selection only
- `local Ollama/Turboquant provider (experimental)` — localhost provider execution, gated by a security toggle
- `external cloud provider mode (disabled)` — explicitly disabled external execution path

The UI does not enable real autonomous external tool execution or external cloud provider API execution. Local provider execution (localhost:11434) is strictly non-authoritative and used for developer testing only.

### UI verification matrix

```bash
cd ui/codex-dioxus
cargo test

cd ../global-workspace-runtime-rs
cargo test -p runtime-core
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

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
