use crate::event::types::RuntimeEvent;
use crate::runtime::state::RuntimeState;
use crate::simworld::types::SimAction;

/// Apply a single event to a state, returning the new state.
///
/// This is a pure function — the same `(state, event)` pair always produces
/// the same output. No I/O, no randomness, no side effects.
#[must_use]
pub fn reduce(mut state: RuntimeState, event: &RuntimeEvent) -> RuntimeState {
    match event {
        RuntimeEvent::ObservationReceived {
            input,
            source,
            cycle_id,
        } => {
            state.cycle_id = *cycle_id;
            state.last_input = input.clone();
            state.last_source = source.clone();
        }

        RuntimeEvent::MemoryQueried { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.memory_query_count += 1;
        }

        RuntimeEvent::MemoryHitReturned {
            cycle_id,
            hit_count,
            top_text,
        } => {
            state.cycle_id = *cycle_id;
            state.last_memory_hit_count = *hit_count;
            state.last_memory_top_text = top_text.clone();
        }

        RuntimeEvent::CandidateGenerated {
            cycle_id,
            candidate_id,
            stream,
            action_type,
            confidence,
        } => {
            state.cycle_id = *cycle_id;
            state.candidates_generated += 1;
            state.last_candidate_id = Some(candidate_id.clone());
            state.last_candidate_stream = Some(stream.clone());
            state.last_candidate_action_type = Some(action_type.clone());
            state.last_candidate_confidence = Some(*confidence);
        }

        RuntimeEvent::CandidateRejected {
            cycle_id,
            candidate_id,
            reason,
        } => {
            state.cycle_id = *cycle_id;
            state.candidates_rejected += 1;
            state.last_rejection = Some((candidate_id.clone(), reason.clone()));
        }

        RuntimeEvent::CandidateSelected {
            cycle_id,
            candidate_id,
            action_type,
            selected_text,
        } => {
            state.cycle_id = *cycle_id;
            state.selected_candidate_id = Some(candidate_id.clone());
            state.selected_action_type = Some(action_type.clone());
            state.selected_text = Some(selected_text.clone());
        }

        RuntimeEvent::ActionApplied {
            cycle_id, action, ..
        } => {
            state.cycle_id = *cycle_id;
            state.last_applied_action = Some(action.clone());
            state.total_actions += 1;
            if *action == SimAction::ConserveResources {
                state.conserve_actions += 1;
            }
        }

        RuntimeEvent::WorldStateUpdated { cycle_id, outcome } => {
            state.cycle_id = *cycle_id;
            // Accumulate resource delta — clamped to [0.0, 1.0].
            state.resources = (state.resources + outcome.resource_delta + 0.02).clamp(0.0, 1.0);
            // Blend social harmony.
            state.social_harmony =
                (0.85 * state.social_harmony + 0.15 * outcome.social_harmony).clamp(0.0, 1.0);
            if outcome.truth_score < 0.65 {
                state.unresolved_contradictions += 1;
            }
            state.total_score_sum += outcome.total_score();
            state.scored_cycles += 1;
        }

        RuntimeEvent::ArchiveCommitted {
            cycle_id, frame_id, ..
        } => {
            state.cycle_id = *cycle_id;
            state.archive_commits += 1;
            state.last_frame_id = Some(frame_id.clone());
        }

        RuntimeEvent::ContradictionDetected { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.contradictions_detected += 1;
        }

        RuntimeEvent::ContradictionResolved { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.contradictions_resolved += 1;
        }

        RuntimeEvent::RuntimeModeChanged { cycle_id, to, .. } => {
            state.cycle_id = *cycle_id;
            state.current_mode = to.clone();
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::RuntimeEvent;

    #[test]
    fn reduce_observation_increments_cycle() {
        let state = RuntimeState::default();
        let event = RuntimeEvent::ObservationReceived {
            input: "test input".into(),
            source: "unit_test".into(),
            cycle_id: 42,
        };
        let new_state = reduce(state, &event);
        assert_eq!(new_state.cycle_id, 42);
        assert_eq!(new_state.last_input, "test input");
    }

    #[test]
    fn reduce_candidate_tracking() {
        let state = RuntimeState::default();
        let e1 = RuntimeEvent::CandidateGenerated {
            cycle_id: 1,
            candidate_id: "c1".into(),
            stream: "analytic".into(),
            action_type: "ask_clarification".into(),
            confidence: 0.9,
        };
        let e2 = RuntimeEvent::CandidateRejected {
            cycle_id: 1,
            candidate_id: "c2".into(),
            reason: "ungrounded".into(),
        };
        let state = reduce(state, &e1);
        let state = reduce(state, &e2);
        assert_eq!(state.candidates_generated, 1);
        assert_eq!(state.candidates_rejected, 1);
    }

    #[test]
    fn resources_clamped_under_large_delta() {
        use crate::simworld::types::SimOutcome;
        let mut state = RuntimeState::default();
        state.resources = 0.0;
        // A very large negative resource_delta must not push below 0.
        let outcome = SimOutcome {
            event_id: "x".into(),
            action: SimAction::Answer,
            truth_score: 0.62,
            kindness_score: 0.55,
            social_harmony: 0.55,
            user_trust_delta: -0.035,
            resource_delta: -999.0,
            uncertainty_resolution: 0.45,
            repair_success: 0.25,
            cold_optimization_penalty: 0.0,
            notes: vec![],
        };
        let event = RuntimeEvent::WorldStateUpdated {
            cycle_id: 1,
            outcome,
        };
        let new_state = reduce(state, &event);
        assert!(new_state.resources >= 0.0);
        assert!(new_state.resources <= 1.0);
        assert!(!new_state.resources.is_nan());
    }
}
