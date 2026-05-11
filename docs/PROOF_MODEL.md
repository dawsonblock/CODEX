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

## Bounded claims

- NL benchmark demonstrates bounded routing behavior over this diagnostic set.
- Contradiction integration demonstrates structured contradiction handling only.
- Pressure reports represent deterministic control signals only.
- Reasoning audit reports are structured metadata, not hidden chain-of-thought.
- Tool policy reports do not imply autonomous external execution safety.

This integration creates a bounded evidence/claim/audit path. It does not make the runtime sentient, conscious, AGI, production-ready, semantically omniscient, or fully evidence-grounded across arbitrary real-world data.
