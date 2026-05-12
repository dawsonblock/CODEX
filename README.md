# CODEX

> Bounded cognitive runtime research scaffold. Deterministic, testable, honest.
>
> CODEX is a Rust-authoritative global-workspace runtime for studying memory,
> symbolic context, action selection, and closed-loop evaluation. It is not
> sentient, conscious, aware, or autonomous in the human sense. Runtime
> metrics such as valence, arousal, and threat are selection signals only.

[![Rust](https://img.shields.io/badge/rust-stable%201.85%2B-orange)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9%2B-blue)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![CI](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml/badge.svg)](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml)

## At A Glance

- Rust runtime is authoritative.
- Evidence, claims, contradiction, and proof systems are bounded and explicit.
- Memvid storage remains stubbed out in the default CODEX path.
- Governed memory has been salvaged into a CODEX-native `governed-memory` crate.
- External provider execution stays disabled by default.

## Quick Start

```bash
cd global-workspace-runtime-rs
cargo build
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

For the Python reference surface:

```bash
python -m pip install -e ".[test]"
python -m pytest -q
python scripts/clean_python_artifacts.py
python architecture_guard.py
```

## Project Layout

### Active runtime

| Component | Path | Role |
| --- | --- | --- |
| runtime-core | `crates/runtime-core/` | Action vocabulary, runtime loop, event log, reasoning audit |
| simworld | `crates/simworld/` | Deterministic simulation and non-oracle evaluator |
| modulation | `crates/modulation/` | Internal state, self model, operational pressure scaffold |
| cognition | `crates/cognition/` | Critic, planner, candidate generation |
| symbolic | `crates/symbolic/` | Graph-based symbolic abstraction machinery |
| memory | `crates/memory/` | Archive backend, claim store scaffold, memvid stub |
| evidence | `crates/evidence/` | Evidence vault scaffold with SHA-256 hash chain |
| contradiction | `crates/contradiction/` | Contradiction detection, resolution, reporting |
| tools | `crates/tools/` | Policy-gated tool scaffold |
| governed-memory | `crates/governed-memory/` | CODEX-native memory admission, trust, retrieval, audit |
| runtime-cli | `crates/runtime-cli/` | CLI for simworld, replay, proof, and diagnostics |

### Legacy surface

| Path | Status |
| --- | --- |
| `runtime/kernel/` | Superseded by `global-workspace-runtime-rs/` |
| `src/global_workspace_runtime/` | Legacy Python reference |
| `vendor/memvid-main/` | Vendored reference only, not runtime authority |

## Runtime Flow

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

Every stage emits typed structs and events. `RuntimeStepResult` is the central
contract for selected action, rejections, memory hits, symbolic activations,
policy scores, and rationale.

## Action Vocabulary

The runtime uses a 10-action bounded vocabulary, shared across Rust, Python,
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
| `no_op` | Preserve resources |
| `internal_diagnostic` | Internal-only, never user-facing |

Selection is policy-driven. Unsafe requests are refused, ambiguous prompts
trigger clarification, and resource pressure biases toward `no_op`.

## Governed Memory

`crates/governed-memory/` is the CODEX-native salvage layer for bounded memory
governance. It preserves useful Memvid-Human ideas without importing Memvid
storage, provider execution, or embedding dependencies.

What it provides:

- candidate admission for evidence-backed memory
- source trust scoring
- retrieval intent routing
- read-only retrieval planning
- conflict metadata and governed audit records
- CODEX adapters that avoid provider metadata

What it intentionally does not provide:

- `.mv2` storage as a default path
- OpenAI / Anthropic / Tavily execution paths
- automatic memory writes from external providers
- replacement of CODEX evidence, claim, contradiction, or proof systems

## Verification

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

Strict proof thresholds:

- resource_survival > 0.70
- unsafe_action_count == 0
- mean_total_score > 0.45

`action_match_rate` is informational only.

Python and architecture guards:

```bash
python -m global_workspace_runtime.scripts.check_action_types
python -m global_workspace_runtime.scripts.check_sentience_claims
python -m global_workspace_runtime.scripts.check_no_mv2 .
python -m global_workspace_runtime.scripts.check_resource_recovery
python architecture_guard.py
```

Run `python scripts/clean_python_artifacts.py` before the architecture guard if
Python tests created `__pycache__` or `.pyc` files.

## Proof Artifacts

Proof runs write to `artifacts/proof/current/`.

- `simworld_summary.json` - Main strict proof summary
- `replay_report.json` - Event-log replay and idempotence report
- `evidence_integrity_report.json` - SHA-256 evidence vault integrity report
- `long_horizon_report.json` - Multi-episode deterministic runner report
- `nl_benchmark_report.json` - NL diagnostic scenario report

See `artifacts/proof/README.md` for full artifact descriptions and
`artifacts/proof/CURRENT_PROOF_SUMMARY.md` for current metrics.

## UI Shell

The desktop UI shell lives in `ui/codex-dioxus/` and is bounded to
visualization, trace inspection, and safe bridge modes:

- mock UI mode - deterministic mock routing
- local CODEX runtime mode (read-only) - in-process selection only
- external provider mode (disabled) - explicitly disabled execution path

The UI does not enable real autonomous external tool execution.

## Current Limits

- Mock-only deterministic runtime, no real LLM integration
- SimWorld is a synthetic regression harness
- Memvid backend is stubbed, JsonlArchiveBackend is the default
- Python remains legacy/reference only
- No sentience, consciousness, or AGI claims

## License

MIT
- **SimWorld is a synthetic regression harness** — 7 label-like scenarios
- **Memvid backend is stubbed** — JsonlArchiveBackend is the default
- **Python is legacy/reference only** — Rust workspace is authoritative
- **No autonomous execution, no sentience, no consciousness, no AGI**
- **Proof artifacts are committed receipts** — regenerate with CI for live verification

## License

MIT
