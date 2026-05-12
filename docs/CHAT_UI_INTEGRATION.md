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

## 3. Intentionally excluded (all build configurations)

The following are excluded from all builds:

- Cloud Provider API execution (OpenAI, Anthropic, etc.)
- API key storage
- Web search execution
- External autonomous tool execution
- Auth/account logic
- Remote streaming provider backend
- Real external tool execution (real_external_executions: 0)

## 3b. Experimental local provider support (requires `ui-local-providers` Cargo feature)

Local provider execution (Ollama/Turboquant via `localhost:11434`) is **not included in default builds**.
To enable it, build explicitly with:

```
cargo build --features ui-local-providers
```

When the feature is active, the following restrictions apply:
- Calls are **localhost-only** (no external/cloud endpoints).
- First use requires **explicit user approval** (gate must be unlocked in Settings).
- Provider output is labeled `"Local provider draft — not CODEX runtime authority"`.
- Provider output **cannot**: execute tools, write evidence/claims, or override `selected_action`.
- Failure returns a clean UI error; no silent fallback to another provider.
- Cloud and external provider request counts must remain 0.


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
- local CODEX runtime mode (read-only)
- experimental local Ollama provider (localhost:11434)
- experimental local Turboquant provider (localhost:11434)
- external cloud provider mode (disabled)

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

External cloud provider API execution is strictly disabled.

Experimental LOCAL provider execution (Ollama/Turboquant) is enabled for developer testing only. This mode is:
- Gated behind a "Provider Security Gate" in Settings.
- Strictly non-authoritative; it does NOT represent the CODEX runtime selection.
- Monitored via local provider execution counters.

Tool requests remain policy-gated and are rendered as bounded/disabled "Dry Run" behaviors. Real external tool execution is NOT enabled.

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

No real external tool execution or external cloud provider API execution is enabled in this pass. Local provider execution (localhost:11434) is experimental, non-authoritative, and security-gated.

This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient, not conscious, not AGI, not production-ready, not a safe autonomous external tool executor, and not a complete evidence-grounded cognitive agent.

