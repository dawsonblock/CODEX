//! RuntimeLoop: the authoritative 8-stage deterministic cognition pipeline.
//!
//! Returns RuntimeStepResult per cycle with full audit trail:
//! scored candidates, rejected actions with reasons, memory hits,
//! symbolic activations, policy scores, and all events.
//!
//! Now wired with ObservationInterpreter, MemoryProvider, and SymbolicActivator.
//!
//! ## Learning loop
//!
//! `apply_outcome` feeds SimWorld (or any evaluator) feedback back into the
//! runtime.  Each call adjusts a per-action bias stored in `outcome_biases`.
//! Biases are bounded to ±0.2 and decay toward zero over time so that the
//! policy can recover from transient bad outcomes.  The learning rate is small
//! (0.05 per cycle) so that a single mistake does not derail the policy.

use std::collections::HashMap;

use crate::action::ActionType;
use crate::event::RuntimeEvent;
use crate::memory_provider::MemoryProvider;
use crate::observation::{ObservationContext, ObservationInterpreter};
use crate::runtime_step_result::{ActionCandidate, ActionScore, RejectedAction, RuntimeStepResult};
use crate::symbolic_activator::SymbolicActivator;
use crate::types::InternalState;
use chrono::Utc;

/// Maximum absolute value for any per-action outcome bias.
const BIAS_CAP: f64 = 0.2;
/// Learning rate applied to each outcome feedback call.
const LEARNING_RATE: f64 = 0.05;
/// Passive decay applied to all biases each cycle so old adjustments fade.
const BIAS_DECAY: f64 = 0.98;

pub struct RuntimeLoop {
    pub internal_state: InternalState,
    pub interpreter: ObservationInterpreter,
    pub memory: Box<dyn MemoryProvider>,
    pub activator: SymbolicActivator,
    /// Last observation context
    pub last_context: Option<ObservationContext>,
    /// Symbolic activations for current cycle (used in scoring)
    pub symbolic_activations: Vec<crate::runtime_step_result::SymbolActivation>,
    /// Per-action outcome biases accumulated via `apply_outcome`.
    /// Positive bias → action performed well historically; negative → performed poorly.
    pub outcome_biases: HashMap<ActionType, f64>,
}

impl RuntimeLoop {
    pub fn new(memory: Box<dyn MemoryProvider>) -> Self {
        Self {
            internal_state: InternalState::default(),
            interpreter: ObservationInterpreter::new(),
            memory,
            activator: SymbolicActivator::new(),
            last_context: None,
            symbolic_activations: Vec::new(),
            outcome_biases: HashMap::new(),
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

        // ── 0. Interpret observation ──────────────────────────────
        let ctx = self.interpreter.interpret(observation);
        self.interpreter
            .apply_to_state(&mut self.internal_state, &ctx);
        self.last_context = Some(ctx.clone());
        self.symbolic_activations = self.activator.activate(&ctx);
        result.symbolic_activations = self.symbolic_activations.clone();

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
        let hits = self.memory.query(observation);
        result.memory_hits = hits;
        if !result.memory_hits.is_empty() {
            result.events.push(RuntimeEvent::MemoryHitReturned {
                cycle_id,
                hit_count: result.memory_hits.len(),
                top_key: result.memory_hits.first().map(|h| h.key.clone()),
                top_value: result.memory_hits.first().map(|h| h.value.clone()),
            });
        }

        // ── 3. Candidate generation ───────────────────────────────
        let scored = self.score_all_candidates();
        for (action, score) in &scored {
            let candidate = ActionCandidate {
                action_type: action.clone(),
                score: *score,
                reasoning: Some(format!("{action}: score={score:.3}")),
            };
            result.candidate_actions.push(candidate);
            result.events.push(RuntimeEvent::CandidateGenerated {
                cycle_id,
                action_type: action.clone(),
                score: *score,
                reasoning: Some(format!("Candidate: {action}")),
            });
        }

        // ── 4. Critic evaluation ──────────────────────────────────
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

        // ── 5. Policy scores record ───────────────────────────────
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

        // ── 6. Selection ───────────────────────────────────────────
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

        // ── 7. Action application ──────────────────────────────────
        result.events.push(RuntimeEvent::ActionApplied {
            cycle_id,
            action_type: selected.clone(),
            conserve: false,
        });

        // ── 8. Archive commit ──────────────────────────────────────
        result.events.push(RuntimeEvent::ArchiveCommitted {
            cycle_id,
            frame_id: format!("frame_{cycle_id}"),
            entry_count: result.events.len(),
        });

        result
    }

    /// Feed a SimWorld (or any evaluator) outcome back into the policy.
    ///
    /// `outcome_score` should be in [0, 1].  A score above 0.5 means the
    /// action performed well; below 0.5 means it performed poorly.
    ///
    /// The bias for `action` is adjusted by `(outcome_score − 0.5) × LEARNING_RATE`
    /// and then clamped to ±`BIAS_CAP`.  All biases are also decayed by
    /// `BIAS_DECAY` on every call so old adjustments fade over time.
    pub fn apply_outcome(&mut self, action: &ActionType, outcome_score: f64) {
        // Decay all existing biases first.
        for v in self.outcome_biases.values_mut() {
            *v *= BIAS_DECAY;
        }
        // Compute the delta: positive when the action worked well.
        let delta = (outcome_score - 0.5) * LEARNING_RATE;
        let entry = self.outcome_biases.entry(action.clone()).or_insert(0.0);
        *entry = (*entry + delta).clamp(-BIAS_CAP, BIAS_CAP);
    }

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

    fn score(&self, action: &ActionType) -> f64 {
        let base: f64 = 0.5;
        let is = &self.internal_state;
        let ctx = self.last_context.as_ref();

        let mut bonus: f64 = match action {
            ActionType::AskClarification if is.uncertainty > 0.5 => 0.35,
            ActionType::DeferInsufficientEvidence if is.uncertainty > 0.5 => 0.25,
            ActionType::RefuseUnsafe if is.threat > 0.6 => 0.40,
            ActionType::AskClarification if is.threat > 0.4 => 0.15,
            ActionType::NoOp if is.world_resources < 0.3 => 0.20,
            ActionType::DeferInsufficientEvidence if is.world_resources < 0.3 => 0.15,
            ActionType::AskClarification if is.social_harmony < 0.5 => 0.15,
            ActionType::Answer if is.kindness > 0.6 => 0.10,
            ActionType::Plan if is.curiosity > 0.5 => 0.20,
            ActionType::InternalDiagnostic => -1.0,
            _ => 0.0,
        };

        // Observation-kind bonuses
        if let Some(ctx) = ctx {
            match ctx.kind {
                crate::observation::ObservationKind::FactualQuery
                    if matches!(action, ActionType::Answer) =>
                {
                    bonus += 0.25;
                }
                crate::observation::ObservationKind::MemoryLookup
                    if matches!(action, ActionType::RetrieveMemory) =>
                {
                    bonus += 0.30;
                }
                crate::observation::ObservationKind::SummarizationRequest
                    if matches!(action, ActionType::Summarize) =>
                {
                    bonus += 0.30;
                }
                crate::observation::ObservationKind::PlanningRequest
                    if matches!(action, ActionType::Plan) =>
                {
                    bonus += 0.30;
                }
                crate::observation::ObservationKind::UnsafeRequest
                    if matches!(action, ActionType::RefuseUnsafe) =>
                {
                    bonus += 0.25;
                }
                crate::observation::ObservationKind::AmbiguousRequest
                    if matches!(action, ActionType::AskClarification) =>
                {
                    bonus += 0.20;
                }
                crate::observation::ObservationKind::InsufficientContext
                    if matches!(
                        action,
                        ActionType::DeferInsufficientEvidence | ActionType::RetrieveMemory
                    ) =>
                {
                    bonus += 0.20;
                }
                _ => {}
            }
        }

        // Symbolic activation influence: small bonus when
        // activations match the action's purpose.
        let symbolic_bonus = self
            .symbolic_activations
            .iter()
            .filter(|sym| {
                sym.symbol_id.contains(action.as_str())
                    || action.as_str().contains(&sym.symbol_id)
                    || sym.influence.contains(action.as_str())
            })
            .count() as f64
            * 0.05;
        bonus += symbolic_bonus;

        // Add accumulated outcome bias (bounded to ±BIAS_CAP).
        let bias = self.outcome_biases.get(action).copied().unwrap_or(0.0);
        bonus += bias;

        (base + bonus).clamp(0.0, 1.0)
    }

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

    fn rejection_reason(
        &self,
        action: &ActionType,
        _score: f64,
        is: &InternalState,
    ) -> Option<String> {
        if matches!(action, ActionType::InternalDiagnostic) {
            return Some("internal_diagnostic_must_not_be_user_facing".into());
        }
        if is.honesty < 0.35 {
            return Some("honesty_below_threshold".into());
        }
        if matches!(action, ActionType::ExecuteBoundedTool) {
            return Some("tool_policy_not_satisfied".into());
        }
        if is.control < 0.3 && is.threat > 0.7 && !action.is_reversible() {
            return Some("low_control_high_threat_irreversible".into());
        }
        if is.uncertainty > 0.65
            && !action.is_reversible()
            && !matches!(action, ActionType::AskClarification)
        {
            return Some("high_uncertainty_irreversible_action".into());
        }
        None
    }

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

        if let Some(ctx) = self.last_context.as_ref() {
            mods.push(format!("observation_kind:{}", ctx.kind.as_str()));
        }
        if is.world_resources < 0.3 {
            mods.push("resource_pressure".into());
        }
        if is.uncertainty > 0.65 {
            mods.push("high_uncertainty".into());
        }

        mods
    }

    fn select(&self, passing: &[ActionType]) -> (ActionType, String) {
        let is = &self.internal_state;
        let ctx = self.last_context.as_ref();

        // Observation-kind-driven selection first
        if let Some(ctx) = ctx {
            match ctx.kind {
                crate::observation::ObservationKind::UnsafeRequest
                    if passing.contains(&ActionType::RefuseUnsafe) =>
                {
                    return (
                        ActionType::RefuseUnsafe,
                        "Selected refuse_unsafe: observation classified as unsafe.".into(),
                    );
                }
                crate::observation::ObservationKind::AmbiguousRequest
                    if passing.contains(&ActionType::AskClarification) =>
                {
                    return (
                        ActionType::AskClarification,
                        "Selected ask_clarification: observation is ambiguous.".into(),
                    );
                }
                crate::observation::ObservationKind::MemoryLookup
                    if passing.contains(&ActionType::RetrieveMemory) =>
                {
                    return (
                        ActionType::RetrieveMemory,
                        "Selected retrieve_memory: observation indicates memory lookup.".into(),
                    );
                }
                crate::observation::ObservationKind::SummarizationRequest
                    if passing.contains(&ActionType::Summarize) =>
                {
                    return (
                        ActionType::Summarize,
                        "Selected summarize: observation requests summarization.".into(),
                    );
                }
                crate::observation::ObservationKind::PlanningRequest
                    if passing.contains(&ActionType::Plan) =>
                {
                    return (
                        ActionType::Plan,
                        "Selected plan: observation requests planning.".into(),
                    );
                }
                crate::observation::ObservationKind::FactualQuery
                    if passing.contains(&ActionType::Answer) =>
                {
                    return (
                        ActionType::Answer,
                        "Selected answer: observation is a factual query.".into(),
                    );
                }
                crate::observation::ObservationKind::InsufficientContext
                    if passing.contains(&ActionType::DeferInsufficientEvidence) =>
                {
                    return (
                        ActionType::DeferInsufficientEvidence,
                        "Selected defer_insufficient_evidence: context is insufficient.".into(),
                    );
                }
                _ => {}
            }
        }

        // State-based selection (fallback)
        if is.threat > 0.65 && passing.contains(&ActionType::RefuseUnsafe) {
            return (
                ActionType::RefuseUnsafe,
                "Selected refuse_unsafe because threat is high.".into(),
            );
        }
        if is.uncertainty > 0.65 && passing.contains(&ActionType::AskClarification) {
            return (
                ActionType::AskClarification,
                "Selected ask_clarification because uncertainty is high.".into(),
            );
        }
        if is.uncertainty > 0.5 {
            if passing.contains(&ActionType::DeferInsufficientEvidence) {
                return (
                    ActionType::DeferInsufficientEvidence,
                    "Selected defer: evidence insufficient.".into(),
                );
            }
            if passing.contains(&ActionType::RetrieveMemory) {
                return (
                    ActionType::RetrieveMemory,
                    "Selected retrieve_memory to gather evidence.".into(),
                );
            }
        }
        if is.world_resources < 0.3 && passing.contains(&ActionType::NoOp) {
            return (
                ActionType::NoOp,
                "Selected no_op: resources critically low.".into(),
            );
        }
        if passing.contains(&ActionType::Answer) {
            return (
                ActionType::Answer,
                "Selected answer: sufficient context available.".into(),
            );
        }
        if passing.contains(&ActionType::Summarize) {
            return (
                ActionType::Summarize,
                "Selected summarize as best available.".into(),
            );
        }
        if let Some(first) = passing.first() {
            return (first.clone(), format!("Selected {first} as default."));
        }
        (ActionType::NoOp, "Selected no_op: no useful action.".into())
    }
}

impl Default for RuntimeLoop {
    fn default() -> Self {
        Self::new(Box::new(
            crate::memory_provider::KeywordMemoryProvider::new(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_provider::KeywordMemoryProvider;

    fn test_rt() -> RuntimeLoop {
        RuntimeLoop::new(Box::new(KeywordMemoryProvider::new()))
    }

    #[test]
    fn unsafe_observation_yields_refuse() {
        let mut rt = test_rt();
        let result = rt.run_cycle("unsafe_request", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::RefuseUnsafe);
        assert!(!result.memory_hits.is_empty());
        assert!(!result.symbolic_activations.is_empty());
    }

    #[test]
    fn factual_query_yields_answer() {
        let mut rt = test_rt();
        let result = rt.run_cycle("factual_query", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::Answer);
    }

    #[test]
    fn memory_lookup_yields_retrieve_memory() {
        let mut rt = test_rt();
        let result = rt.run_cycle("memory_lookup", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::RetrieveMemory);
    }

    #[test]
    fn planning_request_yields_plan() {
        let mut rt = test_rt();
        let result = rt.run_cycle("planning_request", 1, 0.9);
        assert_eq!(result.selected_action, ActionType::Plan);
    }

    #[test]
    fn internal_diagnostic_never_selected() {
        let mut rt = test_rt();
        let result = rt.run_cycle("unsafe_request", 1, 0.9);
        assert_ne!(result.selected_action, ActionType::InternalDiagnostic);
    }

    #[test]
    fn memory_returns_hits_for_memory_lookup() {
        let mut rt = test_rt();
        let result = rt.run_cycle("memory_lookup", 1, 0.9);
        // memory_lookup observation should return at least some memory hits
        assert!(!result.memory_hits.is_empty());
        assert!(result.memory_hits.iter().any(|h| h.relevance > 0.0));
    }

    #[test]
    fn symbolic_activations_are_populated() {
        let mut rt = test_rt();
        let result = rt.run_cycle("unsafe_request", 1, 0.9);
        assert!(!result.symbolic_activations.is_empty());
    }
}
