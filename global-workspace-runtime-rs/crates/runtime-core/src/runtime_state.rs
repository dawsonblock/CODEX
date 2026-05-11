use crate::action::ActionType;
use crate::mode::RuntimeMode;
use serde::{Deserialize, Serialize};

/// Full runtime state — rebuilt by replaying events through the reducer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeState {
    // Cycle tracking
    pub cycle_id: u64,
    pub total_cycles: u64,

    // Resource accounting
    pub resources: f64,
    pub conserve_actions: u64,

    // Safety counters (MUST stay at zero in production)
    pub unsafe_action_count: u64,
    pub false_confidence_count: u64,
    pub repeated_mistakes: u64,

    // Action history
    pub last_action_type: Option<ActionType>,
    pub selected_action_type: Option<ActionType>,
    pub last_candidate_action_type: Option<ActionType>,
    pub total_actions: u64,

    // Dialogue/world coherence
    pub contradiction_count: u64,
    pub world_model_mismatch: u64,
    pub self_report_invalid: u64,
    pub contradictions_detected: u64,
    pub contradictions_resolved: u64,
    pub contradictions_escalated: u64,
    pub unresolved_contradictions: u64,

    // Mode
    pub current_mode: RuntimeMode,

    // Memory health (0.0–1.0)
    pub memory_health: f64,
    pub memory_query_count: u64,
    pub last_memory_hit_count: usize,
    pub last_memory_top_key: Option<String>,
    pub last_memory_top_value: Option<String>,

    // Candidate tracking
    pub candidates_generated: u64,
    pub candidates_rejected: u64,
    pub last_rejection: Option<(String, String)>,

    // Symbolic state
    pub symbolic_state_hash: Option<u64>,
    pub symbol_activations: u64,
    pub symbol_links: u64,
    pub last_symbolic_symbol_count: usize,
    pub last_symbolic_edge_count: usize,
    pub blends_generated: u64,
    pub principles_extracted: u64,
    pub last_principle_confidence: f64,
    pub compression_applied: u64,
    pub last_compression_ratio: f64,
    pub last_resonance_score: f64,

    // Score tracking (rolling)
    pub total_score_sum: f64,
    pub total_score_count: u64,
    pub last_total_score: f64,
    pub scored_cycles: u64,
    pub social_harmony: f64,

    // Per-component score sums (for audit)
    pub truth_score_sum: f64,
    pub kindness_score_sum: f64,
    pub social_score_sum: f64,
    pub logic_score_sum: f64,
    pub utility_score_sum: f64,
    pub harm_score_sum: f64,

    // Outcome tracking
    pub outcome_count: u64,
    pub matched_expected_count: u64,

    // Archive tracking
    pub archive_commits: u64,
    pub last_frame_id: Option<String>,

    // Evidence vault tracking
    pub evidence_entries: u64,
    pub evidence_tampered: u64,
    pub evidence_integrity_all_valid: bool,

    // Claim memory tracking
    pub claims_asserted: u64,
    pub claims_retrieved: u64,
    pub claims_with_evidence_links: u64,
    pub claims_validated: u64,
    pub claims_superseded: u64,
    pub contradictions_checked: u64,

    // Reasoning audit tracking
    pub reasoning_audits: u64,
    pub audits_with_evidence_refs: u64,
    pub audits_with_claim_refs: u64,

    // Tool execution tracking
    pub tools_executed: u64,
    pub tools_blocked: u64,

    // Pressure modulation tracking
    pub pressure_updates: u64,
    pub policy_bias_applications: u64,
    /// Snapshot of last known pressure values (for replay reconstruction).
    pub last_pressure_safety: f64,
    pub last_pressure_uncertainty: f64,
    pub last_pressure_resource: f64,
    pub last_pressure_contradiction: f64,
    pub last_pressure_evidence_gap: f64,
    pub last_pressure_social_risk: f64,
    pub last_pressure_tool_risk: f64,
    pub last_pressure_urgency: f64,
    pub last_pressure_coherence: f64,

    // Scratchpad
    pub scratchpad_entry_count: usize,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            cycle_id: 0,
            total_cycles: 0,
            resources: 1.0,
            conserve_actions: 0,
            unsafe_action_count: 0,
            false_confidence_count: 0,
            repeated_mistakes: 0,
            last_action_type: None,
            selected_action_type: None,
            last_candidate_action_type: None,
            total_actions: 0,
            contradiction_count: 0,
            world_model_mismatch: 0,
            self_report_invalid: 0,
            contradictions_detected: 0,
            contradictions_resolved: 0,
            contradictions_escalated: 0,
            unresolved_contradictions: 0,
            current_mode: RuntimeMode::Normal,
            memory_health: 1.0,
            memory_query_count: 0,
            last_memory_hit_count: 0,
            last_memory_top_key: None,
            last_memory_top_value: None,
            candidates_generated: 0,
            candidates_rejected: 0,
            last_rejection: None,
            symbolic_state_hash: None,
            symbol_activations: 0,
            symbol_links: 0,
            last_symbolic_symbol_count: 0,
            last_symbolic_edge_count: 0,
            blends_generated: 0,
            principles_extracted: 0,
            last_principle_confidence: 0.0,
            compression_applied: 0,
            last_compression_ratio: 0.0,
            last_resonance_score: 0.0,
            total_score_sum: 0.0,
            total_score_count: 0,
            last_total_score: 0.0,
            scored_cycles: 0,
            social_harmony: 0.7,
            truth_score_sum: 0.0,
            kindness_score_sum: 0.0,
            social_score_sum: 0.0,
            logic_score_sum: 0.0,
            utility_score_sum: 0.0,
            harm_score_sum: 0.0,
            outcome_count: 0,
            matched_expected_count: 0,
            archive_commits: 0,
            last_frame_id: None,
            evidence_entries: 0,
            evidence_tampered: 0,
            evidence_integrity_all_valid: true,
            claims_asserted: 0,
            claims_retrieved: 0,
            claims_with_evidence_links: 0,
            claims_validated: 0,
            claims_superseded: 0,
            contradictions_checked: 0,
            reasoning_audits: 0,
            audits_with_evidence_refs: 0,
            audits_with_claim_refs: 0,
            tools_executed: 0,
            tools_blocked: 0,
            pressure_updates: 0,
            policy_bias_applications: 0,
            last_pressure_safety: 0.0,
            last_pressure_uncertainty: 0.0,
            last_pressure_resource: 0.0,
            last_pressure_contradiction: 0.0,
            last_pressure_evidence_gap: 0.0,
            last_pressure_social_risk: 0.0,
            last_pressure_tool_risk: 0.0,
            last_pressure_urgency: 0.0,
            last_pressure_coherence: 0.0,
            scratchpad_entry_count: 0,
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
