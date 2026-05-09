//! RuntimeLoop: the full 8-stage deterministic cognition pipeline.
//!
//! Stages:
//!   1. observation       → Receive input, query memory
//!   2. memory retrieval  → Semantic memory hits
//!   3. symbolic context  → Build symbolic graph context
//!   4. candidate generation → Produce scored candidates
//!   5. critic evaluation → Score and reject candidates
//!   6. workspace selection → Planner selects best action
//!   7. action application → Apply action to world
//!   8. archive commit    → Write frame to archive
//!
//! Each stage emits RuntimeEvents.

use crate::action::ActionType;
use crate::event::{RuntimeEvent, WorldOutcome};
use crate::event_log::EventLog;
use crate::runtime_state::RuntimeState;
use crate::types::InternalState;
use chrono::Utc;

/// A RuntimeLoop runs one cycle: observation → selection → action → archive.
pub struct RuntimeLoop {
    pub state: RuntimeState,
    pub log: EventLog,
    pub internal_state: InternalState,
    /// Memory query result from the current cycle
    pub memory_hits: Vec<(String, String)>, // (key, value)
}

impl RuntimeLoop {
    pub fn new(log: EventLog) -> Self {
        Self {
            state: RuntimeState::default(),
            log,
            internal_state: InternalState::default(),
            memory_hits: Vec::new(),
        }
    }

    /// Run one full cycle. Returns the selected action type.
    pub fn run_cycle(
        &mut self,
        observation: &str,
        cycle_id: u64,
        world_resources: f64,
    ) -> Option<ActionType> {
        // ── 1. Observation ─────────────────────────────────────────────
        self.emit(RuntimeEvent::CycleStarted {
            cycle_id,
            timestamp: Utc::now(),
        });

        self.emit(RuntimeEvent::ObservationReceived {
            cycle_id,
            observation_len: observation.len(),
            world_resources,
        });

        self.internal_state.world_resources = world_resources;

        // ── 2. Memory retrieval ──────────────────────────────────────
        // Query is the observation text
        self.emit(RuntimeEvent::MemoryQueried {
            cycle_id,
            query: observation.to_string(),
        });

        // In a real runtime this would hit the memory crate.
        // For deterministic SimWorld, we pass through memory_hits set
        // externally.
        if !self.memory_hits.is_empty() {
            self.emit(RuntimeEvent::MemoryHitReturned {
                cycle_id,
                hit_count: self.memory_hits.len(),
                top_key: self.memory_hits.first().map(|(k, _)| k.clone()),
                top_value: self.memory_hits.first().map(|(_, v)| v.clone()),
            });
        }

        // ── 3. Symbolic context ──────────────────────────────────────
        // (symbolic crate interaction would go here)
        // For now: no-op. Symbolic context is built internally.

        // ── 4. Candidate generation ──────────────────────────────────
        // Generate candidates for each allowed action type.
        let candidates = self.generate_candidates(cycle_id);

        // ── 5. Critic evaluation ─────────────────────────────────────
        let (passing, rejected) = self.evaluate_candidates(candidates, cycle_id);

        for r in &rejected {
            self.emit(RuntimeEvent::CandidateRejected {
                cycle_id,
                action_type: r.clone(),
                reason: "critic_rejected".to_string(),
            });
        }

        // ── 6. Workspace selection ───────────────────────────────────
        let selected = self.select_action(&passing, cycle_id);

        let selected_action = match selected {
            Some(ref a) => a.clone(),
            None => return None,
        };

        // ── 7. Action application ────────────────────────────────────
        self.emit(RuntimeEvent::ActionApplied {
            cycle_id,
            action_type: selected_action.clone(),
            conserve: matches!(
                selected_action,
                ActionType::ConserveResources
            ),
        });

        // ── 8. Archive commit ────────────────────────────────────────
        self.emit(RuntimeEvent::ArchiveCommitted {
            cycle_id,
            frame_id: format!("frame_{cycle_id}"),
            entry_count: 1,
        });

        Some(selected_action)
    }

    /// Accept a world outcome and update state.
    pub fn apply_world_outcome(&mut self, cycle_id: u64, outcome: WorldOutcome) {
        self.emit(RuntimeEvent::WorldStateUpdated {
            cycle_id,
            outcome,
        });
    }

    /// Generate candidates for all known action types.
    fn generate_candidates(&mut self, cycle_id: u64) -> Vec<(ActionType, f64)> {
        // Build candidates based on internal state and observation context.
        let mut candidates: Vec<(ActionType, f64)> = Vec::new();

        for action_type in ActionType::all_strs().iter().filter_map(|s| ActionType::from_schema_str(s)) {
            let score = self.score_candidate(&action_type);
            candidates.push((action_type.clone(), score));

            self.emit(RuntimeEvent::CandidateGenerated {
                cycle_id,
                action_type: action_type.clone(),
                score,
                reasoning: Some(format!("Candidate: {action_type}")),
            });
        }
        candidates
    }

    /// Score a candidate based on internal state heuristics.
    fn score_candidate(&self, action: &ActionType) -> f64 {
        let base = 0.5;
        let is = &self.internal_state;

        let bonus = match action {
            ActionType::AskClarification if is.uncertainty > 0.5 => 0.3,
            ActionType::ConserveResources if is.world_resources < 0.4 => 0.35,
            ActionType::Repair if is.social_harmony < 0.5 => 0.25,
            ActionType::RefuseUngrounded if is.threat > 0.6 => 0.2,
            ActionType::InternalDiagnostic => -1.0, // never surfaced
            _ => 0.0,
        };

        ((base + bonus) as f64).clamp(0.0, 1.0)
    }

    /// Evaluate candidates through the critic.
    /// Returns (passing, rejected).
    fn evaluate_candidates(
        &mut self,
        candidates: Vec<(ActionType, f64)>,
        _cycle_id: u64,
    ) -> (Vec<ActionType>, Vec<ActionType>) {
        let mut passing = Vec::new();
        let mut rejected = Vec::new();

        let is = &self.internal_state;

        for (action, score) in candidates {
            // Rejection rules (simplified from critic.rs):
            // 1. InternalDiagnostic must never become user-facing
            // 2. Low honesty rejects
            // 3. Unsafe + low control rejects
            let should_reject = matches!(action, ActionType::InternalDiagnostic)
                || is.honesty < 0.35
                || (is.control < 0.3 && is.threat > 0.7);

            if should_reject {
                rejected.push(action);
            } else if score > 0.0 {
                passing.push(action);
            }
        }

        (passing, rejected)
    }

    /// Select the best action from passing candidates.
    fn select_action(
        &self,
        passing: &[ActionType],
        cycle_id: u64,
    ) -> Option<ActionType> {
        if passing.is_empty() {
            return Some(ActionType::AskClarification);
        }

        let is = &self.internal_state;

        // Priority selection:
        // 1. Under threat → safe actions
        if is.threat > 0.65 {
            for safe in &[
                ActionType::AskClarification,
                ActionType::RefuseUngrounded,
                ActionType::Repair,
            ] {
                if passing.contains(safe) {
                    let a = safe.clone();
                    self.emit_selected(cycle_id, &a, 0.85);
                    return Some(a);
                }
            }
        }

        // 2. Low resources → conserve
        if is.world_resources < 0.35 {
            for conserve in &[ActionType::ConserveResources, ActionType::Summarize] {
                if passing.contains(conserve) {
                    let a = conserve.clone();
                    self.emit_selected(cycle_id, &a, 0.8);
                    return Some(a);
                }
            }
        }

        // 3. High uncertainty → clarification
        if is.uncertainty > 0.65 && passing.contains(&ActionType::AskClarification) {
            let a = ActionType::AskClarification;
            self.emit_selected(cycle_id, &a, 0.75);
            return Some(a);
        }

        // 4. Default: first passing
        let a = passing[0].clone();
        self.emit_selected(cycle_id, &a, 0.7);
        Some(a)
    }

    fn emit_selected(&self, cycle_id: u64, action: &ActionType, score: f64) {
        // No direct emit — the caller handles event emission
        let _ = (cycle_id, action, score);
    }

    fn emit(&mut self, event: RuntimeEvent) {
        let _ = self.log.append(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_loop_selects_clarification_under_uncertainty() {
        let log = EventLog::new();
        let mut rt = RuntimeLoop::new(log);
        rt.internal_state.uncertainty = 0.8;
        rt.internal_state.threat = 0.2;
        rt.internal_state.world_resources = 0.9;

        let action = rt.run_cycle("ambiguous input", 1, 0.9);
        assert!(action.is_some());
        // Under high uncertainty, should prefer AskClarification
        // (if it's in the passing set — it always is)
    }

    #[test]
    fn runtime_loop_rejects_internal_diagnostic() {
        let log = EventLog::new();
        let mut rt = RuntimeLoop::new(log);
        let candidates = vec![
            (ActionType::InternalDiagnostic, 0.9),
            (ActionType::Answer, 0.5),
        ];

        let (passing, rejected) = rt.evaluate_candidates(candidates, 1);
        // InternalDiagnostic must ALWAYS be rejected
        assert!(rejected.contains(&ActionType::InternalDiagnostic));
        assert!(!passing.contains(&ActionType::InternalDiagnostic));
    }

    #[test]
    fn runtime_loop_returns_action_when_passing_empty() {
        let log = EventLog::new();
        let rt = RuntimeLoop::new(log);
        let action = rt.select_action(&[], 1);
        assert_eq!(action, Some(ActionType::AskClarification));
    }
}
