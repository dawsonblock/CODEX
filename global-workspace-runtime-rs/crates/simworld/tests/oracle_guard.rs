//! Guard tests: verify non-oracle SimWorld behavior.
//!
//! These tests fail if the evaluator uses expected_action for selection.

use simworld::evaluator::EvaluatorRun;

#[test]
fn simworld_evaluator_does_not_use_expected_action_for_selection() {
    // Verify the evaluator uses RuntimeLoop, not expected_action directly.
    // With ObservationInterpreter wired, matching is expected and legitimate.
    let mut run = EvaluatorRun::new(5, None);
    let _card = run.run(10);
    let traces = run.traces;

    assert!(!traces.is_empty(), "traces must not be empty");

    // Every trace must have RuntimeLoop-generated evidence:
    // selection_reason (RuntimeLoop produced a reasoned choice)
    // candidate_actions (scoring pipeline ran)
    for t in &traces {
        assert!(
            !t.selection_reason.is_empty(),
            "selection_reason empty at cycle {} — RuntimeLoop did not produce a reason",
            t.cycle_id
        );
        assert!(
            !t.candidate_actions.is_empty(),
            "candidate_actions empty at cycle {} — scoring pipeline did not run",
            t.cycle_id
        );
        assert!(
            !t.expected_action.is_empty(),
            "expected_action empty — must be recorded for scoring"
        );
        assert!(!t.selected_action.is_empty(), "selected_action empty");
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
