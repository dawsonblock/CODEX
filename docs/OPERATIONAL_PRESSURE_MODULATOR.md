# Operational Pressure Modulator

## Purpose

The Operational Pressure Modulator tracks deterministic control signals used to bias action scoring under uncertainty, risk, contradiction, resource pressure, and evidence gaps.

## What it is

- A numeric vector of 9 pressure fields, each clamped 0.0–1.0
- A deterministic mapping from pressure state to action-level policy bias
- Replayable via `PressureUpdated` and `PolicyBiasApplied` runtime events
- Visible in TUI text output and exportable as a DeepSeek context block

## What it is not

- The Operational Pressure Modulator does **not** model subjective emotions.
- It does **not** represent feelings, wants, or emotional awareness.
- The system is **not** sentient, conscious, or self-aware.
- Pressure values are engineering control signals, derived from runtime state.

## Pressure fields

| Field | Interpretation |
|---|---|
| uncertainty_pressure | High when the runtime lacks enough context |
| contradiction_pressure | High when active claims conflict |
| safety_pressure | High when unsafe/refusal behavior may be needed |
| resource_pressure | High when conservative action is preferred |
| social_risk_pressure | High when user-facing response should avoid escalation |
| tool_risk_pressure | High when tool execution is risky or under-approved |
| evidence_gap_pressure | High when answer lacks evidence support |
| urgency_pressure | High when input suggests immediate action is needed |
| coherence_pressure | High when runtime state is unstable or inconsistent |

## Policy bias mapping

Safety pressure always overrides urgency. Key mappings:
- High safety → boost refuse_unsafe, suppress execute_bounded_tool
- High evidence_gap → boost retrieve_memory, suppress answer
- High uncertainty → boost ask_clarification
- High resource → boost no_op, suppress execute_bounded_tool
- High tool_risk → suppress execute_bounded_tool, boost plan

## Replay/event integration

- `RuntimeEvent::PressureUpdated` — per-field pressure change
- `RuntimeEvent::PolicyBiasApplied` — per-cycle bias application
- Counters: `pressure_updates`, `policy_bias_applications` in replay state

## TUI display

Text-based bar chart rendering via `PressureTuiView::render_text()`.
Panel title: "Operational Pressure Modulator" with boundary note.

## DeepSeek context block

Exportable via `PressureTuiView::to_deepseek_context_block()`.
Format: `[CODEX OPERATIONAL PRESSURE STATE]...[/CODEX OPERATIONAL PRESSURE STATE]`
Includes boundary warning: "These values are deterministic runtime control signals, not emotions."

## Test coverage

15 tests: clamping, decay, safety boost, evidence gap, uncertainty, resource, tool risk, safety-over-urgency, TUI render fields, TUI boundary, DeepSeek fields, DeepSeek boundary, DeepSeek no-emotion, dominant sorting, max pressure.

## Limitations

- Pressure bias is applied to RuntimeLoop action scoring through `run_cycle_with_bias()` in both standard and NL SimWorld evaluation.
- Replay tracks pressure update/application counters; full pressure-state vector reconstruction is not yet implemented.
- No persistent pressure history beyond replay counters.
- TUI is text-only, no interactive panel