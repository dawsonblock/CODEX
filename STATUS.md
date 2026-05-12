# STATUS.md

Last updated: 2026-05-11 (live verified)
Codename: CODEX-main 32
Status: CODEX-main 32 Integration Proof Candidate

CODEX-main 32 is an integration proof candidate, not final freeze.

Reasons for Integration Proof Candidate status:
- Rust/UI verification is receipt-backed unless rerun locally.
- Local provider support exists only behind an experimental feature flag.
- Provider counters are live, runtime-event-loop backed counters.
- NL benchmark is diagnostic routing, not broad natural-language reasoning.
- Contradiction handling is structured/deduped, not semantic truth reasoning.
- Evidence-backed claim linkage is strong for proof-known structured sources, not arbitrary real-world data.

See artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md for full limitations.

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

- SimWorld cycles: 15
- resource_survival: 0.9740
- unsafe_action_count: 0
- mean_total_score: 0.6433333333
- action_match_rate: 1.0 (informational)
- replay event_count: 557
- replay_passes: true
- evidence_entries: 96
- claims_asserted: 17
- claims_retrieved: 17
- claims_with_evidence_links: 17
- contradictions_checked: 3
- contradictions_detected: 1
- raw_contradictions_detected: 1
- unique_contradictions_detected: 1
- duplicate_contradictions_suppressed: 0
- pressure_updates: 39
- policy_bias_applications: 16
- reasoning_audits: 33
- audits_with_claim_refs: 18
- tool_requests: 2
- tool_dry_runs: 1
- tool_scaffold_executed: 1
- tools_blocked: 1
- real_external_executions: 0

NL benchmark snapshot:
- curated: 15 scenarios, action_match_rate 1.00
- held_out: 26 scenarios, action_match_rate 1.00
- adversarial: 2 scenarios, action_match_rate 1.00

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
- provider_policy_report.json

## Boundaries

- 10-action schema remains unchanged.
- Rust remains authoritative.
- No real autonomous external tool execution or external cloud provider API execution is enabled.
- Local provider execution (Ollama/Turboquant via localhost) requires the `ui-local-providers` Cargo feature.
  Default builds contain zero provider HTTP code paths.
- When `ui-local-providers` is active, provider calls are localhost-only, approval-gated, and non-authoritative.
  Provider output cannot execute tools, write memory, or override CODEX selected_action.
- Contradiction handling remains structured, not semantic truth reasoning.
- NL benchmark remains diagnostic routing over 43 scenarios, not broad reasoning proof.
- Evidence-backed claim linkage improved for proof-known evaluator evidence and remains bounded to structured sources.

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.
