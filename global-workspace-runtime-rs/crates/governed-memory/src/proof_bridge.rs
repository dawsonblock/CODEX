//! Proof bridge: Integration points for governed-memory in runtime proof paths.
//!
//! This module provides instrumentation for the proof command to route
//! claim promotion candidates through the admission gate and collect
//! decision metadata for proof counters.

use crate::admission::MemoryAdmissionDecision;
use crate::enums::*;
use crate::reason_codes::ReasonCode;
use crate::schemas::{AdmissionPolicy, CandidateMemory, Provenance};
use crate::MemoryAdmissionGate;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Stats collected during proof run for governed-memory integration reporting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofGovernedMemoryStats {
    /// Total candidates evaluated for admission
    pub candidates_evaluated: usize,

    /// Candidates admitted as active claims
    pub active_admission_recommendations: usize,

    /// Candidates with verified evidence backing promotion
    pub evidence_backed_promotion_recommendations: usize,

    /// Candidates recommended for evidence-only storage
    pub evidence_only_recommendations: usize,

    /// Candidates explicitly rejected as unverified
    pub rejected_unverified: usize,

    /// Candidates deferred pending additional evidence
    pub deferred_pending_evidence: usize,

    /// Candidates marked as disputed due to contradictions
    pub disputed_recommendations: usize,

    /// Claims actually written to ClaimStore by CODEX (proof authority)
    pub claimstore_writes_performed_by_codex: usize,

    /// Claims written to ClaimStore by governed-memory (should be 0)
    pub claimstore_writes_performed_by_governed_memory: usize,

    /// Reasoning audits that included governed-memory reason codes
    pub audits_with_governed_memory_reason_codes: usize,

    /// Retrieval plans generated and classified
    pub retrieval_plans_generated: usize,

    /// Reason codes collected across all decisions
    pub decision_reason_codes: Vec<ReasonCode>,
}

/// Candidate creator for proof evidence->claim pathway.
///
/// Converts proof-known structured evidence into a CandidateMemory
/// suitable for admission gate evaluation.
pub struct ProofCandidateFactory;

impl ProofCandidateFactory {
    /// Create a candidate from already-asserted claim data.
    ///
    /// Used to retroactively evaluate claims made during proof for
    /// decision tracking without changing the actual claim workflow.
    pub fn from_claim_data(
        claim_id: &str,
        subject: &str,
        predicate: &str,
        object: Option<&str>,
        evidence_id: Option<&str>,
        confidence: f64,
    ) -> CandidateMemory {
        let source_type = if evidence_id.is_some() {
            SourceTrustType::VerifiedEvidence
        } else {
            SourceTrustType::TrustedUser
        };

        let provenance = if let Some(ev_id) = evidence_id {
            Provenance::DirectEvidence {
                evidence_id: ev_id.to_string(),
                vault_entry_hash: format!("proof_hash_{}", ev_id),
            }
        } else {
            Provenance::UserSupplied {
                user_id: "proof_scenario".to_string(),
                session_id: "proof_run".to_string(),
                timestamp: Utc::now(),
            }
        };

        CandidateMemory {
            id: format!("candidate_{}", claim_id),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.map(String::from),
            source_id: evidence_id.unwrap_or("proof_scenario").to_string(),
            source_type,
            confidence,
            created_at: Utc::now(),
            evidence_id: evidence_id.map(String::from),
            provenance,
        }
    }
}

/// Decision tracker for proof scenarios.
pub struct ProofAdmissionTracker {
    gate: MemoryAdmissionGate,
    stats: ProofGovernedMemoryStats,
}

impl ProofAdmissionTracker {
    /// Create a new tracker with default admission policy.
    pub fn new() -> Self {
        Self {
            gate: MemoryAdmissionGate::default_policy(),
            stats: ProofGovernedMemoryStats::default(),
        }
    }

    /// Create with explicit policy.
    pub fn with_policy(policy: AdmissionPolicy) -> Self {
        Self {
            gate: MemoryAdmissionGate::new(policy),
            stats: ProofGovernedMemoryStats::default(),
        }
    }

    /// Evaluate a claim candidate and track the decision.
    ///
    /// Returns the decision and updates internal counters.
    pub fn evaluate_claim(&mut self, candidate: &CandidateMemory) -> MemoryAdmissionDecision {
        self.stats.candidates_evaluated += 1;

        let decision = self.gate.admit(candidate);

        // Track storage location recommendations
        match decision.storage_location.as_str() {
            "active_claim" => {
                self.stats.active_admission_recommendations += 1;
                if candidate.evidence_id.is_some() {
                    self.stats.evidence_backed_promotion_recommendations += 1;
                }
            }
            "pending_evidence" => {
                self.stats.evidence_only_recommendations += 1;
            }
            "rejected" => {
                if candidate.source_type == SourceTrustType::Unverified {
                    self.stats.rejected_unverified += 1;
                } else {
                    self.stats.deferred_pending_evidence += 1;
                }
            }
            _ => {}
        }

        // Track reason codes
        for code in &decision.reason_codes {
            if !self.stats.decision_reason_codes.contains(code) {
                self.stats.decision_reason_codes.push(code.clone());
            }
        }

        decision
    }

    /// Record that CODEX ClaimStore actually wrote a claim.
    pub fn record_claim_written_by_codex(&mut self) {
        self.stats.claimstore_writes_performed_by_codex += 1;
    }

    /// Record an audit that included governed-memory reason codes.
    pub fn record_audit_with_reason_codes(&mut self) {
        self.stats.audits_with_governed_memory_reason_codes += 1;
    }

    /// Get the current stats snapshot.
    pub fn stats(&self) -> &ProofGovernedMemoryStats {
        &self.stats
    }

    /// Take ownership of the stats.
    pub fn into_stats(self) -> ProofGovernedMemoryStats {
        self.stats
    }
}

impl Default for ProofAdmissionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_counts_verified_evidence_candidate() {
        let mut tracker = ProofAdmissionTracker::new();
        let candidate = ProofCandidateFactory::from_claim_data(
            "claim_1",
            "fact",
            "is_true",
            None,
            Some("evidence_1"),
            0.9,
        );

        let decision = tracker.evaluate_claim(&candidate);
        assert!(decision.admitted);
        assert_eq!(tracker.stats().candidates_evaluated, 1);
        assert_eq!(tracker.stats().evidence_backed_promotion_recommendations, 1);
    }

    #[test]
    fn test_tracker_counts_unverified_rejection() {
        let mut tracker = ProofAdmissionTracker::new();
        let mut candidate = ProofCandidateFactory::from_claim_data(
            "claim_2",
            "fact",
            "is_unknown",
            None,
            None,
            0.0,
        );
        candidate.source_type = SourceTrustType::Unverified;

        let decision = tracker.evaluate_claim(&candidate);
        assert!(!decision.admitted);
        assert_eq!(tracker.stats().candidates_evaluated, 1);
        assert_eq!(tracker.stats().rejected_unverified, 1);
    }

    #[test]
    fn test_tracker_records_codex_writes() {
        let mut tracker = ProofAdmissionTracker::new();
        tracker.record_claim_written_by_codex();
        tracker.record_claim_written_by_codex();
        assert_eq!(tracker.stats().claimstore_writes_performed_by_codex, 2);
    }
}
