# FINAL VERIFICATION REPORT

Date: 2026-05-11
Package identity: CODEX-main 32 Final Freeze Candidate
Decision: Final Freeze Candidate

## 1. Files changed

- global-workspace-runtime-rs/crates/simworld/src/evaluator.rs
- global-workspace-runtime-rs/crates/simworld/src/nl_scenarios.rs
- global-workspace-runtime-rs/crates/runtime-cli/src/main.rs
- ui/codex-dioxus/src/bridge/types.rs
- ui/codex-dioxus/src/bridge/runtime_client.rs
- ui/codex-dioxus/src/components/action_trace_panel.rs
- ui/codex-dioxus/src/components/message_bubble.rs
- artifacts/proof/current/*.json (regenerated via official proof command)
- artifacts/proof/CURRENT_PROOF_SUMMARY.md
- artifacts/proof/README.md
- artifacts/proof/verification/proof_manifest.json
- artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md
- STATUS.md
- README.md
- docs/PHASE_STATUS_AND_ROADMAP.md
- docs/PROOF_MODEL.md

## 2. Commands run

Python:
- python -m pip install -e ".[test]"
- python -m global_workspace_runtime.scripts.check_action_types
- python -m global_workspace_runtime.scripts.check_sentience_claims
- python -m global_workspace_runtime.scripts.check_no_mv2 .
- python -m global_workspace_runtime.scripts.check_resource_recovery
- python -m pytest -q
- python scripts/clean_python_artifacts.py
- python architecture_guard.py
- python scripts/architecture_guard.py
- python scripts/check_proof_manifest_consistency.py

Rust:
- cargo --version
- rustc --version
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo test --workspace --all-targets --all-features
- cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current

UI:
- cargo fmt --all -- --check
- cargo check
- cargo test
- dx build (not run; dx CLI unavailable)

## 3. Command results

- Python checks: pass (35 passed in pytest)
- Rust format/clippy/tests: pass
- Official strict proof command: pass
- UI fmt/check/test: pass (25 tests)
- dx build: unavailable in environment

## 4. Proof artifacts regenerated

- artifacts/proof/current/simworld_summary.json
- artifacts/proof/current/replay_report.json
- artifacts/proof/current/evidence_integrity_report.json
- artifacts/proof/current/nl_benchmark_report.json
- artifacts/proof/current/long_horizon_report.json
- artifacts/proof/current/evidence_claim_link_report.json
- artifacts/proof/current/claim_retrieval_report.json
- artifacts/proof/current/contradiction_integration_report.json
- artifacts/proof/current/pressure_replay_report.json
- artifacts/proof/current/reasoning_audit_report.json
- artifacts/proof/current/tool_policy_report.json

## 5. Current metrics

- evidence_entries: 96
- claims_asserted: 17
- claims_with_evidence_links: 17
- evidence_backed_claim_ratio: 1.0
- held_out scenario count: 26
- held_out action_match_rate: 1.0
- raw contradictions: 1
- unique contradictions: 1
- duplicate contradictions suppressed: 0
- audits_with_claim_refs: 18
- real_external_executions: 0

## 6. Before/after deltas

- Evidence-backed claim coverage: 2/30 -> 17/17 in current strict proof run
- UI metadata_quality: absent -> explicit RuntimeGrounded, PartiallyGrounded, MockOnly, Unavailable
- Held-out scenario count: 11 -> 26
- Held-out routing score: 0.9615384615 -> 1.0

## 7. UI bridge status

- Mock mode: enabled, metadata_quality = MockOnly
- Local runtime read-only mode: enabled, event-derived evidence/claim/audit IDs when available
- Metadata quality surfaced in UI: yes
- External providers: disabled
- Shell execution from runtime_client: none
- Real tool execution: disabled

## 8. Remaining limitations

- NL benchmark remains a bounded diagnostic routing benchmark, not broad natural-language reasoning proof.
- Contradiction handling remains structured/deduped, not semantic contradiction reasoning.
- Evidence grounding is bounded to structured proof-known/evaluator sources.
- Tool policy remains scaffolded; real_external_executions remains 0.
- Local runtime bridge is safe/read-only and not a production assistant.

## 9. Final decision

- Final Freeze Candidate

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.
