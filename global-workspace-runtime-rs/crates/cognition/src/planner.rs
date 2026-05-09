//! Planner: select best action from a scored packet.
//! Implements the 8-step decision tree from cognition/planner.py.

use crate::candidate::CandidatePacket;
use modulation::SomaticMap;
use runtime_core::{ActionType, InternalState};

pub struct Planner;

impl Planner {
    /// Select the best action given current state, somatic map, and candidates.
    pub fn select(
        state: &InternalState,
        somatic: &SomaticMap,
        packet: &CandidatePacket,
        allowed: &[ActionType],
    ) -> ActionType {
        let passing: Vec<&crate::candidate::ThoughtCandidate> = packet
            .candidates
            .iter()
            .filter(|c| c.passes_critic && allowed.contains(&c.action_type))
            .collect();

        // Step 1: control too low → ask for clarification
        if state.control < 0.3 {
            return Self::prefer_or_best(ActionType::AskClarification, &passing);
        }

        // Step 2: no allowed candidates → retrieve memory
        if passing.is_empty() {
            return ActionType::RetrieveMemory;
        }

        // Step 3: somatic override
        if somatic.predicts_bad_outcome(0.52) {
            if let Some(preferred_str) = somatic.preferred_action_under_pressure() {
                if let Some(a) = ActionType::from_schema_str(preferred_str) {
                    if allowed.contains(&a) {
                        return a;
                    }
                }
            }
        }

        // Step 4: threat or uncertainty > 0.65 → safe set
        let safe_set = [
            ActionType::AskClarification,
            ActionType::RetrieveMemory,
            ActionType::RefuseUngrounded,
            ActionType::Repair,
            ActionType::Summarize,
        ];
        if state.threat > 0.65 || state.uncertainty > 0.65 {
            if let Some(best) = passing
                .iter()
                .filter(|c| safe_set.contains(&c.action_type))
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                return best.action_type.clone();
            }
        }

        // Step 5: world_resources < 0.35 → conserve-preferred, sort by cost ASC then score DESC
        if state.world_resources < 0.35 {
            let conserve_set = [
                ActionType::ConserveResources,
                ActionType::Summarize,
                ActionType::AskClarification,
            ];
            if let Some(best) = passing
                .iter()
                .filter(|c| conserve_set.contains(&c.action_type))
                .min_by(|a, b| {
                    a.resource_cost
                        .partial_cmp(&b.resource_cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            b.score
                                .partial_cmp(&a.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                })
            {
                return best.action_type.clone();
            }
        }

        // Step 6: resource_pressure > 0.65 → prefer conserve set
        if state.resource_pressure > 0.65 {
            let conserve_set = [
                ActionType::ConserveResources,
                ActionType::Summarize,
                ActionType::AskClarification,
            ];
            if let Some(best) = passing
                .iter()
                .filter(|c| conserve_set.contains(&c.action_type))
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                return best.action_type.clone();
            }
        }

        // Step 7: curious + low threat → exploratory
        if state.curiosity > 0.65 && state.threat < 0.4 {
            let exploratory = [
                ActionType::Answer,
                ActionType::WriteScratchpad,
                ActionType::GeneratePrinciple,
            ];
            if let Some(best) = passing
                .iter()
                .filter(|c| exploratory.contains(&c.action_type))
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                return best.action_type.clone();
            }
        }

        // Step 8: best by total score
        let best = passing
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|c| c.action_type.clone());

        best.unwrap_or(ActionType::AskClarification)
    }

    /// Return `preferred` if it's in the passing set, otherwise best.
    fn prefer_or_best(
        preferred: ActionType,
        passing: &[&crate::candidate::ThoughtCandidate],
    ) -> ActionType {
        if passing.iter().any(|c| c.action_type == preferred) {
            preferred
        } else {
            passing
                .iter()
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|c| c.action_type.clone())
                .unwrap_or(ActionType::AskClarification)
        }
    }
}
