# Chat UI Integration: Codex Dioxus Shell

## 1. Purpose

This document defines the chatbot-style desktop UI integration for Codex-main 32.

The goal is to provide a conversational shell while keeping runtime authority in CODEX Rust systems.

## 2. Reused from overlooked

The integration reuses UI patterns only:

- Dioxus desktop app boot structure
- Chat-oriented shell layout
- Sidebar-driven navigation style
- Theme and settings panel patterns
- CSS pattern vocabulary for cards, panels, and controls

## 3. Intentionally excluded

The following were intentionally not merged into CODEX runtime authority:

- Provider API execution
- API key storage
- Web search execution
- External autonomous tool execution
- Auth/account logic
- Streaming provider backend

## 4. Chat UI architecture

Chat flow:

User message
-> Chat UI
-> CODEX Runtime Bridge
-> observation routing and bounded action mapping
-> Runtime response envelope
-> assistant bubble + structured metadata

The response envelope includes:

- selected action
- trace metadata (evidence IDs, claim IDs, contradiction IDs, dominant pressures)
- replay-safe status

## 5. Runtime authority boundary

The chat UI is a viewer/controller shell. CODEX runtime remains authoritative.

The UI does not become the runtime brain, and does not replace runtime selection logic.

## 6. Mock/runtime bridge distinction

Bridge modes:

- mock UI mode (active)
- local CODEX runtime mode (disabled until direct crate wiring is enabled)
- external provider mode (disabled)

The mock bridge is explicitly non-authoritative and bounded to the fixed 10-action schema.

## 7. Proof panel

The chat UI shows proof/status cards sourced from proof artifacts:

- simworld summary
- replay report
- evidence integrity report
- NL benchmark report
- long-horizon report
- proof manifest

Missing files are shown as warning/error state; no panic behavior is intended.

## 8. Evidence/claim/audit inspector

Inspector panel surfaces structured metadata for each assistant response:

- selected action
- evidence IDs
- claim IDs
- contradiction IDs
- policy/tool decision metadata (if available)
- missing-evidence reason (if available)

Unavailable fields are explicitly labeled as unavailable in current runtime bridge.

## 9. Operational pressure panel

Operational pressure display is provided as control-state visibility.

Pressure wording remains bounded:

- control signals, not emotions

## 10. Tool/provider limitations

No real external tool execution or provider API execution is enabled in this pass.

Tool requests are policy-gated and rendered as bounded/disabled behavior.

## 11. How to run the UI

From repository root:

```bash
cd ui/codex-dioxus
cargo fmt --all -- --check
cargo check
cargo test
cargo run
```

If `dx` is available:

```bash
dx build
```

## Required boundary statements

The chat UI is a viewer/controller shell. CODEX runtime remains authoritative.

The UI may display chat history, but UI history is not claim memory or evidence memory.

No real external tool execution or provider API execution is enabled in this pass.
