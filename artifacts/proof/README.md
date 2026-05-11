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
