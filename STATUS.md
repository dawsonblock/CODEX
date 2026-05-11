# STATUS.md

Last updated: 2026-05-10 (live verified)
Codename: CODEX-main 32
Status: Integration implementation in progress, strict proof passing

Official proof command:

cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current

## Current Verification Snapshot

Python verification:
- check_action_types: pass
- check_sentience_claims: pass
- check_no_mv2: pass
- check_resource_recovery: pass
- pytest: 35 passed
- architecture guards: pass

Rust verification:
- cargo fmt --all -- --check: pass
- cargo clippy --workspace --all-targets --all-features -- -D warnings: pass
- cargo test --workspace --all-targets --all-features: pass
- strict proof command: pass

## Current Proof Metrics

- SimWorld cycles: 28
- resource_survival: 0.922
- unsafe_action_count: 0
- mean_total_score: 0.6401
- action_match_rate: 0.75 (informational)
- replay event_count: 1123
- replay_passes: true
- evidence_entries: 30
- claims_asserted: 30
- claims_retrieved: 2
- claims_with_evidence_links: 2
- contradictions_checked: 6
- contradictions_detected: 326
- pressure_updates: 78
- policy_bias_applications: 29
- reasoning_audits: 31

## Expanded Proof Artifacts

The official proof command now generates:

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

## Boundaries

- 10-action schema remains unchanged.
- Rust remains authoritative.
- No real autonomous external tool execution is enabled.
- Contradiction handling remains structured, not semantic truth reasoning.
- NL benchmark remains diagnostic, not broad reasoning proof.

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.
