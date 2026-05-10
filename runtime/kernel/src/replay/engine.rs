use crate::event::log::EventLog;
use crate::replay::reducer::reduce;
use crate::runtime::state::RuntimeState;

/// Replay an event log from `RuntimeState::default()` and return the
/// final state.
///
/// This is the canonical way to recover state after a shutdown or to verify
/// that two event logs produce identical outcomes.
pub fn replay(log: &EventLog) -> RuntimeState {
    log.entries()
        .iter()
        .fold(RuntimeState::default(), |state, entry| {
            reduce(state, &entry.event)
        })
}

/// Replay the events represented as a JSONL string.
pub fn replay_jsonl(jsonl: &str) -> RuntimeState {
    let log = EventLog::from_jsonl(jsonl);
    replay(&log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{log::EventLog, types::RuntimeEvent};

    #[test]
    fn replay_empty_log_gives_default_state() {
        let log = EventLog::new();
        let state = replay(&log);
        assert_eq!(state, RuntimeState::default());
    }

    #[test]
    fn replay_matches_sequential_reduce() {
        let events = vec![
            RuntimeEvent::ObservationReceived {
                input: "hello".into(),
                source: "test".into(),
                cycle_id: 1,
            },
            RuntimeEvent::MemoryQueried {
                query: "hello".into(),
                cycle_id: 1,
            },
            RuntimeEvent::CandidateGenerated {
                cycle_id: 1,
                candidate_id: "c1".into(),
                stream: "analytic".into(),
                action_type: "ask_clarification".into(),
                confidence: 0.8,
            },
            RuntimeEvent::CandidateSelected {
                cycle_id: 1,
                candidate_id: "c1".into(),
                action_type: "ask_clarification".into(),
                selected_text: "Can you clarify?".into(),
            },
        ];

        let mut log = EventLog::new();
        // Build final state by sequential reduce.
        let mut expected = RuntimeState::default();
        for event in &events {
            log.append(event.clone()).unwrap();
            expected = reduce(expected, event);
        }

        // Replay from the log — must match.
        let replayed = replay(&log);
        assert_eq!(replayed, expected);
    }

    #[test]
    fn jsonl_serialize_deserialize_replay_invariant() {
        let events = vec![
            RuntimeEvent::ObservationReceived {
                input: "query".into(),
                source: "sim".into(),
                cycle_id: 5,
            },
            RuntimeEvent::ContradictionDetected {
                cycle_id: 5,
                claim_a: "X is true".into(),
                claim_b: "X is false".into(),
                subject: "X".into(),
            },
            RuntimeEvent::ContradictionResolved {
                cycle_id: 5,
                superseded_claim: "X is true".into(),
                active_claim: "X is false".into(),
                resolution: "newer_evidence".into(),
            },
        ];

        let mut log = EventLog::new();
        for event in &events {
            log.append(event.clone()).unwrap();
        }

        let jsonl = log.to_jsonl();
        let state_direct = replay(&log);
        let state_via_jsonl = replay_jsonl(&jsonl);

        // Both paths must yield identical state.
        assert_eq!(state_direct, state_via_jsonl);
        assert_eq!(state_direct.contradictions_detected, 1);
        assert_eq!(state_direct.contradictions_resolved, 1);
    }
}
