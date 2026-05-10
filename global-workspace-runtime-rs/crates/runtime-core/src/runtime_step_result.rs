//! RuntimeStepResult — the central contract returned by RuntimeLoop.
//!
//! Every cycle produces one RuntimeStepResult with full audit trail:
//! scored candidates, rejected actions with reasons, memory hits,
//! symbolic activations, policy scores, and all events emitted.

use crate::action::ActionType;
use crate::event::RuntimeEvent;
use serde::{Deserialize, Serialize};

/// A single scored candidate action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCandidate {
    pub action_type: ActionType,
    pub score: f64,
    pub reasoning: Option<String>,
}

/// A rejected action with explicit rejection reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedAction {
    pub action_type: ActionType,
    pub reason: String,
}

/// A memory retrieval hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHit {
    pub key: String,
    pub value: String,
    pub relevance: f64,
}

/// A symbolic activation triggered by this cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolActivation {
    pub symbol_id: String,
    pub glyph: String,
    pub activation: f64,
    pub influence: String,
}

/// A policy score modifier for an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionScore {
    pub action_type: ActionType,
    pub base_score: f64,
    pub bonus: f64,
    pub final_score: f64,
    pub modifiers: Vec<String>,
}

/// The full result of one runtime cycle.
///
/// This is the authoritative output contract. The evaluator consumes
/// this to populate traces and proof artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStepResult {
    pub cycle_id: u64,
    pub observation: String,
    pub selected_action: ActionType,
    /// All candidates that were scored.
    pub candidate_actions: Vec<ActionCandidate>,
    /// Candidates that were rejected with reasons.
    pub rejected_actions: Vec<RejectedAction>,
    /// Memory retrieval results.
    pub memory_hits: Vec<MemoryHit>,
    /// Symbolic activations from this cycle.
    pub symbolic_activations: Vec<SymbolActivation>,
    /// Per-action policy scores.
    pub policy_scores: Vec<ActionScore>,
    /// All events emitted during this cycle.
    pub events: Vec<RuntimeEvent>,
    /// Human-readable reason for selection.
    pub selection_reason: String,
    /// Whether the selected action is safe for users.
    pub is_safe: bool,
}

impl RuntimeStepResult {
    pub fn new(cycle_id: u64, observation: impl Into<String>) -> Self {
        Self {
            cycle_id,
            observation: observation.into(),
            selected_action: ActionType::NoOp,
            candidate_actions: Vec::new(),
            rejected_actions: Vec::new(),
            memory_hits: Vec::new(),
            symbolic_activations: Vec::new(),
            policy_scores: Vec::new(),
            events: Vec::new(),
            selection_reason: String::new(),
            is_safe: true,
        }
    }

    /// Total number of scored candidates.
    pub fn candidate_count(&self) -> usize {
        self.candidate_actions.len()
    }

    /// Total number of rejected candidates.
    pub fn rejected_count(&self) -> usize {
        self.rejected_actions.len()
    }

    /// True if any candidate was rejected.
    pub fn has_rejections(&self) -> bool {
        !self.rejected_actions.is_empty()
    }

    /// True if any memory was retrieved.
    pub fn has_memory_hits(&self) -> bool {
        !self.memory_hits.is_empty()
    }

    /// True if any symbols were activated.
    pub fn has_symbolic_activations(&self) -> bool {
        !self.symbolic_activations.is_empty()
    }

    /// Number of events emitted.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}
