# Current Proof Summary

Regenerated proof artifacts: packaged current set under `artifacts/proof/current`.

## Identity and limits

- Codename: **CODEX-main 36 hardening candidate**
- This is a bounded research scaffold.
- Not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified.

## Fresh checks in this environment

- `pytest`: 35 passed
- architecture guard: pass
- proof manifest consistency: pass
- action-types / claim guard / no-mv2 / resource recovery: pass
- generated-artifact checks (before and after): pass

## Rust/UI verification evidence

- Rust checks rerun in this environment: cargo fmt --check, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo test --workspace --all-targets --all-features
- Rust test result: 274 passed, 0 failed, 0 ignored
- UI default packaged log: 76 passed, 0 failed, 6 ignored
- UI provider-feature packaged log: 75 passed, 0 failed, 6 ignored
- Caveat: UI values are packaged evidence unless rerun locally

## Core proof values

- replay event_count: 589
- replay total_cycles: 15
- long_horizon total_cycles: 150
- long_horizon episodes: 3
- contradictions_detected: 1
- tool_dry_runs: 1
- tools_blocked: 1
- real_external_executions: 0
- local_provider_disabled_blocks: 1

## NL benchmark snapshot

- curated: 15 scenarios, action_match_rate 1.0
- held_out: 59 scenarios, action_match_rate 1.0 (0 failures)
- adversarial: 2 scenarios, action_match_rate 1.0

## Boundary status

- local_provider_requests: 0
- cloud_provider_requests: 0
- external_provider_requests: 0
- provider_can_execute_tools: false
- provider_can_write_memory: false
- provider_can_override_codex_action: false
- api_key_storage_enabled: false
