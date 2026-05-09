//! ThoughtCandidate — a single scored candidate action.

use runtime_core::ActionType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtCandidate {
    pub action_type: ActionType,
    /// Raw critic score (not clamped).
    pub score: f64,
    /// Whether this candidate passed all critic rejection rules.
    pub passes_critic: bool,
    /// Resource cost estimate (0–1).
    pub resource_cost: f64,
    /// Whether this candidate was generated as reversible.
    pub reversible: bool,
    /// Reasoning string (optional, for traces).
    pub reasoning: Option<String>,
}

impl ThoughtCandidate {
    pub fn new(action_type: ActionType, resource_cost: f64) -> Self {
        Self {
            action_type,
            score: 0.0,
            passes_critic: true,
            resource_cost,
            reversible: true,
            reasoning: None,
        }
    }
}

/// A full packet of candidates for one cycle.
#[derive(Debug, Clone)]
pub struct CandidatePacket {
    pub cycle_id: u64,
    pub candidates: Vec<ThoughtCandidate>,
}

impl CandidatePacket {
    pub fn new(cycle_id: u64) -> Self {
        Self {
            cycle_id,
            candidates: Vec::new(),
        }
    }

    pub fn push(&mut self, c: ThoughtCandidate) {
        self.candidates.push(c);
    }

    /// Best passing candidate by descending score.
    pub fn best(&self) -> Option<&ThoughtCandidate> {
        self.candidates
            .iter()
            .filter(|c| c.passes_critic)
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
