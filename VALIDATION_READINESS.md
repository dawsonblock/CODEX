# CODEX-main 36 Validation Readiness

> This document is for validation/review status only. It is not a deployment certification.

## Current identity

- Label: **CODEX-main 36 hardening candidate**
- Current package SHA-256: `44d56855d242ced21286841ce1f42b65b8924794f486b699ec32d73f8123ddca`
- Classification: bounded Rust-authoritative cognitive-runtime scaffold

## Evidence-backed verification status

Fresh checks run in this environment:

- generated-artifact checks: pass
- `pytest`: 35 passed
- architecture guard: pass
- proof manifest consistency: pass
- action schema guard: pass
- claim/sentience guard: pass
- no-mv2 guard: pass
- resource recovery: pass

Packaged evidence (unless rerun locally):

- Rust tests: 274 passed, 0 failed, 0 ignored
- UI default tests: 76 passed, 0 failed, 6 ignored
- UI provider-feature tests: 75 passed, 0 failed, 6 ignored

UI logs include warnings; this package does not claim warning-free UI logs.

## Proof and benchmark state

- curated: 15 scenarios, action_match_rate 1.0
- held_out: 59 scenarios, action_match_rate 0.9152542372881356, failures 5
- adversarial: 2 scenarios, action_match_rate 1.0
- total diagnostic scenarios: 76

The 5 held-out failures are documented and not hidden.

## Retrieval policy status

Retrieval routing and inspection exist. Enforcement is partial/advisory:

- Some provider paths enforce flags.
- ClaimStore compatibility path keeps `governance_only` advisory.
- Complete blocking enforcement is not claimed.

## Answer metadata and event provenance status

- AnswerBuilder/UI metadata forwarding exists for `cited_claim_ids`, `cited_evidence_ids`, `rejected_action_summary`, and `answer_confidence` on primary paths.
- Fallback paths may intentionally return empty citation fields when no answer context exists.
- EventOrigin enum/EventEnvelope are expanded; call-site adoption remains partial.
- Some events still use `RuntimeLoop` default through `append()`.

## Known UI warnings

UI tests pass in packaged logs, but warnings remain (including cfg/unused/dead-code warnings). Warnings are tracked cleanup work, not test failures.

## Operational limits

This package is not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified.

Operational deployment requires separate engineering, security, legal, and safety review.
