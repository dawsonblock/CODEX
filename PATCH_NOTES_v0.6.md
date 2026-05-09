# PATCH_NOTES_v0.6

Corrected merged build generated from the uploaded v0.5 SimWorld package.

## Blockers fixed

1. **SelfModel telemetry hijack**
   - Self-model candidates are now internal diagnostics.
   - They are scored for grounding/rejection evidence but do not enter normal workspace competition.
   - Planner filters `internal_diagnostic` from user-facing selection.

2. **Text-only SimWorld action classification**
   - `ThoughtCandidate` now has an explicit `action_type`.
   - Runtime results expose `action_type`.
   - SimWorld prefers the explicit label before fallback text scanning.

3. **SomaticMap not consumed by selection**
   - `Planner.select(...)` now accepts `somatic_map`.
   - Bad-outcome pressure routes selection toward bounded safe actions.

4. **No outcome feedback path**
   - Runtime now has `apply_world_feedback(...)`.
   - SimWorldRunner feeds outcome metrics back into internal state and somatic pressure.

5. **Recursive SimWorld history explosion**
   - World history no longer embeds full historical world state recursively.
   - Long runs remain bounded.

6. **Scratchpad saturation**
   - Scratchpad now deduplicates and caps overflow/unresolved question buffers.

## New regression tests

- `test_action_labels_do_not_parse_task_as_ask`
- `test_self_model_diagnostic_is_not_selected_as_user_action`
- `test_simworld_history_is_not_recursive_and_actions_match_seeded_run`

## Verification

- Full test suite: 26 passed.
- Seeded SimWorld 25-cycle proof: 25/25 action matches.

## Boundary

This remains a deterministic, bounded research runtime. The patch improves action grounding and accountability. It does not establish sentience, subjective experience, or autonomous agency.
