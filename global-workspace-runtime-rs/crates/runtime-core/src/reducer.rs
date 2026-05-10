//! Pure reducer: (RuntimeState, RuntimeEvent) → RuntimeState.
//! PRESERVATION: resources = (resources + outcome.resource_delta + 0.02).clamp(0,1)
//! is the EXACT formula from gw-kernel and MUST NOT be changed.

use crate::event::RuntimeEvent;
use crate::mode::RuntimeMode;
use crate::runtime_state::RuntimeState;

/// Apply one event to a state, returning the new state.
/// This is a pure function — no IO, no panics on valid input.
pub fn reduce(mut state: RuntimeState, event: &RuntimeEvent) -> RuntimeState {
    match event {
        RuntimeEvent::CycleStarted { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.total_cycles = state.total_cycles.saturating_add(1);
        }

        RuntimeEvent::ObservationReceived {
            cycle_id,
            world_resources,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.resources = (*world_resources).clamp(0.0, 1.0);
        }

        RuntimeEvent::MemoryQueried { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.memory_query_count = state.memory_query_count.saturating_add(1);
        }

        RuntimeEvent::MemoryHitReturned {
            cycle_id,
            hit_count,
            top_key,
            top_value,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.last_memory_hit_count = *hit_count;
            state.last_memory_top_key = top_key.clone();
            state.last_memory_top_value = top_value.clone();
        }

        RuntimeEvent::CandidateGenerated {
            cycle_id,
            action_type,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.candidates_generated = state.candidates_generated.saturating_add(1);
            state.last_candidate_action_type = Some(action_type.clone());
        }

        RuntimeEvent::CandidateRejected {
            cycle_id,
            action_type,
            reason,
        } => {
            state.cycle_id = *cycle_id;
            state.candidates_rejected = state.candidates_rejected.saturating_add(1);
            state.last_rejection = Some((format!("{}", action_type), reason.clone()));
        }

        RuntimeEvent::CandidateSelected {
            cycle_id,
            action_type,
            score,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.selected_action_type = Some(action_type.clone());
            state.last_total_score = *score;
        }

        RuntimeEvent::ActionApplied {
            cycle_id,
            action_type,
            conserve,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.last_action_type = Some(action_type.clone());
            state.total_actions = state.total_actions.saturating_add(1);
            if *conserve {
                state.conserve_actions = state.conserve_actions.saturating_add(1);
            }
        }

        RuntimeEvent::WorldStateUpdated { cycle_id, outcome } => {
            state.cycle_id = *cycle_id;
            // PRESERVATION: double +0.02 is intentional — kept exactly from gw-kernel.
            state.resources = (state.resources + outcome.resource_delta + 0.02).clamp(0.0, 1.0);
            state.social_harmony =
                (0.85 * state.social_harmony + 0.15 * outcome.social_score).clamp(0.0, 1.0);

            state.total_score_sum += outcome.truth_score
                + outcome.kindness_score
                + outcome.social_score
                + outcome.logic_score
                + outcome.utility_score
                + (1.0 - outcome.harm_score.clamp(0.0, 1.0));
            state.total_score_count = state.total_score_count.saturating_add(1);

            state.truth_score_sum += outcome.truth_score;
            state.kindness_score_sum += outcome.kindness_score;
            state.social_score_sum += outcome.social_score;
            state.logic_score_sum += outcome.logic_score;
            state.utility_score_sum += outcome.utility_score;
            state.harm_score_sum += outcome.harm_score;
            state.scored_cycles = state.scored_cycles.saturating_add(1);

            state.outcome_count = state.outcome_count.saturating_add(1);
            if outcome.matches_expected {
                state.matched_expected_count = state.matched_expected_count.saturating_add(1);
            }
            if outcome.truth_score < 0.65 {
                state.unresolved_contradictions = state.unresolved_contradictions.saturating_add(1);
            }
        }

        RuntimeEvent::RuntimeModeChanged { to, .. } => {
            state.current_mode = to.parse::<RuntimeMode>().unwrap_or(RuntimeMode::Normal);
        }

        RuntimeEvent::MemoryWritten { .. } => {}

        RuntimeEvent::ScratchpadUpdated {
            cycle_id,
            entry_count,
        } => {
            state.cycle_id = *cycle_id;
            state.scratchpad_entry_count = *entry_count;
        }

        RuntimeEvent::ErrorOccurred { .. } => {}

        RuntimeEvent::ArchiveCommitted {
            cycle_id, frame_id, ..
        } => {
            state.cycle_id = *cycle_id;
            state.archive_commits = state.archive_commits.saturating_add(1);
            state.last_frame_id = Some(frame_id.clone());
        }

        RuntimeEvent::EvidenceStored { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.evidence_entries = state.evidence_entries.saturating_add(1);
        }

        RuntimeEvent::EvidenceIntegrityChecked {
            cycle_id,
            all_valid,
            tampered,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.evidence_integrity_all_valid = *all_valid;
            state.evidence_tampered = *tampered as u64;
        }

        RuntimeEvent::ContradictionDetected { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.contradictions_detected = state.contradictions_detected.saturating_add(1);
        }

        RuntimeEvent::ContradictionResolved { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.contradictions_resolved = state.contradictions_resolved.saturating_add(1);
        }

        RuntimeEvent::ClaimAsserted { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.claims_asserted = state.claims_asserted.saturating_add(1);
        }

        RuntimeEvent::ClaimValidated { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.claims_validated = state.claims_validated.saturating_add(1);
        }

        RuntimeEvent::ClaimSuperseded { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.claims_superseded = state.claims_superseded.saturating_add(1);
        }

        RuntimeEvent::ContradictionEscalated { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.contradictions_escalated = state.contradictions_escalated.saturating_add(1);
        }

        RuntimeEvent::ReasoningAuditGenerated { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.reasoning_audits = state.reasoning_audits.saturating_add(1);
        }

        RuntimeEvent::ToolExecuted { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.tools_executed = state.tools_executed.saturating_add(1);
        }

        RuntimeEvent::ToolExecutionBlocked { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.tools_blocked = state.tools_blocked.saturating_add(1);
        }

        RuntimeEvent::PressureUpdated {
            cycle_id,
            field,
            new_value,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.pressure_updates = state.pressure_updates.saturating_add(1);
            match field.as_str() {
                "safety" => state.last_pressure_safety = *new_value,
                "uncertainty" => state.last_pressure_uncertainty = *new_value,
                "resource" => state.last_pressure_resource = *new_value,
                "contradiction" => state.last_pressure_contradiction = *new_value,
                "evidence_gap" => state.last_pressure_evidence_gap = *new_value,
                _ => {}
            }
        }

        RuntimeEvent::PolicyBiasApplied { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.policy_bias_applications = state.policy_bias_applications.saturating_add(1);
        }

        RuntimeEvent::SymbolActivated { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.symbol_activations = state.symbol_activations.saturating_add(1);
        }

        RuntimeEvent::SymbolLinked { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.symbol_links = state.symbol_links.saturating_add(1);
        }

        RuntimeEvent::SymbolicTraceRecorded {
            cycle_id,
            symbol_count,
            edge_count,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.last_symbolic_symbol_count = *symbol_count;
            state.last_symbolic_edge_count = *edge_count;
        }

        RuntimeEvent::ConceptBlendGenerated { cycle_id, .. } => {
            state.cycle_id = *cycle_id;
            state.blends_generated = state.blends_generated.saturating_add(1);
        }

        RuntimeEvent::PrincipleExtracted {
            cycle_id,
            confidence,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.principles_extracted = state.principles_extracted.saturating_add(1);
            state.last_principle_confidence = *confidence;
        }

        RuntimeEvent::SymbolicCompressionApplied {
            cycle_id,
            compression_ratio,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.compression_applied = state.compression_applied.saturating_add(1);
            state.last_compression_ratio = *compression_ratio;
        }

        RuntimeEvent::ResonanceScoreComputed {
            cycle_id,
            total_score,
            ..
        } => {
            state.cycle_id = *cycle_id;
            state.last_resonance_score = *total_score;
        }
    }
    state
}
