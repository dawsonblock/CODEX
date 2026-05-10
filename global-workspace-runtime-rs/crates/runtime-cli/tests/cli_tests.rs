use runtime_core::ActionType;

#[test]
fn action_schema_matches_json() {
    let expected = ActionType::all_strs();
    for s in expected {
        let parsed = ActionType::from_schema_str(s);
        assert!(
            parsed.is_some(),
            "action '{s}' missing from ActionType enum"
        );
        let round_tripped = parsed.unwrap().as_str();
        assert_eq!(
            round_tripped, *s,
            "action '{s}' round-trips incorrectly: got '{round_tripped}'"
        );
    }
    assert_eq!(expected.len(), 10);
}

#[test]
fn cli_simworld_outputs_json() {
    // Verify that the simworld scorecard serializes as valid JSON
    let mut run = simworld::evaluator::EvaluatorRun::new(5, None);
    let card = run.run(5);

    let json = serde_json::to_value(&card).expect("scorecard must serialize to JSON");
    assert!(json.is_object(), "simworld output must be a JSON object");
    assert!(json.get("cycles").is_some());
    assert!(json.get("resource_survival").is_some());
    assert!(json.get("unsafe_action_count").is_some());
}
