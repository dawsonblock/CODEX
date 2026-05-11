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

    #[test]
    fn replay_tracks_claim_retrieval_pressure_and_audit_references() {
        let events = vec![
            RuntimeEvent::ClaimRetrieved {
                cycle_id: 7,
                claim_id: "claim_1".into(),
                evidence_id: Some("ev_1".into()),
                status: "active".into(),
                confidence: 0.8,
            },
            RuntimeEvent::ContradictionChecked {
                cycle_id: 7,
                checked_claim_ids: vec!["claim_1".into()],
                contradiction_ids: vec!["contra_1".into()],
                active_contradictions: 1,
            },
            RuntimeEvent::PressureUpdated {
                cycle_id: 7,
                field: "tool_risk".into(),
                old_value: 0.0,
                new_value: 0.6,
                source: "ToolPolicy".into(),
                reason: "denied tool request".into(),
            },
            RuntimeEvent::PressureUpdated {
                cycle_id: 7,
                field: "social_risk".into(),
                old_value: 0.0,
                new_value: 0.3,
                source: "Observation".into(),
                reason: "sensitive context".into(),
            },
            RuntimeEvent::ReasoningAuditGenerated {
                cycle_id: 7,
                audit_id: "audit_7".into(),
                selected_action: ActionType::DeferInsufficientEvidence.to_string(),
                evidence_ids: vec!["ev_1".into()],
                claim_ids: vec!["claim_1".into()],
                contradiction_ids: vec!["contra_1".into()],
                dominant_pressures: vec!["tool_risk".into()],
                audit_text: "bounded audit".into(),
            },
        ];

        let state = runtime_core::replay(&events);
        assert_eq!(state.claims_retrieved, 1);
        assert_eq!(state.claims_with_evidence_links, 1);
        assert_eq!(state.contradictions_checked, 1);
        assert_eq!(state.unresolved_contradictions, 1);
        assert_eq!(state.last_pressure_tool_risk, 0.6);
        assert_eq!(state.last_pressure_social_risk, 0.3);
        assert_eq!(state.audits_with_evidence_refs, 1);
        assert_eq!(state.audits_with_claim_refs, 1);
    }
}
