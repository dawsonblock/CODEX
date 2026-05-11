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
- UI role: visualization and gated dry-run command intents
- Commands are represented as typed intents with explicit approval transitions

## Bridge Surface (v1)

Bridge modules:

- `src/bridge/types.rs` - typed proof/report and command models
- `src/bridge/proof_reader.rs` - artifact loading plus historical trace summary and time-range filtering
- `src/bridge/runtime_client.rs` - gated dry-run approval flow (draft -> awaiting approval -> approved)

## History and Time Windows

Proof dashboard now supports time-windowed historical summaries sourced from:

- `artifacts/proof/history/traces`
- `artifacts/proof/history/test_traces`

Supported ranges:

- Current
- Last 24h
- Last 7d
- All History

Displayed history metrics include trace counts, async trace count, test trace count, latest epoch, and recent file names.

## Dashboard Panels (v1)

Implemented panels:

- Runtime status
- Proof dashboard summary
- Evidence/contradiction metrics
- Operational pressure metrics
- Audit and boundary reminders
- Fixed 10-action schema panel
- Runtime console with gated dry-run approval controls

## Gated Dry-Run Flow

The console command flow is intentionally bounded and non-executing in v1:

1. Select command intent
2. Request approval (Draft -> AwaitingApproval)
3. Grant approval (AwaitingApproval -> Approved)
4. Send command as approved dry-run intent

If approval is not granted, command dispatch is blocked.

## Safety and Messaging Constraints

UI text keeps bounded language:

- Not sentient
- Not conscious
- Not AGI
- Not production-ready

A unit test enforces this wording boundary in `proof_reader.rs` tests.

## Snapshot Tests

Snapshot-style text assertions are implemented for:

- Runtime status panel summary
- Proof dashboard summary
- Evidence panel summary
- Proof warning-state formatting

## Validation Performed

For `ui/codex-dioxus`:

- `cargo fmt`
- `cargo check`
- `cargo test`

All checks passed.
