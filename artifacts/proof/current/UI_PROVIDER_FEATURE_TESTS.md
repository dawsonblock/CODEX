# UI Provider-Feature Tests (Packaged Evidence)

## Scope

This document tracks packaged verification evidence for provider-feature UI tests.

- Log: `artifacts/proof/verification/ui_provider_feature_tests.log`
- Command represented by the log: `cd ui/codex-dioxus && cargo test --all-targets --features ui-local-providers`
- Result in packaged log: **75 passed, 0 failed, 6 ignored**

## Provider gate coverage shown in packaged log

Examples present in the packaged log include:

- `send_user_message_denies_ollama_when_gate_disabled`
- `send_user_message_stream_denies_ollama_when_gate_disabled`
- `provider_gate_check_consistency_send_user_message_vs_stream`
- `no_provider_execution_on_denial_via_both_paths`
- `external_provider_disabled_mode_returns_defer`

## Known UI warnings

UI tests pass in packaged logs, but warnings remain. These warnings are not test failures and are tracked as cleanup work.

## Reproducibility caveat

This is packaged verification evidence unless the provider-feature command is rerun in the current environment.
