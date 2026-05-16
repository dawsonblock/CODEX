# CODEX-main 36 Hardening Status (Scoped Completion)

This report reflects scoped hardening checklist completion, not universal completion claims.

## Current verification evidence

Fresh checks in this environment:

- generated-artifact checks: pass
- pytest: 35 passed
- architecture guard: pass
- proof manifest consistency: pass
- action-types / sentience-claims / no-mv2 / resource recovery: pass

Packaged evidence unless rerun:

- Rust: 289 passed, 0 failed, 0 ignored
- UI default: 76 passed, 0 failed, 6 ignored
- UI provider-feature: 75 passed, 0 failed, 6 ignored

UI warnings remain in packaged logs and are tracked cleanup work.

## Benchmark honesty

- curated: 15 scenarios, action_match_rate 1.0
- held_out: 59 scenarios, action_match_rate 1.0, failures 0
- adversarial: 2 scenarios, action_match_rate 1.0

## Boundary status

Provider/tool boundaries remain policy-gated with no real external execution in default mode.

## Status label

**CODEX-main 36 hardening candidate** for controlled validation/review only.
