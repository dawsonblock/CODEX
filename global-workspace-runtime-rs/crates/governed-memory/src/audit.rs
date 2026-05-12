//! Governed memory audit records and logging.
//!
//! Records admission, retrieval, and conflict decisions for proof, audit trails,
//! and reasoning transparency.

use crate::reason_codes::ReasonCode;
use crate::schemas::GovernedConflictMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Single memory audit entry (basic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAuditRecord {
    pub audit_id: String,
    pub cycle_id: u64,
    pub action: String, // "admission", "retrieval", "conflict"
    pub subject: String,
    pub decision: String, // "admit", "defer", "reject", "mark_disputed", etc.
    pub reason_codes: Vec<ReasonCode>,
    pub timestamp: DateTime<Utc>,
}

/// Comprehensive governance audit record.
/// Integrates admission, retrieval, and conflict decisions for a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedMemoryAuditRecord {
    pub audit_id: String,
    pub cycle_id: u64,
    pub timestamp: DateTime<Utc>,

    /// Facts queried in this cycle
    pub queried_facts: Vec<String>,

    /// Facts admitted to active claims
    pub admitted_facts: Vec<String>,

    /// Decision reason codes
    pub reason_codes: Vec<ReasonCode>,

    /// Conflicts detected and marked
    pub contradictions_detected: Vec<GovernedConflictMetadata>,

    /// Actions selected by runtime (for proof that no bad actions occurred)
    pub actions_selected: Vec<String>,
}

impl GovernedMemoryAuditRecord {
    /// Create a new audit record for a cycle.
    pub fn new(cycle_id: u64) -> Self {
        Self {
            audit_id: format!("audit_{}", Utc::now().timestamp_millis()),
            cycle_id,
            timestamp: Utc::now(),
            queried_facts: vec![],
            admitted_facts: vec![],
            reason_codes: vec![],
            contradictions_detected: vec![],
            actions_selected: vec![],
        }
    }

    /// Add a reason code to this audit record.
    pub fn add_reason_code(&mut self, code: ReasonCode) {
        self.reason_codes.push(code);
    }

    /// Record a query.
    pub fn record_query(&mut self, fact: String) {
        self.queried_facts.push(fact);
    }

    /// Record admission.
    pub fn record_admission(&mut self, fact: String) {
        self.admitted_facts.push(fact);
    }

    /// Record a detected contradiction.
    pub fn record_contradiction(&mut self, conflict: GovernedConflictMetadata) {
        self.contradictions_detected.push(conflict);
    }

    /// Record an action selected by runtime.
    pub fn record_action(&mut self, action: String) {
        self.actions_selected.push(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_audit_record() {
        let record = GovernedMemoryAuditRecord::new(42);
        assert_eq!(record.cycle_id, 42);
        assert_eq!(record.queried_facts.len(), 0);
    }

    #[test]
    fn test_record_query() {
        let mut record = GovernedMemoryAuditRecord::new(42);
        record.record_query("is_raining".to_string());
        assert_eq!(record.queried_facts.len(), 1);
        assert_eq!(record.queried_facts[0], "is_raining");
    }

    #[test]
    fn test_record_action() {
        let mut record = GovernedMemoryAuditRecord::new(42);
        record.record_action("answer".to_string());
        assert_eq!(record.actions_selected.len(), 1);
        assert_eq!(record.actions_selected[0], "answer");
    }
}
