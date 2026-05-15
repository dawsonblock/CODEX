# CODEX-main 36 Hardening Report

## 1. Current package SHA

`44d56855d242ced21286841ce1f42b65b8924794f486b699ec32d73f8123ddca`

## 2. Current identity

**CODEX-main 36 hardening candidate** — bounded Rust-authoritative cognitive-runtime scaffold.

## 3. Claim guard result

`PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_sentience_claims` → pass.

## 4. Python/guard results

- `python3 -m pytest -q`: 35 passed
- `python3 architecture_guard.py`: pass
- `python3 scripts/check_proof_manifest_consistency.py`: pass
- `check_action_types`: pass
- `check_no_mv2`: pass
- `check_resource_recovery`: pass
- generated-artifact checks before/after: pass

## 5. Rust test status

Packaged verification evidence unless rerun locally in this environment: 274 passed, 0 failed, 0 ignored.

## 6. Default UI test status

Packaged log evidence: 76 passed, 0 failed, 6 ignored (`artifacts/proof/verification/ui_tests.log`).

## 7. UI provider-feature test status

Packaged log evidence: 75 passed, 0 failed, 6 ignored (`artifacts/proof/verification/ui_provider_feature_tests.log`) for command:

`cd ui/codex-dioxus && cargo test --all-targets --features ui-local-providers`

## 8. Proof manifest consistency

`python3 scripts/check_proof_manifest_consistency.py` passes with current values.

## 9. NL benchmark result

- curated: 15
- held_out: 59
- adversarial: 2
- held_out action_match_rate: 0.9152542372881356
- held_out failures: 5

Known held-out failures are documented, not hidden.

## 10. Provider/tool boundary result

- local/cloud/external provider requests: 0
- provider cannot execute tools, write memory, or override codex action
- api key storage disabled
- real external executions: 0

## 11. Retrieval policy status

Retrieval policy routing and inspection exist. Enforcement remains advisory/partial: some provider paths enforce flags, while ClaimStore compatibility keeps `governance_only` advisory. Full blocking enforcement is not claimed.

## 12. AnswerBuilder/UI metadata status

Primary answer-building path forwards `cited_claim_ids`, `cited_evidence_ids`, `rejected_action_summary`, and `answer_confidence`. Fallback/error paths may intentionally contain empty citation fields when context is unavailable. Citation fields indicate internal basis linkage, not external real-world truth.

## 13. EventOrigin status

EventOrigin enum/EventEnvelope infrastructure is expanded. Call-site adoption is partial; some events still appear as `RuntimeLoop` through default `append()` paths.

## 14. UI warnings status

UI tests pass in packaged logs, but warnings remain (cfg/unused/dead-code classes). Warnings are tracked cleanup work and are not represented as test failures.

## 15. Known limitations

- Retrieval policy enforcement is not complete in all paths.
- EventOrigin attribution is not complete in all call sites.
- Held-out benchmark has 5 unresolved diagnostic failures.
- Proof benchmarks are synthetic/diagnostic unless external evidence ingestion is explicitly enabled.
- Evidence links prove internal linkage unless independently sourced and verified.

## 16. Skipped verification and why

Rust/UI tests were not fully rerun as part of this documentation-focused patch in this environment; Rust/UI statuses are therefore reported as packaged evidence unless rerun.

## Required honesty statement

This is a bounded research scaffold. It is not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified. Provider execution remains disabled by default. Real external tool execution remains disabled or dry-run by default. Operational deployment requires separate engineering, security, legal, and safety review.
