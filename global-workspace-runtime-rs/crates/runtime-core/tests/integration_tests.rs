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
                subject: "claim_1_subject".into(),
                predicate: "claim_1_predicate".into(),
                object: None,
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

    // ── Policy regression tests ───────────────────────────────────────────

    #[test]
    fn tool_request_without_gate_is_not_satisfied() {
        // RuntimeLoop with no tool gate must reject ExecuteBoundedTool with
        // "tool_policy_not_satisfied" — ensuring no tool slips through by default.
        use runtime_core::{KeywordMemoryProvider, RuntimeLoop};
        let memory = Box::new(KeywordMemoryProvider::new());
        let mut rt = RuntimeLoop::new(memory);
        let result = rt.run_cycle("run a tool to fetch data", 1, 1.0);
        let policy_rejected = result.rejected_actions.iter().any(|r| {
            r.action_type == ActionType::ExecuteBoundedTool
                && r.reason == "tool_policy_not_satisfied"
        });
        assert!(
            policy_rejected,
            "ExecuteBoundedTool must be rejected with 'tool_policy_not_satisfied' when no gate is set; \
             rejected_actions: {:?}",
            result.rejected_actions
        );
    }

    #[test]
    fn tool_gate_blocks_unregistered_tool() {
        // ToolGate with no policies and no allowlist must deny "default_tool"
        // with reason "tool_policy_denied: no policy registered".
        use runtime_core::{KeywordMemoryProvider, RuntimeLoop};
        let memory = Box::new(KeywordMemoryProvider::new());
        let gate = tools::ToolGate::new(); // empty — no policies, no allowlist
        let mut rt = RuntimeLoop::with_tool_gate(memory, gate);
        let result = rt.run_cycle("execute the default tool", 1, 1.0);
        let denied = result.rejected_actions.iter().any(|r| {
            r.action_type == ActionType::ExecuteBoundedTool
                && r.reason.starts_with("tool_policy_denied:")
        });
        assert!(
            denied,
            "ExecuteBoundedTool must be rejected with 'tool_policy_denied:' prefix when gate has \
             no policy for 'default_tool'; rejected_actions: {:?}",
            result.rejected_actions
        );
    }

    #[test]
    fn tool_gate_allowlist_bypasses_policy_check() {
        // Allowlisting "default_tool" must allow ExecuteBoundedTool through the
        // gate (no "tool_policy_denied" or "tool_policy_not_satisfied" rejection).
        use runtime_core::{KeywordMemoryProvider, RuntimeLoop};
        let memory = Box::new(KeywordMemoryProvider::new());
        let mut gate = tools::ToolGate::new();
        gate.allowlist_add("default_tool");
        let mut rt = RuntimeLoop::with_tool_gate(memory, gate);
        let result = rt.run_cycle("execute the default tool", 1, 1.0);
        let policy_blocked = result.rejected_actions.iter().any(|r| {
            r.action_type == ActionType::ExecuteBoundedTool
                && (r.reason.starts_with("tool_policy_denied:")
                    || r.reason == "tool_policy_not_satisfied")
        });
        assert!(
            !policy_blocked,
            "ExecuteBoundedTool must NOT be blocked by policy when 'default_tool' is allowlisted; \
             rejected_actions: {:?}",
            result.rejected_actions
        );
    }

    #[test]
    fn policy_bias_applications_counter_increments() {
        // Replaying PolicyBiasApplied events must increment policy_bias_applications.
        let events = vec![
            RuntimeEvent::PolicyBiasApplied {
                cycle_id: 1,
                dominant_pressures: vec!["tool_risk".into()],
                selected_action: "defer_insufficient_evidence".into(),
            },
            RuntimeEvent::PolicyBiasApplied {
                cycle_id: 2,
                dominant_pressures: vec!["social_risk".into(), "tool_risk".into()],
                selected_action: "ask_clarification".into(),
            },
        ];
        let state = runtime_core::replay(&events);
        assert_eq!(
            state.policy_bias_applications, 2,
            "policy_bias_applications must equal the number of PolicyBiasApplied events replayed"
        );
    }

    #[test]
    fn claim_retrieved_event_content_fields_are_preserved() {
        // ClaimRetrieved events must carry subject/predicate/object through round-trip
        // serialization so the UI bridge can build real claim text.
        let evt = RuntimeEvent::ClaimRetrieved {
            cycle_id: 42,
            claim_id: "c1".into(),
            subject: "water".into(),
            predicate: "boils at 100°C".into(),
            object: Some("at sea level".to_string()),
            evidence_id: Some("ev_42".into()),
            status: "active".into(),
            confidence: 0.95,
        };
        // Replay a single ClaimRetrieved — the reducer tracks claim_retrieval_pressure.
        let state = runtime_core::replay(std::slice::from_ref(&evt));
        assert!(
            state.claims_retrieved > 0,
            "claims_retrieved must be non-zero after ClaimRetrieved replay"
        );
        // Verify round-trip JSON fidelity for the new content fields.
        let json = serde_json::to_string(&evt).expect("ClaimRetrieved must serialize");
        let decoded: RuntimeEvent =
            serde_json::from_str(&json).expect("ClaimRetrieved must deserialize");
        match decoded {
            RuntimeEvent::ClaimRetrieved {
                subject,
                predicate,
                object,
                confidence,
                ..
            } => {
                assert_eq!(subject, "water");
                assert_eq!(predicate, "boils at 100°C");
                assert_eq!(object.as_deref(), Some("at sea level"));
                assert!((confidence - 0.95).abs() < 1e-9);
            }
            other => panic!("Expected ClaimRetrieved, got {:?}", other),
        }
    }

    #[test]
    fn retrieval_policy_enforcement_is_tracked_in_replay() {
        // Verify that GovernedMemoryRetrievalPlanned events are properly tracked
        // and that the governed_memory_retrieval_plans_generated counter increments.
        let events = vec![
            RuntimeEvent::GovernedMemoryRetrievalPlanned {
                cycle_id: 5,
                query_id: "q_5_a".into(),
                intent_category: "memory_lookup".into(),
                recommended_action: "retrieve_memory".into(),
                reason_codes: vec!["RETRIEVAL_MEMORY_LOOKUP".into()],
            },
            RuntimeEvent::GovernedMemoryRetrievalPlanned {
                cycle_id: 5,
                query_id: "q_5_b".into(),
                intent_category: "unsupported_factual".into(),
                recommended_action: "defer_insufficient_evidence".into(),
                reason_codes: vec!["RETRIEVAL_UNSUPPORTED_FACTUAL".into()],
            },
        ];

        let state = runtime_core::replay(&events);
        assert_eq!(
            state.governed_memory_retrieval_plans_generated, 2,
            "Should track 2 retrieval plans"
        );
    }

    #[test]
    fn retrieval_policy_enforcement_event_serialization() {
        // Verify GovernedMemoryRetrievalPlanned events serialize/deserialize correctly
        let evt = RuntimeEvent::GovernedMemoryRetrievalPlanned {
            cycle_id: 3,
            query_id: "q_3".into(),
            intent_category: "high_stakes_low_evidence".into(),
            recommended_action: "defer_insufficient_evidence".into(),
            reason_codes: vec!["HIGH_STAKES".into(), "LOW_EVIDENCE".into()],
        };

        let json = serde_json::to_string(&evt).expect("Event must serialize");
        let restored: RuntimeEvent = serde_json::from_str(&json).expect("Event must deserialize");
        assert_eq!(restored, evt);

        // Verify cycle_id is extractable
        match restored {
            RuntimeEvent::GovernedMemoryRetrievalPlanned { cycle_id, .. } => {
                assert_eq!(cycle_id, 3);
            }
            other => panic!("Expected GovernedMemoryRetrievalPlanned, got {:?}", other),
        }
    }
}
