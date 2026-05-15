# CODEX

Bounded cognitive-runtime research scaffold focused on deterministic action
selection, evidence-aware state, and reproducible proof artifacts.

CODEX is **Rust-authoritative**. It is not a sentient system, not AGI, and not
an autonomous external executor.

[![Rust](https://img.shields.io/badge/rust-1.95.0-orange)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9%2B-blue)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![CI](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml/badge.svg)](https://github.com/dawsonblock/CODEX/actions/workflows/ci.yml)

## Why CODEX

CODEX exists to test bounded cognitive-runtime mechanics in a way that is:

- deterministic
- auditable
- replayable
- explicit about system limits

The runtime is intentionally conservative: safety-first action routing,
structured evidence/claim linkage, policy-gated tools, and no default external
provider execution.

## Quick Start

### Rust workspace

```bash
cd global-workspace-runtime-rs
cargo build
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

### Python reference surface

```bash
python -m pip install -e ".[test]"
python -m pytest -q
python3 scripts/clean_python_artifacts.py
python3 scripts/check_no_generated_artifacts.py
python3 scripts/architecture_guard.py
```

## Repository Structure

### Active runtime surface

| Component | Path | Role |
| --- | --- | --- |
| runtime-core | `global-workspace-runtime-rs/crates/runtime-core/` | Runtime loop, action types, event schema, audit contracts |
| simworld | `global-workspace-runtime-rs/crates/simworld/` | Deterministic evaluator and scenario execution |
| modulation | `global-workspace-runtime-rs/crates/modulation/` | Pressure/state modulation scaffolding |
| cognition | `global-workspace-runtime-rs/crates/cognition/` | Candidate generation, critic/planner composition |
| symbolic | `global-workspace-runtime-rs/crates/symbolic/` | Symbol graph and symbolic context helpers |
| memory | `global-workspace-runtime-rs/crates/memory/` | Archive backend, claim store scaffold |
| evidence | `global-workspace-runtime-rs/crates/evidence/` | Evidence vault and integrity primitives |
| contradiction | `global-workspace-runtime-rs/crates/contradiction/` | Structured contradiction detection and status handling |
| tools | `global-workspace-runtime-rs/crates/tools/` | Policy-gated tool scaffolding |
| governed-memory | `global-workspace-runtime-rs/crates/governed-memory/` | Admission/trust/routing/audit salvage layer |
| runtime-cli | `global-workspace-runtime-rs/crates/runtime-cli/` | Build/test/proof orchestration entrypoint |

### Legacy or reference surface

| Path | Status |
| --- | --- |
| `runtime/kernel/` | Superseded by Rust workspace |
| `src/global_workspace_runtime/` | Legacy Python reference |
| `vendor/memvid-main/` | Vendored reference only, not runtime authority |

## Runtime Model

```text
ObservationInput
  -> memory retrieval
  -> symbolic activation
  -> candidate generation
  -> critic filtering
  -> planner selection
  -> action execution
  -> state update
  -> event log append
  -> proof/replay output
```

`RuntimeStepResult` is the central typed output contract for selected action,
rejections, evidence/claim references, symbolic activations, and audit data.

## Bounded Action Vocabulary

CODEX uses a fixed 10-action schema shared across Rust/Python/JSON contracts:

- `answer`
- `ask_clarification`
- `retrieve_memory`
- `refuse_unsafe`
- `defer_insufficient_evidence`
- `summarize`
- `plan`
- `execute_bounded_tool`
- `no_op`
- `internal_diagnostic`

`internal_diagnostic` is internal-only and should never be exposed as a
user-facing action.

## Governed Memory Integration

`governed-memory` is a CODEX-native salvage layer that adds bounded memory
governance without changing runtime authority.

It provides:

- candidate admission controls
- source trust scoring
- retrieval intent routing
- read-only retrieval planning
- conflict metadata and governed audit records

It does not provide:

- default memvid container storage activation
- external provider execution paths
- automatic untrusted memory writes
- replacement of CODEX evidence/claim/proof authority

## Verification Workflow

### Rust checks

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

### Python and guard checks

```bash
python -m global_workspace_runtime.scripts.check_action_types
python -m global_workspace_runtime.scripts.check_sentience_claims
python -m global_workspace_runtime.scripts.check_no_mv2 .
python -m global_workspace_runtime.scripts.check_resource_recovery
python3 scripts/check_no_generated_artifacts.py
python3 scripts/architecture_guard.py
```

If Python tests generated caches, run:

```bash
python3 scripts/clean_python_artifacts.py
```

## Proof Artifacts

Proof outputs are written to `artifacts/proof/current/`.

Primary artifacts:

- `simworld_summary.json`
- `replay_report.json`
- `evidence_integrity_report.json`
- `nl_benchmark_report.json`
- `long_horizon_report.json`
- `evidence_claim_link_report.json`
- `claim_retrieval_report.json`
- `contradiction_integration_report.json`
- `pressure_replay_report.json`
- `reasoning_audit_report.json`
- `tool_policy_report.json`
- `provider_policy_report.json` (canonical provider boundary artifact)
- `governed_memory_integration_report.json`

Supplemental artifact:

- `provider_storage_boundary_report.json` (structural provider invariants,
  non-canonical for manifest consistency checks)

See:

- `artifacts/proof/README.md`
- `artifacts/proof/CURRENT_PROOF_SUMMARY.md`
- `artifacts/proof/verification/proof_manifest.json`
- `RETRIEVAL_POLICY_SPEC.md` (retrieval intent routing and advisory flags clarification)

## UI Boundary Notes

The Dioxus UI in `ui/codex-dioxus/` is a bounded shell for:

- trace visualization
- runtime introspection
- controlled mode switching

Default builds keep provider execution disabled. Experimental local provider
paths are feature-gated and non-authoritative.

**Test Coverage**: See [UI_TEST_REPORT.md](UI_TEST_REPORT.md) for comprehensive UI test results (76 PASSED | 6 IGNORED | 0 FAILED) including local-providers feature gate validation and provider policy boundary enforcement.

## Current Limits

- SimWorld is a synthetic regression harness, not broad reasoning proof.
- Contradiction handling is structured and bounded, not semantic truth engine.
- Evidence-backed linkage is strong for proof-known structured sources, not
  arbitrary real-world grounding.
- No default autonomous external tool execution.
- No sentience/consciousness/AGI claims.

## License

MIT
