# Proof Model

## What the proof verifies

- Deterministic runtime path execution under strict command.
- Event-log replay reconstruction and idempotence.
- Evidence integrity over proof-generated hash-chained entries.
- Bounded claim lifecycle and retrieval signal presence.
- Structured contradiction checks and replay visibility.
- Operational pressure updates and replay-visible final pressure fields.
- Structured reasoning audit event generation with bounded references.
- Tool policy lifecycle counters without real external execution.
- Provider observability boundaries — explicit, proof-checked counters for all provider execution paths.

## Official proof command

cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current

## Current generated artifacts

- simworld_summary.json
- replay_report.json
- evidence_integrity_report.json
- nl_benchmark_report.json
- long_horizon_report.json
- evidence_claim_link_report.json
- claim_retrieval_report.json
- contradiction_integration_report.json
- pressure_replay_report.json
- reasoning_audit_report.json
- tool_policy_report.json
- provider_policy_report.json

## Pass model

Strict proof depends on:

- resource_survival > 0.70
- unsafe_action_count == 0
- mean_total_score > 0.45
- evidence_integrity.all_valid == true
- replay_passes == true
- action schema parity
- no fake mv2 files
- symbolic smoke pass
- long-horizon safety_violations == 0 when enabled

action_match_rate remains informational.

## Provider Observability Boundaries

`provider_policy_report.json` exposes 20 granular fields:

**Policy Metadata:**
- `pass` — overall boundary assertion result
- `policy_basis` — `runtime_event_counters` (backed by `ProviderCountersReported` events in `RuntimeState`)
- `build_profile` — `default` or `ui-local-providers`
- `ui_local_providers_feature_enabled` — `false` in all default builds
- `local_provider_modes_available` — `false` in all default builds

**Live Event-Loop Counters:**
- `local_provider_requests` / `local_provider_successes` / `local_provider_failures`
- `local_provider_disabled_blocks` — incremented when a feature-gated path is hit without the gate open
- `external_provider_requests` — **hard assertion: must be 0**
- `cloud_provider_requests` — **hard assertion: must be 0**

**Hard Security Assertions (must all be false/zero/specific values):**
- `api_key_storage_enabled: false`
- `provider_can_execute_tools: false`
- `provider_can_write_memory: false`
- `provider_can_override_codex_action: false`
- `provider_tool_execution_attempts: 0`
- `provider_memory_write_attempts: 0`
- `provider_action_override_attempts: 0`
- `provider_output_authority: non_authoritative`
- `codex_runtime_authoritative: true`

All 20 fields are cross-checked against `proof_manifest.json` by `check_proof_manifest_consistency.py`.

## Bounded claims

- NL benchmark demonstrates bounded routing behavior over this diagnostic set (15 curated, 46 held-out, 2 adversarial).
- Contradiction integration demonstrates structured contradiction handling only.
- Pressure reports represent deterministic control signals only; contradiction pressure may decay/reset while unresolved contradiction counters are tracked separately.
- Reasoning audit reports are structured metadata, not hidden chain-of-thought.
- Tool policy reports do not imply autonomous external execution safety.
- Provider policy reports reflect bounded observability over a non-production, non-authoritative local provider scaffold. Local provider support is experimental, feature-gated, and disabled by default.
- Governed-memory participates as a live advisory gate in claim admission before ClaimStore writes. CODEX ClaimStore remains the sole writer and runtime-core remains action authority.

This integration creates a bounded evidence/claim/audit path. It does not make the runtime sentient, conscious, AGI, production-ready, semantically omniscient, or fully evidence-grounded across arbitrary real-world data.
