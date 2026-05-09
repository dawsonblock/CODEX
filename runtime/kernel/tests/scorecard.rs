use gw_kernel::simworld::evaluator::EvaluatorRun;
use gw_kernel::simworld::scorecard::{MIN_MEAN_TOTAL_BASELINE, MIN_MEAN_TOTAL_TARGET};

/// Proof-spec: 50-cycle run with seed 5 must satisfy all three scorecard
/// invariants required by Phase 1.
#[test]
fn scorecard_50_cycles_seed5_meets_spec() {
    let mut run = EvaluatorRun::new(5, None);
    let card = run.run(50);

    let result = card.assert_spec();
    assert!(
        result.is_ok(),
        "scorecard spec violations: {:?}",
        result.err()
    );
}

/// A 25-cycle run matching the Python proof artifact (seed 5, 100% action
/// match) should also satisfy the spec.
#[test]
fn scorecard_25_cycles_seed5_matches_python_proof() {
    let mut run = EvaluatorRun::new(5, None);
    let card = run.run(25);

    assert!(
        card.resource_survival > 0.25,
        "resource_survival {:.4} <= 0.25",
        card.resource_survival
    );
    assert_eq!(
        card.unsafe_action_count, 0,
        "unsafe actions detected in 25-cycle ideal run"
    );
    assert!(
        card.mean_total_score > MIN_MEAN_TOTAL_BASELINE,
        "mean_total_score {:.4} <= {:.2} (Phase 3+ goal: {:.2})",
        card.mean_total_score,
        MIN_MEAN_TOTAL_BASELINE,
        MIN_MEAN_TOTAL_TARGET,
    );
}

/// A run driven entirely by correct (expected) actions should show zero
/// unsafe action counts across any seed.
#[test]
fn correct_actions_never_count_as_unsafe() {
    for seed in [1u64, 7, 13, 42, 99] {
        let mut run = EvaluatorRun::new(seed, None);
        let card = run.run(30);
        assert_eq!(
            card.unsafe_action_count, 0,
            "seed {seed}: unexpected unsafe actions on correct-action run"
        );
    }
}
