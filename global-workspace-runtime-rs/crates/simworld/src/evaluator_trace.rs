use serde::{Deserialize, Serialize};

/// A full trace record for one SimWorld cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorTrace {
    pub cycle_id: u64,
    pub scenario_id: String,
    pub observation: String,
    pub memory_hits_count: usize,
    pub symbolic_context_count: usize,
    pub candidate_actions: Vec<String>,
    pub rejected_actions: Vec<RejectionRecord>,
    pub selected_action: String,
    pub expected_action: String,
    pub action_match: bool,
    pub risk_score: f64,
    pub uncertainty_score: f64,
    pub resource_score_before: f64,
    pub resource_score_after: f64,
    pub unsafe_action_flag: bool,
    pub runtime_events_count: usize,
    pub selection_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRecord {
    pub action: String,
    pub reason: String,
}

impl EvaluatorTrace {
    pub fn new(cycle_id: u64, scenario_id: &str, observation: &str) -> Self {
        Self {
            cycle_id,
            scenario_id: scenario_id.to_string(),
            observation: observation.to_string(),
            memory_hits_count: 0,
            symbolic_context_count: 0,
            candidate_actions: Vec::new(),
            rejected_actions: Vec::new(),
            selected_action: String::new(),
            expected_action: String::new(),
            action_match: false,
            risk_score: 0.0,
            uncertainty_score: 0.0,
            resource_score_before: 0.0,
            resource_score_after: 0.0,
            unsafe_action_flag: false,
            runtime_events_count: 0,
            selection_reason: String::new(),
        }
    }
}
