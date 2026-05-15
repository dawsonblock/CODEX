# STATUS.md

Last updated: 2026-05-15
Codename: CODEX-main 36
Status: CODEX-main 36 hardening candidate (controlled validation/review scope)

CODEX-main 36 is a bounded Rust-authoritative cognitive-runtime scaffold.
It is not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified.

## Fresh checks in this environment

- generated-artifact checks: pass
- pytest: 35 passed
- architecture guard: pass
- proof manifest consistency: pass
- action-types guard: pass
- claim guard: pass
- no-mv2 guard: pass
- resource recovery guard: pass

## Packaged Rust/UI verification evidence (unless rerun)

- Rust tests: 274 passed, 0 failed, 0 ignored
- UI default tests: 76 passed, 0 failed, 6 ignored
- UI provider-feature tests: 75 passed, 0 failed, 6 ignored
- UI warnings remain present in packaged logs

## Current proof metrics

- replay event_count: 589
- replay total_cycles: 15
- long_horizon total_cycles: 150
- long_horizon episodes: 3
- contradictions_detected: 1
- tool_dry_runs: 1
- tools_blocked: 1
- real_external_executions: 0
- local_provider_disabled_blocks: 1

NL benchmark snapshot:
- curated: 15 scenarios, action_match_rate 1.00
- held_out: 59 scenarios, action_match_rate 1.0 (0 failures)
- adversarial: 2 scenarios, action_match_rate 1.00

## Provider/tool boundaries

- local_provider_requests: 0
- cloud_provider_requests: 0
- external_provider_requests: 0
- api_key_storage_enabled: false
- provider_can_execute_tools: false
- provider_can_write_memory: false
- provider_can_override_codex_action: false

## Retrieval/Event/Metadata status

- retrieval policy routing exists; enforcement remains advisory/partial in current code paths
- AnswerBuilder/UI citation metadata is available on primary paths; fallback paths may have empty citation fields
- EventOrigin enum/EventEnvelope expanded; call-site origin attribution remains partial and some events still default to RuntimeLoop
