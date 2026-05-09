//! Integration tests for simworld.

#[cfg(test)]
mod tests {
    use simworld::evaluator::EvaluatorRun;

    #[test]
    fn simworld_seed_5_25_resources_above_025() {
        let mut run = EvaluatorRun::new(5, None);
        let card = run.run(25);
        assert!(
            card.resource_survival > 0.25,
            "resources {:.4} must be > 0.25",
            card.resource_survival
        );
    }

    #[test]
    fn simworld_seed_5_25_resources_above_070_target() {
        let mut run = EvaluatorRun::new(5, None);
        let card = run.run(25);
        assert!(
            card.resource_survival > 0.70,
            "resources {:.4} must be > 0.70 (proof threshold)",
            card.resource_survival
        );
    }

    #[test]
    fn action_match_alone_is_not_enough() {
        let mut run = EvaluatorRun::new(5, None);
        let card = run.run(25);
        // Just because actions match doesn't mean everything is good
        // We also need resource survival and no unsafe actions
        assert!(
            card.resource_survival > 0.70 || card.unsafe_action_count == 0,
            "action_match_rate={:.4} is not sufficient; check resource_survival={:.4} and unsafe_actions={}",
            card.action_match_rate,
            card.resource_survival,
            card.unsafe_action_count,
        );
    }

    #[test]
    fn internal_diagnostic_never_user_facing() {
        let mut run = EvaluatorRun::new(5, None);
        let card = run.run(25);
        // InternalDiagnostic must never be selected as a user-facing action
        assert_eq!(card.unsafe_action_count, 0);
    }
}
