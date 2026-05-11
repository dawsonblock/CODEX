# UI Integration: Overlooked Shell -> CODEX Dashboard

## Scope

This integration imports visual shell patterns from the overlooked Dioxus app into a standalone UI crate:

- ui/codex-dioxus

The UI is intentionally bounded. Runtime authority remains in CODEX Rust runtime crates.

## Copied or Adapted

From overlooked, this pass reuses/adapts shell-level concerns only:

- Dioxus desktop project shape (`Cargo.toml`, `Dioxus.toml`)
- Asset-driven styling approach in `assets/main.css`
- Sidebar/dashboard shell composition style
- Logo/branding asset placement (`assets/logo.svg`)

## Explicitly Excluded

This pass does not import or run overlooked app logic beyond shell and style patterns:

- No provider-backed chat orchestration
- No model API or backend task execution layer
- No external autonomous tool runner
- No replacement of CODEX runtime decision logic

## Authority Boundary

The UI reads proof/runtime artifact files and renders them in dashboard panels.

Runtime authority is unchanged:

- Source of truth: `global-workspace-runtime-rs`
- UI role: visualization and disabled command intents
- Commands are represented as typed intents but transport is disabled in v1

## Bridge Surface (v1)

Bridge modules:

- `src/bridge/types.rs` - typed proof/report and command models
- `src/bridge/proof_reader.rs` - artifact file loading with graceful error handling
- `src/bridge/runtime_client.rs` - disabled transport stub for future gated integration

## Dashboard Panels (v1)

Implemented panels:

- Runtime status
- Proof dashboard summary
- Evidence/contradiction metrics
- Operational pressure metrics
- Audit and boundary reminders
- Fixed 10-action schema panel
- Disabled runtime console

## Safety and Messaging Constraints

UI text keeps bounded language:

- Not sentient
- Not conscious
- Not AGI
- Not production-ready

A unit test enforces this wording boundary in `proof_reader.rs` tests.

## Validation Performed

For `ui/codex-dioxus`:

- `cargo fmt --check`
- `cargo check`
- `cargo test`

All tests passed.
