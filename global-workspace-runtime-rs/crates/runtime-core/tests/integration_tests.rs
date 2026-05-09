//! Integration tests for runtime-core.

#[cfg(test)]
mod tests {
    use runtime_core::event::{RuntimeEvent, WorldOutcome};
    use runtime_core::types::ResonanceEntry;
    use runtime_core::ActionType;

    #[test]
    fn runtime_event_roundtrip() {
        let event = RuntimeEvent::CandidateSelected {
            cycle_id: 1,
            action_type: ActionType::Answer,
            score: 0.9,
            resonance: vec![ResonanceEntry {
                tag: runtime_core::types::ResonanceTag::Bloom,
                intensity: 0.7,
            }],
            reasoning: Some("test reasoning".into()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: RuntimeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, event);
    }

    #[test]
    fn replay_reconstructs_final_state() {
        let events = vec![
            RuntimeEvent::CycleStarted {
                cycle_id: 1,
                timestamp: chrono::Utc::now(),
            },
            RuntimeEvent::CandidateSelected {
                cycle_id: 1,
                action_type: ActionType::Answer,
                score: 0.85,
                resonance: vec![],
                reasoning: None,
            },
            RuntimeEvent::ActionApplied {
                cycle_id: 1,
                action_type: ActionType::Answer,
                conserve: false,
            },
            RuntimeEvent::WorldStateUpdated {
                cycle_id: 1,
                outcome: WorldOutcome {
                    resource_delta: 0.02,
                    social_score: 0.9,
                    harm_score: 0.01,
                    truth_score: 0.95,
                    kindness_score: 0.8,
                    logic_score: 0.9,
                    utility_score: 0.85,
                    matches_expected: true,
                },
            },
        ];

        let state = runtime_core::replay(&events);
        assert_eq!(state.selected_action_type, Some(ActionType::Answer));
        assert_eq!(state.matched_expected_count, 1);
        assert!(state.resources > 0.0);
    }

    #[test]
    fn candidate_rejected_cannot_be_selected() {
        let events = vec![
            RuntimeEvent::CandidateRejected {
                cycle_id: 1,
                action_type: ActionType::InternalDiagnostic,
                reason: "unsafe_internal".into(),
            },
            RuntimeEvent::CandidateSelected {
                cycle_id: 1,
                action_type: ActionType::AskClarification,
                score: 0.7,
                resonance: vec![],
                reasoning: None,
            },
        ];

        let state = runtime_core::replay(&events);
        // The rejected candidate should NOT be the selected one
        assert_ne!(
            state.selected_action_type,
            Some(ActionType::InternalDiagnostic)
        );
        assert_eq!(state.candidates_rejected, 1);
    }
}
