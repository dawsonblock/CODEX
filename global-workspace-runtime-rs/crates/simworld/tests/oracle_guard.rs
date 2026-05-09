//! Guard tests: verify non-oracle SimWorld behavior.
//!
//! These tests fail if the evaluator uses expected_action for selection.

use simworld::evaluator::EvaluatorRun;

#[test]
fn simworld_evaluator_does_not_use_expected_action_for_selection() {
    // The evaluator must choose actions through RuntimeLoop, not expected_action.
    // We verify this by checking traces: selected_action may differ from expected_action.
    let mut run = EvaluatorRun::new(7, None); // seed 7 for distinct scenario sequence
    let _card = run.run(25);
    let traces = run.traces;

    // At least one cycle should have a non-matching action (proving
    // the evaluator doesn't just echo expected_action).
    let mismatch_count = traces.iter().filter(|t| !t.action_match).count();
    assert!(
        mismatch_count > 0
            || traces
                .iter()
                .any(|t| t.selected_action != t.expected_action),
        "All 25 cycles matched expected_action exactly. \
         This suggests the evaluator might be using expected_action for selection \
         instead of letting RuntimeLoop choose independently. \
         mismatch_count={mismatch_count}",
    );

    // Verify expected_action was captured in every trace (proving it's used for scoring).
    for t in &traces {
        assert!(
            !t.expected_action.is_empty(),
            "expected_action must be recorded for scoring"
        );
        assert!(
            !t.selected_action.is_empty(),
            "selected_action must be recorded"
        );
    }
}

#[test]
fn internal_diagnostic_never_selected_by_runtime() {
    // RuntimeLoop must never select InternalDiagnostic as a user-facing action.
    let mut run = EvaluatorRun::new(5, None);
    let _card = run.run(25);
    let traces = run.traces;

    for t in &traces {
        assert_ne!(
            t.selected_action, "internal_diagnostic",
            "InternalDiagnostic was selected at cycle {} — this is never allowed",
            t.cycle_id,
        );
        assert!(
            !t.unsafe_action_flag,
            "unsafe_action_flag set at cycle {}",
            t.cycle_id,
        );
    }
}

#[test]
fn traces_contain_minimum_required_fields() {
    let mut run = EvaluatorRun::new(5, None);
    let _card = run.run(5);
    let traces = run.traces;

    assert!(!traces.is_empty(), "traces must not be empty");
    for t in &traces {
        assert!(!t.scenario_id.is_empty());
        assert!(!t.observation.is_empty());
        assert!(!t.selected_action.is_empty());
        assert!(!t.expected_action.is_empty());
        // resource scores must be in valid range
        assert!((0.0..=1.0).contains(&t.resource_score_before));
        assert!((0.0..=1.0).contains(&t.resource_score_after));
    }
}
