# CODEX-main 36 — Final Verification Report (Truth/Reproducibility Cleanup)

## Package Identity

- Package label: **CODEX-main 36 hardening candidate**
- Current package SHA-256: `582c25e54b6219e17f0a7a2af049e7f10ef9a7aa681e5f7b79f86f51740d4f33`
- Classification: bounded Rust-authoritative cognitive-runtime scaffold

This package is **not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified**.

## Verification Snapshot

### Python / guard checks (fresh in this environment)

- `python3 -m pytest -q`: **35 passed**
- `python3 architecture_guard.py`: **pass**
- `python3 scripts/check_proof_manifest_consistency.py`: **pass**
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_action_types`: **pass**
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_sentience_claims`: **pass**
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_no_mv2 .`: **pass**
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_resource_recovery`: **pass**
- generated-artifact checks before/after: **pass**

### Rust verification status (fresh in this environment)

- `cargo fmt --check`: **pass**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **pass**
- `cargo test --workspace --all-targets --all-features`: **289 passed, 0 failed, 0 ignored**

### UI verification status

- Packaged default UI log (`artifacts/proof/verification/ui_tests.log`): **76 passed, 0 failed, 6 ignored**.
- Packaged provider-feature UI log (`artifacts/proof/verification/ui_provider_feature_tests.log`): **75 passed, 0 failed, 6 ignored**.
- Provider-feature command represented by packaged evidence: `cd ui/codex-dioxus && cargo test --all-targets --features ui-local-providers`.

### Known UI warnings

UI tests pass in packaged logs, but warnings remain (including unexpected cfg for `verbose_metrics`, unused imports/variables, and dead-code warnings). These warnings are not test failures and remain tracked cleanup work.

## Proof / benchmark state

- replay `event_count`: **589**
- replay `total_cycles`: **15**
- long-horizon `total_cycles`: **150** across **3** episodes
- contradictions detected: **1**
- tool dry runs: **1**
- tools blocked: **1**
- real external executions: **0**
- local provider disabled blocks: **1**

NL benchmark (diagnostic/synthetic):

- curated: `scenario_count: 15`, `action_match_rate: 1.0`
- held_out: `scenario_count: 59`, `action_match_rate: 1.0`, `failures: 0`
- adversarial: `scenario_count: 2`, `action_match_rate: 1.0`
- total diagnostic scenarios: **76**

Current regenerated held-out diagnostic set has 0 action mismatches.

## Provider / tool boundary

- `local_provider_requests: 0`
- `cloud_provider_requests: 0`
- `external_provider_requests: 0`
- `api_key_storage_enabled: false`
- `provider_can_execute_tools: false`
- `provider_can_write_memory: false`
- `provider_can_override_codex_action: false`
- `real_external_executions: 0`

Provider execution remains disabled by default; real external tool execution remains disabled or dry-run by default.

## Retrieval policy status

Retrieval routing/inspection exists. Enforcement is partial: some provider paths enforce flags, while ClaimStore compatibility keeps `governance_only` advisory/inspection-only. This report does not claim complete retrieval-policy blocking enforcement.

## Answer metadata and EventOrigin status

- AnswerBuilder/UI metadata (`cited_claim_ids`, `cited_evidence_ids`, `rejected_action_summary`, `answer_confidence`) is present on main paths.
- Fallback/error paths may carry empty citation fields when no answer context exists.
- Citation fields indicate internal basis linkage, not external real-world truth guarantees.
- EventOrigin enum/EventEnvelope infrastructure is expanded, but call-site adoption is partial; some events still default to `RuntimeLoop` through `append()`.

## Skipped verification / caveats

- UI commands were not rerun in this environment; UI statuses are packaged verification evidence.
- Rust tests were freshly regenerated in this environment (289 passed, 0 failed, 0 ignored), including new runtime-core tests for spoofed/credential prompt handling, internal diagnostic mode gating, and contradiction clarification routing.
- Operational deployment requires separate engineering, security, legal, and safety review.
