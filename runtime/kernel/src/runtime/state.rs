use crate::simworld::types::SimAction;

/// Pure data representing the full observable state of the runtime kernel.
///
/// `RuntimeState` must never be mutated directly from business logic.
/// All changes go through `reduce(state, event)` in the replay module.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeState {
    // ── cycle bookkeeping ────────────────────────────────────────────────────
    pub cycle_id: u64,

    // ── observation ──────────────────────────────────────────────────────────
    pub last_input: String,
    pub last_source: String,

    // ── memory ───────────────────────────────────────────────────────────────
    pub memory_query_count: u64,
    pub last_memory_hit_count: usize,
    pub last_memory_top_text: Option<String>,

    // ── candidates ───────────────────────────────────────────────────────────
    pub candidates_generated: u64,
    pub candidates_rejected: u64,
    pub last_candidate_id: Option<String>,
    pub last_candidate_stream: Option<String>,
    pub last_candidate_action_type: Option<String>,
    pub last_candidate_confidence: Option<f64>,
    pub last_rejection: Option<(String, String)>, // (candidate_id, reason)

    // ── selection ────────────────────────────────────────────────────────────
    pub selected_candidate_id: Option<String>,
    pub selected_action_type: Option<String>,
    pub selected_text: Option<String>,

    // ── world state ──────────────────────────────────────────────────────────
    pub resources: f64,
    pub social_harmony: f64,
    pub unresolved_contradictions: u64,
    pub total_actions: u64,
    pub conserve_actions: u64,
    pub last_applied_action: Option<SimAction>,

    // ── scoring ──────────────────────────────────────────────────────────────
    pub total_score_sum: f64,
    pub scored_cycles: u64,

    // ── archive ──────────────────────────────────────────────────────────────
    pub archive_commits: u64,
    pub last_frame_id: Option<String>,

    // ── contradiction ────────────────────────────────────────────────────────
    pub contradictions_detected: u64,
    pub contradictions_resolved: u64,

    // ── mode ─────────────────────────────────────────────────────────────────
    pub current_mode: String,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            cycle_id: 0,
            last_input: String::new(),
            last_source: String::new(),
            memory_query_count: 0,
            last_memory_hit_count: 0,
            last_memory_top_text: None,
            candidates_generated: 0,
            candidates_rejected: 0,
            last_candidate_id: None,
            last_candidate_stream: None,
            last_candidate_action_type: None,
            last_candidate_confidence: None,
            last_rejection: None,
            selected_candidate_id: None,
            selected_action_type: None,
            selected_text: None,
            resources: 1.0,
            social_harmony: 0.7,
            unresolved_contradictions: 0,
            total_actions: 0,
            conserve_actions: 0,
            last_applied_action: None,
            total_score_sum: 0.0,
            scored_cycles: 0,
            archive_commits: 0,
            last_frame_id: None,
            contradictions_detected: 0,
            contradictions_resolved: 0,
            current_mode: "Normal".to_string(),
        }
    }
}

impl RuntimeState {
    /// Mean total score across all scored cycles, or 0.0 if none.
    pub fn mean_total_score(&self) -> f64 {
        if self.scored_cycles == 0 {
            return 0.0;
        }
        self.total_score_sum / self.scored_cycles as f64
    }

    /// Resource survival fraction (current resources / initial 1.0).
    pub fn resource_survival(&self) -> f64 {
        self.resources.clamp(0.0, 1.0)
    }
}
