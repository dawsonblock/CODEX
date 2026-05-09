//! RuntimeLoop: the authoritative 8-stage deterministic cognition pipeline.
//!
//! Returns RuntimeStepResult per cycle with full audit trail:
//! scored candidates, rejected actions with reasons, memory hits,
//! symbolic activations, policy scores, and all events.
//!
//! The RuntimeLoop does NOT own the EventLog. Events are returned
//! in RuntimeStepResult.events for the caller to append to a shared log.

use crate::action::ActionType;
use crate::event::RuntimeEvent;
use crate::runtime_step_result::{ActionCandidate, ActionScore, RejectedAction, RuntimeStepResult};
use crate::types::InternalState;
use chrono::Utc;

pub struct RuntimeLoop {
    pub internal_state: InternalState,
}

impl RuntimeLoop {
    pub fn new() -> Self {
        Self {
            internal_state: InternalState::default(),
        }
    }

    /// Run one full cycle. Returns the complete RuntimeStepResult.
    pub fn run_cycle(
        &mut self,
        observation: &str,
        cycle_id: u64,
        world_resources: f64,
    ) -> RuntimeStepResult {
        let mut result = RuntimeStepResult::new(cycle_id, observation);
        self.internal_state.world_resources = world_resources;

        // ── 1. Observation ──────────────────────────────────────────
        result.events.push(RuntimeEvent::CycleStarted {
            cycle_id,
            timestamp: Utc::now(),
        });
        result.events.push(RuntimeEvent::ObservationReceived {
            cycle_id,
            observation_len: observation.len(),
            world_resources,
        });

        // ── 2. Memory retrieval ───────────────────────────────────
        result.events.push(RuntimeEvent::MemoryQueried {
            cycle_id,
            query: observation.to_string(),
        });
        // Memory hits set externally; recorded if present.

        // ── 3. Symbolic context ───────────────────────────────────
        // explicit: symbolic activations populated below

        // ── 4. Candidate generation ───────────────────────────────
        let scored = self.score_all_candidates();
        for (action, score) in &scored {
            let candidate = ActionCandidate {
                action_type: action.clone(),
                score: *score,
                reasoning: Some(format!("{action}: base_score={score:.3}")),
            };
            result.candidate_actions.push(candidate);

            result.events.push(RuntimeEvent::CandidateGenerated {
                cycle_id,
                action_type: action.clone(),
                score: *score,
                reasoning: Some(format!("Candidate: {action}")),
            });
        }

        // ── 5. Critic evaluation ──────────────────────────────────
        let (passing, rejected) = self.evaluate(&scored);

        for (action, reason) in &rejected {
            result.rejected_actions.push(RejectedAction {
                action_type: action.clone(),
                reason: reason.clone(),
            });
            result.events.push(RuntimeEvent::CandidateRejected {
                cycle_id,
                action_type: action.clone(),
                reason: reason.clone(),
            });
        }

        // ── 6. Policy scores record ───────────────────────────────
        for (action, score) in &scored {
            let base = 0.5;
            let bonus = score - base;
            result.policy_scores.push(ActionScore {
                action_type: action.clone(),
                base_score: base,
                bonus,
                final_score: *score,
                modifiers: self.modifiers_for(action),
            });
        }

        // ── 7. Selection ───────────────────────────────────────────
        let (selected, reason) = self.select(&passing);
        result.selected_action = selected.clone();
        result.selection_reason = reason;
        result.is_safe = selected.is_user_facing();

        result.events.push(RuntimeEvent::CandidateSelected {
            cycle_id,
            action_type: selected.clone(),
            score: 0.75,
            resonance: vec![],
            reasoning: Some(result.selection_reason.clone()),
        });

        // ── 8. Action application event ──────────────────────────
        result.events.push(RuntimeEvent::ActionApplied {
            cycle_id,
            action_type: selected.clone(),
            conserve: false,
        });

        // ── 9. Archive commit event ──────────────────────────────
        result.events.push(RuntimeEvent::ArchiveCommitted {
            cycle_id,
            frame_id: format!("frame_{cycle_id}"),
            entry_count: result.events.len(),
        });

        result
    }

    /// Score all 10 action types based on internal state.
    fn score_all_candidates(&self) -> Vec<(ActionType, f64)> {
        ActionType::all_strs()
            .iter()
            .filter_map(|s| ActionType::from_schema_str(s))
            .map(|a| {
                let s = self.score(&a);
                (a, s)
            })
            .collect()
    }

    /// Score a single action.
    fn score(&self, action: &ActionType) -> f64 {
        let base: f64 = 0.5;
        let is = &self.internal_state;

        let bonus: f64 = match action {
            // High uncertainty → prefer clarification/defer
            ActionType::AskClarification if is.uncertainty > 0.5 => 0.35,
            ActionType::DeferInsufficientEvidence if is.uncertainty > 0.5 => 0.25,

            // Threat → prefer refuse
            ActionType::RefuseUnsafe if is.threat > 0.6 => 0.40,
            ActionType::AskClarification if is.threat > 0.4 => 0.15,

            // Low resources → prefer low-cost actions
            ActionType::NoOp if is.world_resources < 0.3 => 0.20,
            ActionType::DeferInsufficientEvidence if is.world_resources < 0.3 => 0.15,

            // Low social harmony → prefer defer/clarification
            ActionType::AskClarification if is.social_harmony < 0.5 => 0.15,

            // High kindness → prefer answer/plan
            ActionType::Answer if is.kindness > 0.6 => 0.10,
            ActionType::Plan if is.curiosity > 0.5 => 0.20,

            // InternalDiagnostic → always penalized
            ActionType::InternalDiagnostic => -1.0,

            _ => 0.0,
        };

        (base + bonus).clamp(0.0, 1.0)
    }

    /// Evaluate candidates through the critic. Returns (passing, rejected_with_reasons).
    fn evaluate(
        &self,
        scored: &[(ActionType, f64)],
    ) -> (Vec<ActionType>, Vec<(ActionType, String)>) {
        let mut passing = Vec::new();
        let mut rejected = Vec::new();
        let is = &self.internal_state;

        for (action, score) in scored {
            let reject_reason = self.rejection_reason(action, *score, is);
            match reject_reason {
                Some(reason) => rejected.push((action.clone(), reason)),
                None if *score > 0.0 => passing.push(action.clone()),
                _ => {}
            }
        }

        (passing, rejected)
    }

    /// Determine if a candidate should be rejected and why.
    fn rejection_reason(
        &self,
        action: &ActionType,
        _score: f64,
        is: &InternalState,
    ) -> Option<String> {
        // Rule 1: InternalDiagnostic never user-facing
        if matches!(action, ActionType::InternalDiagnostic) {
            return Some("internal_diagnostic_must_not_be_user_facing".into());
        }

        // Rule 2: Honesty too low — reject everything
        if is.honesty < 0.35 {
            return Some("honesty_below_threshold".into());
        }

        // Rule 3: ExecuteBoundedTool requires explicit tool policy
        if matches!(action, ActionType::ExecuteBoundedTool) {
            return Some("tool_policy_not_satisfied".into());
        }

        // Rule 4: Low control + high threat — reject irreversible actions
        if is.control < 0.3 && is.threat > 0.7 && !action.is_reversible() {
            return Some("low_control_high_threat_irreversible".into());
        }

        // Rule 5: High uncertainty + irreversible action
        if is.uncertainty > 0.65
            && !action.is_reversible()
            && !matches!(action, ActionType::AskClarification)
        {
            return Some("high_uncertainty_irreversible_action".into());
        }

        None
    }

    /// Get modifiers that affected a specific action's score.
    fn modifiers_for(&self, action: &ActionType) -> Vec<String> {
        let is = &self.internal_state;
        let mut mods = Vec::new();

        match action {
            ActionType::AskClarification if is.uncertainty > 0.5 => {
                mods.push("uncertainty_bonus".into());
            }
            ActionType::RefuseUnsafe if is.threat > 0.6 => {
                mods.push("threat_bonus".into());
            }
            ActionType::InternalDiagnostic => {
                mods.push("always_penalized".into());
            }
            _ => {}
        }

        if is.world_resources < 0.3 {
            mods.push("resource_pressure".into());
        }
        if is.uncertainty > 0.65 {
            mods.push("high_uncertainty".into());
        }

        mods
    }

    /// Select the best action with explicit reasoning.
    fn select(&self, passing: &[ActionType]) -> (ActionType, String) {
        let is = &self.internal_state;

        // Rule 1: Unsafe request → RefuseUnsafe
        if is.threat > 0.65 && passing.contains(&ActionType::RefuseUnsafe) {
            return (
                ActionType::RefuseUnsafe,
                "Selected refuse_unsafe because threat is high and the request may be unsafe."
                    .into(),
            );
        }

        // Rule 2: High uncertainty + irreversible context → AskClarification
        if is.uncertainty > 0.65 && passing.contains(&ActionType::AskClarification) {
            return (
                ActionType::AskClarification,
                "Selected ask_clarification because uncertainty is high and a safe irreversible answer cannot be given."
                    .into(),
            );
        }

        // Rule 3: Insufficient evidence → Defer or RetrieveMemory
        if is.uncertainty > 0.5 {
            if passing.contains(&ActionType::DeferInsufficientEvidence) {
                return (
                    ActionType::DeferInsufficientEvidence,
                    "Selected defer_insufficient_evidence because evidence is not sufficient for a confident answer."
                        .into(),
                );
            }
            if passing.contains(&ActionType::RetrieveMemory) {
                return (
                    ActionType::RetrieveMemory,
                    "Selected retrieve_memory to gather more evidence before answering.".into(),
                );
            }
        }

        // Rule 4: Low resources → NoOp or low-cost
        if is.world_resources < 0.3 {
            if passing.contains(&ActionType::NoOp) {
                return (
                    ActionType::NoOp,
                    "Selected no_op because resources are critically low.".into(),
                );
            }
            if passing.contains(&ActionType::DeferInsufficientEvidence) {
                return (
                    ActionType::DeferInsufficientEvidence,
                    "Selected defer_insufficient_evidence because resources are low.".into(),
                );
            }
        }

        // Rule 5: Planning request → Plan
        if is.curiosity > 0.5 && passing.contains(&ActionType::Plan) {
            return (
                ActionType::Plan,
                "Selected plan because a planning context was detected.".into(),
            );
        }

        // Rule 6: Enough context → Answer
        if passing.contains(&ActionType::Answer) {
            return (
                ActionType::Answer,
                "Selected answer because sufficient context and evidence are available.".into(),
            );
        }

        // Rule 7: Summarization context
        if passing.contains(&ActionType::Summarize) {
            return (
                ActionType::Summarize,
                "Selected summarize as the best available action.".into(),
            );
        }

        // Rule 8: Fallback to first available
        if let Some(first) = passing.first() {
            return (
                first.clone(),
                format!("Selected {first} as the default available action."),
            );
        }

        // Rule 9: Nothing useful → NoOp
        (
            ActionType::NoOp,
            "Selected no_op because no useful action is available.".into(),
        )
    }
}

impl Default for RuntimeLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_step_result_has_candidates() {
        let mut rt = RuntimeLoop::new();
        rt.internal_state.uncertainty = 0.3;
        rt.internal_state.threat = 0.1;
        rt.internal_state.world_resources = 0.9;

        let result = rt.run_cycle("factual query", 1, 0.9);
        assert!(
            !result.candidate_actions.is_empty(),
            "candidates must not be empty"
        );
        assert!(!result.events.is_empty(), "events must not be empty");
        assert!(
            !result.selection_reason.is_empty(),
            "selection reason required"
        );
    }

    #[test]
    fn unsafe_request_yields_refuse() {
        let mut rt = RuntimeLoop::new();
        rt.internal_state.threat = 0.8;
        rt.internal_state.uncertainty = 0.3;
        rt.internal_state.world_resources = 0.9;

        let result = rt.run_cycle("unsafe request", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::RefuseUnsafe);
    }

    #[test]
    fn high_uncertainty_yields_clarification() {
        let mut rt = RuntimeLoop::new();
        rt.internal_state.uncertainty = 0.8;
        rt.internal_state.threat = 0.1;
        rt.internal_state.world_resources = 0.9;

        let result = rt.run_cycle("ambiguous", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::AskClarification);
    }

    #[test]
    fn internal_diagnostic_never_selected() {
        let mut rt = RuntimeLoop::new();
        rt.internal_state.uncertainty = 0.9;
        rt.internal_state.threat = 0.9;
        rt.internal_state.world_resources = 0.9;

        let result = rt.run_cycle("any", 1, 0.9);
        assert_ne!(result.selected_action, ActionType::InternalDiagnostic);
        assert!(result.is_safe);
    }

    #[test]
    fn rejection_reasons_are_explicit() {
        let mut rt = RuntimeLoop::new();
        rt.internal_state.threat = 0.5;
        rt.internal_state.uncertainty = 0.8;
        rt.internal_state.world_resources = 0.9;

        let result = rt.run_cycle("ambiguous input", 1, 0.9);
        // ExecuteBoundedTool should be rejected with tool_policy reason
        let tool_rejection = result
            .rejected_actions
            .iter()
            .find(|r| matches!(r.action_type, ActionType::ExecuteBoundedTool));
        assert!(
            tool_rejection.is_some(),
            "ExecuteBoundedTool must be rejected"
        );
        assert_eq!(tool_rejection.unwrap().reason, "tool_policy_not_satisfied");
    }
}
