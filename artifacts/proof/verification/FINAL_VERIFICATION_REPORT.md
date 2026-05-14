# CODEX-main 32 — Final Verification Report

## 1. Package Identity

- Package identity: CODEX-main 32 governed-memory integration candidate
- Uploaded filename may vary; internal codename is authoritative.
- Decision status: integration candidate, not final freeze.

## 2. Files Changed

- global-workspace-runtime-rs/crates/simworld/Cargo.toml
- global-workspace-runtime-rs/crates/simworld/src/evaluator.rs
- global-workspace-runtime-rs/crates/runtime-core/src/event.rs
- global-workspace-runtime-rs/crates/runtime-core/src/runtime_state.rs
- global-workspace-runtime-rs/crates/runtime-core/src/reducer.rs
- global-workspace-runtime-rs/crates/runtime-cli/src/main.rs
- ui/codex-dioxus/src/bridge/runtime_client.rs
- scripts/check_proof_manifest_consistency.py
- artifacts/proof/current/*.json (regenerated)
- artifacts/proof/CURRENT_PROOF_SUMMARY.md
- artifacts/proof/README.md
- artifacts/proof/verification/proof_manifest.json
- STATUS.md
- docs/PHASE_STATUS_AND_ROADMAP.md
- docs/PROOF_MODEL.md

## 3. Commands Run

Python:

- python3 -m pip install -e ".[test]" (pass)
- python3 -m global_workspace_runtime.scripts.check_action_types (pass)
- python3 -m global_workspace_runtime.scripts.check_sentience_claims (pass)
- python3 -m global_workspace_runtime.scripts.check_no_mv2 . (pass)
- python3 -m global_workspace_runtime.scripts.check_resource_recovery (pass)
- python3 -m pytest -q (35 passed)
- python3 scripts/clean_python_artifacts.py (pass)
- python3 architecture_guard.py (pass)
- python3 scripts/architecture_guard.py (pass)
- python3 scripts/check_proof_manifest_consistency.py (pass)

Rust:

- cargo --version / rustc --version (available)
- cargo fmt --all -- --check (pass)
- cargo clippy --workspace --all-targets --all-features -- -D warnings (pass)
- cargo test --workspace --all-targets --all-features (pass)
- cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current (pass)

UI:

- cargo fmt --all -- --check (pass)
- cargo check (pass)
- cargo test (pass)
- cargo test --features ui-local-providers (pass)
- dx build (dx not available in this environment)

## 4. Current Proof Metrics

- cycles: 15
- event_count: 589
- held_out scenario count: 46
- held_out action_match_rate: 1.0
- claims_with_evidence_links: 17
- audits_with_claim_refs: 18
- real_external_executions: 0

## 5. Governed-Memory Live Hook Metrics

- runtime_integrated: true
- live_admission_hook_enabled: true
- retroactive_evaluations: 17
- live_admission_decisions: 17
- candidates_evaluated: 34
- evidence_backed_promotion_recommendations: 34
- claimstore_writes_approved_after_governed_memory: 17
- claimstore_writes_blocked_by_governed_memory: 0
- claimstore_writes_overrode_governed_memory: 0
- claimstore_writes_performed_by_codex: 17
- claimstore_writes_performed_by_governed_memory: 0
- audits_with_governed_memory_reason_codes: 17
- retrieval_plans_generated: 15
- no_api_keys: true
- no_external_calls: true
- no_mv2_activation: true

## 6. Provider Disabled-Block Proof

- default_provider_attempt_tested: true
- local_provider_disabled_blocks: 1
- local_provider_requests: 0
- external_provider_requests: 0
- cloud_provider_requests: 0
- no HTTP call in default build path: verified by feature gating and consistency scan
- no API key storage: true

## 7. Hard Invariants

- tool: real_external_executions = 0; only dry-run/scaffold evidence in proof.
- provider: default build remains local-provider disabled and non-authoritative.
- storage: no active Memvid video-container storage; no active .mv2 references in guarded surface.
- authority boundaries:
  - runtime-core remains action authority
  - ClaimStore remains claim writer
  - evidence vault remains evidence authority
  - contradiction engine remains contradiction authority
  - governed-memory remains advisory only

## 8. Remaining Limitations

- NL benchmark is diagnostic routing, not broad natural-language reasoning proof.
- Contradiction handling is structured/deduped, not semantic truth reasoning.
- Evidence grounding is strong for proof-known structured sources, not arbitrary real-world truth verification.
- Governed-memory live hook is advisory; no direct writes/execution/provider calls.
- System is an integration candidate, not final freeze.

## 9. Decision

- CODEX-main 32 governed-memory integration candidate.
- Not final freeze.

Required limitation statement:

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.
