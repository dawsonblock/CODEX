# UI Test Report (Packaged Evidence)

## Packaged logs

- Default UI log: `artifacts/proof/verification/ui_tests.log`
  - Result: 76 passed, 0 failed, 6 ignored
- Provider-feature UI log: `artifacts/proof/verification/ui_provider_feature_tests.log`
  - Result: 75 passed, 0 failed, 6 ignored
  - Command represented: `cd ui/codex-dioxus && cargo test --all-targets --features ui-local-providers`

## Known UI warnings

UI tests pass in packaged logs, but warnings remain. These warnings are not test failures and are tracked cleanup work.

## Caveat

Rust/UI results in this report are packaged verification evidence unless rerun in the current environment.
