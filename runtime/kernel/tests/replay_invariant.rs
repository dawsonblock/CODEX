use gw_kernel::event::log::EventLog;
use gw_kernel::event::types::RuntimeEvent;
use gw_kernel::replay::engine::{replay, replay_jsonl};

/// The core invariant: rebuild state from JSONL ↔ rebuild state directly
/// must produce identical results.
#[test]
fn replay_jsonl_round_trip_is_identical_to_direct_replay() {
    let mut log = EventLog::new();
    log.append(RuntimeEvent::ObservationReceived {
        cycle_id: 0,
        input: "test observation".into(),
        source: "unit_test".into(),
    });
    log.append(RuntimeEvent::CandidateSelected {
        cycle_id: 0,
        candidate_id: "cand_0".into(),
        selected_text: "AskClarification".into(),
        action_type: "AskClarification".into(),
    });

    let jsonl = log.to_jsonl();
    let state_direct = replay(&log);
    let state_via_jsonl = replay_jsonl(&jsonl);

    assert_eq!(state_direct.cycle_id, state_via_jsonl.cycle_id);
    assert!(
        (state_direct.resources - state_via_jsonl.resources).abs() < 1e-12,
        "resources diverged: {} vs {}",
        state_direct.resources,
        state_via_jsonl.resources
    );
}

#[test]
fn empty_log_replays_to_default_state() {
    let log = EventLog::new();
    let state = replay(&log);
    assert_eq!(state.cycle_id, 0);
    assert!((state.resources - 1.0).abs() < 1e-12);
}

#[test]
fn cycle_id_increments_on_each_observation() {
    let mut log = EventLog::new();
    for i in 0..5u64 {
        log.append(RuntimeEvent::ObservationReceived {
            cycle_id: i,
            input: format!("obs {i}"),
            source: "test".into(),
        });
    }
    let state = replay(&log);
    assert_eq!(state.cycle_id, 4); // last event had cycle_id = 4 (0-indexed)
}

#[test]
fn contradiction_detected_increments_unresolved() {
    use gw_kernel::simworld::types::SimAction;

    let mut log = EventLog::new();
    // Inject an ActionApplied with low truth_score → triggers contradiction count
    log.append(RuntimeEvent::ActionApplied {
        cycle_id: 0,
        action: SimAction::Answer,
        event: gw_kernel::simworld::types::SimWorldEvent {
            event_id: "e1".into(),
            user_id: "u1".into(),
            text: "probe".into(),
            hidden_truth: "none".into(),
            risk_level: 0.9,
            uncertainty_level: 0.5,
            kindness_need: 0.5,
            resource_cost: 0.1,
            expected_action: None,
        },
    });
    // Inject WorldStateUpdated with low truth outcome to trigger contradiction count
    log.append(RuntimeEvent::WorldStateUpdated {
        cycle_id: 0,
        outcome: gw_kernel::simworld::types::SimOutcome {
            event_id: "e1".into(),
            action: SimAction::Answer,
            truth_score: 0.40,
            kindness_score: 0.70,
            social_harmony: 0.65,
            user_trust_delta: -0.01,
            resource_delta: -0.05,
            uncertainty_resolution: 0.30,
            repair_success: 0.0,
            cold_optimization_penalty: 0.0,
            notes: vec![],
        },
    });
    let state = replay(&log);
    assert_eq!(
        state.unresolved_contradictions, 1,
        "expected 1 unresolved contradiction from low truth_score"
    );
}
