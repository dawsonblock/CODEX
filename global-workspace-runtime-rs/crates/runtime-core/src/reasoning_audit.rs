//! Reasoning audit — per-cycle decision trace.
//!
//! Records why each action was selected. Produces a human-readable trace:
//! observation → memory hits → activated symbols → candidates →
//! rejected candidates → selected action → rationale.
//!
//! # Honesty boundaries
//!
//! - The audit does NOT prove reasoning. It records selection steps.
//! - The audit is NOT explainability. It is a structured log.
//! - The system does NOT understand why it acted.

use crate::action::ActionType;
use serde::{Deserialize, Serialize};

/// One step in the reasoning chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub decision: Option<String>,
}

/// Full reasoning audit for one cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningAudit {
    pub cycle_id: u64,
    pub observation: String,
    pub memory_hits: Vec<String>,
    pub activated_symbols: Vec<String>,
    pub candidates: Vec<String>,
    pub rejected_candidates: Vec<(String, String)>,
    pub selected_action: String,
    pub rationale: String,
    /// Evidence IDs backing this decision.
    pub evidence_ids: Vec<String>,
}

impl ReasoningAudit {
    /// Create an audit trace for a cycle.
    pub fn new(
        cycle_id: u64,
        observation: impl Into<String>,
        selected_action: ActionType,
        rationale: impl Into<String>,
    ) -> Self {
        ReasoningAudit {
            cycle_id,
            observation: observation.into(),
            memory_hits: Vec::new(),
            activated_symbols: Vec::new(),
            candidates: Vec::new(),
            rejected_candidates: Vec::new(),
            selected_action: format!("{}", selected_action),
            rationale: rationale.into(),
            evidence_ids: Vec::new(),
        }
    }

    /// Add memory hits.
    pub fn with_memory_hits(mut self, hits: Vec<String>) -> Self {
        self.memory_hits = hits;
        self
    }

    /// Add activated symbols.
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.activated_symbols = symbols;
        self
    }

    /// Add candidates considered.
    pub fn with_candidates(mut self, candidates: Vec<String>) -> Self {
        self.candidates = candidates;
        self
    }

    /// Add rejected candidates with reasons.
    pub fn with_rejections(mut self, rejected: Vec<(String, String)>) -> Self {
        self.rejected_candidates = rejected;
        self
    }

    /// Add evidence IDs backing this decision.
    pub fn with_evidence(mut self, ids: Vec<String>) -> Self {
        self.evidence_ids = ids;
        self
    }

    /// Format as human-readable text.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Cycle {}:\n", self.cycle_id));
        out.push_str(&format!("  Observation: {}\n", self.observation));
        if !self.memory_hits.is_empty() {
            out.push_str(&format!("  Memory hits: {}\n", self.memory_hits.join(", ")));
        }
        if !self.activated_symbols.is_empty() {
            out.push_str(&format!(
                "  Symbols: {}\n",
                self.activated_symbols.join(", ")
            ));
        }
        if !self.rejected_candidates.is_empty() {
            let r: Vec<String> = self
                .rejected_candidates
                .iter()
                .map(|(a, r)| format!("{} ({})", a, r))
                .collect();
            out.push_str(&format!("  Rejected: {}\n", r.join(", ")));
        }
        if !self.evidence_ids.is_empty() {
            out.push_str(&format!("  Evidence: {}\n", self.evidence_ids.join(", ")));
        }
        out.push_str(&format!("  Selected: {}\n", self.selected_action));
        out.push_str(&format!("  Rationale: {}\n", self.rationale));
        out
    }
}

/// Collection of audits across cycles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditReport {
    pub audits: Vec<ReasoningAudit>,
}

impl AuditReport {
    pub fn new() -> Self {
        AuditReport { audits: Vec::new() }
    }

    pub fn add(&mut self, audit: ReasoningAudit) {
        self.audits.push(audit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_produces_human_readable_text() {
        let audit = ReasoningAudit::new(
            1,
            "factual_query",
            ActionType::Answer,
            "observation is a factual query",
        )
        .with_memory_hits(vec!["memory:fact_store".into()])
        .with_symbols(vec!["factual_context".into()])
        .with_rejections(vec![(
            "internal_diagnostic".into(),
            "not_user_facing".into(),
        )]);

        let text = audit.to_text();
        assert!(text.contains("Cycle 1"));
        assert!(text.contains("factual_query"));
        assert!(text.contains("fact_store"));
        assert!(text.contains("Rationale"));
    }

    #[test]
    fn empty_rationale_still_produces_text() {
        let audit = ReasoningAudit::new(0, "test_obs", ActionType::NoOp, "");
        let text = audit.to_text();
        assert!(!text.is_empty());
        assert!(text.contains("no_op"));
    }

    #[test]
    fn audit_report_collects_all() {
        let mut report = AuditReport::new();
        report.add(ReasoningAudit::new(
            0,
            "obs1",
            ActionType::Summarize,
            "summary needed",
        ));
        report.add(ReasoningAudit::new(
            1,
            "obs2",
            ActionType::Answer,
            "factual",
        ));
        assert_eq!(report.audits.len(), 2);
    }
}
