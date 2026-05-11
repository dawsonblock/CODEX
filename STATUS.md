# STATUS.md

Last updated: 2026-05-11 (live verified)
Codename: CODEX-main 32
Status: CODEX-main 32 integration proof candidate (not final freeze)

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
- resource_survival: 0.8020
- unsafe_action_count: 0
- mean_total_score: 0.6105952381
- action_match_rate: 1.0 (informational)
- replay event_count: 1132
- replay_passes: true
- evidence_entries: 30
- claims_asserted: 30
- claims_retrieved: 2
- claims_with_evidence_links: 2
- contradictions_checked: 6
- contradictions_detected: 326
- raw_contradictions_detected: 326
- unique_contradictions_detected: 1
- duplicate_contradictions_suppressed: 325
- pressure_updates: 78
- policy_bias_applications: 29
- reasoning_audits: 31
- audits_with_claim_refs: 3
- tool_requests: 2
- tool_dry_runs: 1
- tool_scaffold_executed: 1
- tools_blocked: 1
- real_external_executions: 0

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
- NL benchmark remains diagnostic routing over 28 scenarios, not broad reasoning proof.
- Evidence-backed claim coverage remains sparse (2/30 linked).

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.
