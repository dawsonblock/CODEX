# Proof Limitations (Current)

## Scope limits

- Proof outputs are synthetic/diagnostic runtime checks, not broad real-world truth verification.
- Evidence links demonstrate internal claim/evidence linkage, not automatic real-world correctness.

## Verification limits

- Python/guard checks were rerun in this environment and pass.
- Rust/UI statuses may be packaged evidence unless rerun locally.
- UI packaged logs include warnings; warnings are not test failures.

## Retrieval policy limit

Retrieval policy includes routing and inspection; full blocking enforcement is not complete in all paths. ClaimStore compatibility keeps `governance_only` advisory.

## Event provenance limit

EventOrigin and EventEnvelope are expanded, but call-site attribution is partial. Some events still default to `RuntimeLoop` via `append()`.

## NL benchmark limit

Current diagnostic benchmark values remain:

- curated: 15 scenarios
- held_out: 59 scenarios, action_match_rate 0.9152542372881356, failures 5
- adversarial: 2 scenarios
- benchmark tuple: (15 curated, 59 held-out, 2 adversarial)

## Operational limit

This package is a bounded research scaffold. It is not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified.
