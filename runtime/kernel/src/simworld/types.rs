use serde::{Deserialize, Serialize};

/// The bounded set of actions the runtime can take inside SimWorld.
///
/// Kept compatible with the Python `SimAction` string values so JSONL
/// output is interchangeable between the two implementations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimAction {
    Answer,
    AskClarification,
    RetrieveMemory,
    WriteScratchpad,
    Defer,
    RefuseUngrounded,
    Repair,
    Summarize,
    ConserveResources,
    GeneratePrinciple,
}

impl SimAction {
    /// Match a Python-style action-type string to a `SimAction`.
    ///
    /// Returns `None` if the string does not map to any known action so that
    /// callers can decide whether to fall back to `Answer` explicitly — no
    /// silent coercion.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "answer" => Some(Self::Answer),
            "ask_clarification" | "ask_for_clarification" => Some(Self::AskClarification),
            "retrieve_memory" => Some(Self::RetrieveMemory),
            "write_scratchpad" => Some(Self::WriteScratchpad),
            "defer" => Some(Self::Defer),
            "refuse_ungrounded" => Some(Self::RefuseUngrounded),
            "repair" => Some(Self::Repair),
            "summarize" => Some(Self::Summarize),
            "conserve_resources" => Some(Self::ConserveResources),
            "generate_principle" => Some(Self::GeneratePrinciple),
            _ => None,
        }
    }

    /// Classify free-form selected_text as a `SimAction` — used when the
    /// runtime does not emit an explicit `action_type` label.
    pub fn from_text(text: &str) -> Self {
        let lower = text.to_lowercase();
        if lower.contains("clarif") || (lower.contains("ask") && !lower.contains("task")) {
            return Self::AskClarification;
        }
        if lower.contains("retrieve") || lower.contains("memory") {
            return Self::RetrieveMemory;
        }
        if lower.contains("scratchpad") || lower.contains("write") {
            return Self::WriteScratchpad;
        }
        if lower.contains("defer") || lower.contains("wait") {
            return Self::Defer;
        }
        if lower.contains("unsupported") || lower.contains("reject") || lower.contains("refuse") {
            return Self::RefuseUngrounded;
        }
        if lower.contains("repair") || lower.contains("correct") || lower.contains("acknowledge") {
            return Self::Repair;
        }
        if lower.contains("summary") || lower.contains("summarize") {
            return Self::Summarize;
        }
        if lower.contains("short") || lower.contains("resource") || lower.contains("conserve") {
            return Self::ConserveResources;
        }
        if lower.contains("principle") {
            return Self::GeneratePrinciple;
        }
        Self::Answer
    }
}

/// A simulated user with trust and interaction traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimUser {
    pub user_id: String,
    pub temperament: String,
    pub trust: f64,
    pub patience: f64,
}

/// One event emitted by the SimWorld for the runtime to respond to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimWorldEvent {
    pub event_id: String,
    pub user_id: String,
    /// Visible text shown to the runtime.
    pub text: String,
    /// Hidden template tag — not exposed to the runtime.
    pub hidden_truth: String,
    pub risk_level: f64,
    pub uncertainty_level: f64,
    pub kindness_need: f64,
    pub resource_cost: f64,
    /// The action the evaluator considers correct for this event.
    pub expected_action: Option<SimAction>,
}

/// Outcome after the runtime applies an action to an event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimOutcome {
    pub event_id: String,
    pub action: SimAction,
    pub truth_score: f64,
    pub kindness_score: f64,
    pub social_harmony: f64,
    pub user_trust_delta: f64,
    pub resource_delta: f64,
    pub uncertainty_resolution: f64,
    pub repair_success: f64,
    pub cold_optimization_penalty: f64,
    pub notes: Vec<String>,
}

impl SimOutcome {
    /// Weighted composite score — matches the Python `total_score` property.
    pub fn total_score(&self) -> f64 {
        (self.truth_score
            + self.kindness_score
            + self.social_harmony
            + self.uncertainty_resolution
            + self.repair_success
            - self.cold_optimization_penalty
            + self.user_trust_delta)
            / 6.0
    }
}

/// Aggregate state of the simulated world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimWorldState {
    pub cycle: u64,
    pub resources: f64,
    pub social_harmony: f64,
    pub unresolved_contradictions: u64,
    pub repeated_mistakes: u64,
}

impl Default for SimWorldState {
    fn default() -> Self {
        Self {
            cycle: 0,
            resources: 1.0,
            social_harmony: 0.7,
            unresolved_contradictions: 0,
            repeated_mistakes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_known_values_round_trip() {
        let pairs = [
            ("answer", SimAction::Answer),
            ("ask_clarification", SimAction::AskClarification),
            ("ask_for_clarification", SimAction::AskClarification),
            ("retrieve_memory", SimAction::RetrieveMemory),
            ("refuse_ungrounded", SimAction::RefuseUngrounded),
            ("repair", SimAction::Repair),
            ("conserve_resources", SimAction::ConserveResources),
        ];
        for (s, expected) in &pairs {
            assert_eq!(SimAction::from_str(s), Some(expected.clone()));
        }
    }

    #[test]
    fn from_str_unknown_returns_none() {
        // No silent fallback to Answer.
        assert_eq!(SimAction::from_str("definitely_not_a_real_action"), None);
        assert_eq!(SimAction::from_str(""), None);
    }

    #[test]
    fn total_score_formula_matches_python() {
        let outcome = SimOutcome {
            event_id: "test".into(),
            action: SimAction::AskClarification,
            truth_score: 0.85,
            kindness_score: 0.85,
            social_harmony: 0.73,
            user_trust_delta: 0.04,
            resource_delta: -0.10,
            uncertainty_resolution: 0.80,
            repair_success: 0.25,
            cold_optimization_penalty: 0.0,
            notes: vec![],
        };
        // (0.85 + 0.85 + 0.73 + 0.80 + 0.25 - 0.0 + 0.04) / 6.0
        let expected = (0.85 + 0.85 + 0.73 + 0.80 + 0.25 - 0.0 + 0.04) / 6.0;
        let diff = (outcome.total_score() - expected).abs();
        assert!(
            diff < 1e-10,
            "total_score mismatch: {} vs {}",
            outcome.total_score(),
            expected
        );
    }
}
