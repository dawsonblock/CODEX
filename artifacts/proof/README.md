# Proof Artifacts

## Official Regeneration Command

From global-workspace-runtime-rs:

cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current

Only this command is authoritative for artifacts in artifacts/proof/current.

## Current Bundle

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
- provider_storage_boundary_report.json
- governed_memory_integration_report.json

Provider artifacts:

- `provider_policy_report.json` is the canonical provider-boundary artifact used by consistency checks.
- `provider_storage_boundary_report.json` is a supplemental structural invariant report.

Governance artifacts:

- `governed_memory_integration_report.json` provides advisory admission gate metrics split by live admission-hook decisions and retroactive comparison evaluations.

## Current Snapshot (CODEX-main 36)

- Package identity: Internal codename CODEX-main 36 hardening candidate. Uploaded ZIP filename may vary and is not the authority source.
- Package status: integration proof candidate (not final freeze)
- SimWorld cycles: 15
- resource_survival: 0.9740
- mean_total_score: 0.6433333333
- action_match_rate: 1.0 (informational)
- replay event_count: 589
- held_out scenario_count: 59
- held_out action_match_rate: 0.8983050847457628 (6 failures)
- raw_contradictions_detected: 1
- unique_contradictions_detected: 1
- duplicate_contradictions_suppressed: 0
- real_external_executions: 0
- audits_with_claim_refs: 18

## Report Contract

Each integration report includes:

- pass
- scenario_count
- counters
- limitations
- proof_command
- generated_timestamp

## Interpretation Boundaries

- action_match_rate is informational only.
- NL benchmark is a diagnostic routing benchmark, not broad reasoning proof.
- Contradiction reports describe structured contradiction handling only.
- Pressure reports describe deterministic control-signal fields only.
- Tool policy reports do not imply real external execution capability.

## History and Verification

- artifacts/proof/history contains historical snapshots for comparison.
- artifacts/proof/verification contains verification receipts and final verification report.
