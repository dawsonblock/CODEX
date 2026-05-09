//! Event log replay and durability proof tests.

use runtime_core::event::{RuntimeEvent, WorldOutcome};
use runtime_core::{ActionType, EventLog};

#[test]
fn event_log_replay_reconstructs_runtime_state() {
    let mut log = EventLog::new();

    log.append(RuntimeEvent::CycleStarted {
        cycle_id: 1,
        timestamp: chrono::Utc::now(),
    })
    .unwrap();
    log.append(RuntimeEvent::CandidateSelected {
        cycle_id: 1,
        action_type: ActionType::Answer,
        score: 0.85,
        resonance: vec![],
        reasoning: Some("test".into()),
    })
    .unwrap();
    log.append(RuntimeEvent::ActionApplied {
        cycle_id: 1,
        action_type: ActionType::Answer,
        conserve: false,
    })
    .unwrap();
    log.append(RuntimeEvent::WorldStateUpdated {
        cycle_id: 1,
        outcome: WorldOutcome {
            resource_delta: 0.03,
            social_score: 0.9,
            harm_score: 0.01,
            truth_score: 0.95,
            kindness_score: 0.8,
            logic_score: 0.85,
            utility_score: 0.75,
            matches_expected: true,
        },
    })
    .unwrap();

    // Replay
    let state = runtime_core::replay_log(&log);
    assert_eq!(state.selected_action_type, Some(ActionType::Answer));
    assert!(state.resources > 0.0);
    assert_eq!(state.matched_expected_count, 1);
}

#[test]
fn replay_is_idempotent() {
    let mut log = EventLog::new();
    log.append(RuntimeEvent::CycleStarted {
        cycle_id: 1,
        timestamp: chrono::Utc::now(),
    })
    .unwrap();
    log.append(RuntimeEvent::CandidateSelected {
        cycle_id: 1,
        action_type: ActionType::AskClarification,
        score: 0.7,
        resonance: vec![],
        reasoning: None,
    })
    .unwrap();

    // Replay twice — must produce identical state
    let state1 = runtime_core::replay_log(&log);
    let state2 = runtime_core::replay_log(&log);
    assert_eq!(state1, state2, "replay must be idempotent");
}

#[test]
fn corrupt_event_log_fails_loudly() {
    // Invalid JSON in event log
    let jsonl = r#"{"type": "CycleStarted", "payload": {"cycle_id": 1, "timestamp": "bad-date"}}"#;
    let result = runtime_core::replay_jsonl(jsonl);
    assert!(result.is_err(), "corrupt JSON must fail, not silently skip");
}
