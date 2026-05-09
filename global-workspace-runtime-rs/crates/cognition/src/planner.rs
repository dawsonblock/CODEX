//! Planner: select best action from a scored packet.
//! Updated for ActionType v2 (10 types).

use crate::candidate::CandidatePacket;
use modulation::SomaticMap;
use runtime_core::{ActionType, InternalState};

pub struct Planner;

impl Planner {
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

        if state.control < 0.3 {
            return Self::prefer_or_best(ActionType::AskClarification, &passing);
        }

        if passing.is_empty() {
            return ActionType::RetrieveMemory;
        }

        if somatic.predicts_bad_outcome(0.52) {
            if let Some(preferred_str) = somatic.preferred_action_under_pressure() {
                if let Some(a) = ActionType::from_schema_str(preferred_str) {
                    if allowed.contains(&a) {
                        return a;
                    }
                }
            }
        }

        let safe_set = [
            ActionType::AskClarification,
            ActionType::RetrieveMemory,
            ActionType::RefuseUnsafe,
            ActionType::DeferInsufficientEvidence,
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

        if state.world_resources < 0.35 {
            let conserve_set = [
                ActionType::NoOp,
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

        if state.resource_pressure > 0.65 {
            let conserve_set = [
                ActionType::NoOp,
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

        if state.curiosity > 0.65 && state.threat < 0.4 {
            let exploratory = [ActionType::Answer, ActionType::Plan];
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
