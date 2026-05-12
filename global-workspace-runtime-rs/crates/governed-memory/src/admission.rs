//! Memory admission policy and gating logic.
//!
//! Implements the decision rules for accepting candidates into the claim store,
//! storing as pending evidence, or rejecting outright.

use crate::enums::*;
use crate::reason_codes::ReasonCode;
use crate::schemas::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Decision result from admission gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAdmissionDecision {
    /// Was the candidate admitted?
    pub admitted: bool,

    /// Admitted as active claim, pending evidence, or archived/rejected?
    pub storage_location: String,

    /// Unique ID of the source candidate
    pub candidate_id: String,

    /// If admitted, the claim ID created (if any)
    pub resulting_claim_id: Option<String>,

    /// Reason for the decision
    pub reason: AdmissionDecisionReason,

    /// Audit codes for this decision
    pub reason_codes: Vec<ReasonCode>,

    /// Confidence score of the resulting claim (if admitted)
    pub confidence: f64,

    /// Evidence weight (count of supporting evidence)
    pub evidence_weight: u32,

    /// When this decision was made
    pub timestamp: chrono::DateTime<Utc>,
}

/// Memory admission gate.
///
/// Evaluates candidates and routes them to active claims, pending evidence,
/// or rejection based on evidence, source trust, and policy rules.
pub struct MemoryAdmissionGate {
    /// Admission policy rules
    pub policy: AdmissionPolicy,
}

impl MemoryAdmissionGate {
    /// Create a new admission gate with given policy.
    pub fn new(policy: AdmissionPolicy) -> Self {
        Self { policy }
    }

    /// Create with default policy.
    pub fn default_policy() -> Self {
        Self::new(AdmissionPolicy::default())
    }

    /// Admit a candidate based on policy rules.
    ///
    /// Returns a `MemoryAdmissionDecision` with storage location and reason codes.
    pub fn admit(&self, candidate: &CandidateMemory) -> MemoryAdmissionDecision {
        // Rule 1: Unsupported raw text
        if candidate.source_type == SourceTrustType::Unverified {
            return self.reject_decision(
                candidate,
                AdmissionDecisionReason::InsufficientEvidence,
                ReasonCode::admission_unsupported_raw_text(),
            );
        }

        // Rule 2: Verified evidence
        if candidate.source_type == SourceTrustType::VerifiedEvidence {
            return self.admit_decision(
                candidate,
                "active_claim",
                AdmissionDecisionReason::VerifiedByEvidence,
                ReasonCode::admission_verified_evidence(),
                0.9,
                1,
            );
        }

        // Rule 3: Trusted user input
        if candidate.source_type == SourceTrustType::TrustedUser {
            return self.admit_decision(
                candidate,
                "active_claim",
                AdmissionDecisionReason::TrustedInput,
                ReasonCode::admission_trusted_input(),
                0.7,
                1,
            );
        }

        // Rule 4: Low confidence survey -> pending evidence only
        if candidate.source_type == SourceTrustType::LowConfidenceSurvey {
            return self.pending_decision(
                candidate,
                AdmissionDecisionReason::InsufficientEvidence,
                ReasonCode::admission_low_source_trust(),
                0.3,
            );
        }

        // Rule 5: Default rejection (contradiction suspect, unverified, etc.)
        self.reject_decision(
            candidate,
            AdmissionDecisionReason::InsufficientEvidence,
            ReasonCode::admission_insufficient_evidence(),
        )
    }

    /// Build an admission decision (active claim).
    fn admit_decision(
        &self,
        candidate: &CandidateMemory,
        location: &str,
        reason: AdmissionDecisionReason,
        code: ReasonCode,
        confidence: f64,
        evidence_weight: u32,
    ) -> MemoryAdmissionDecision {
        let claim_id = format!("claim_{}", candidate.id);
        MemoryAdmissionDecision {
            admitted: true,
            storage_location: location.to_string(),
            candidate_id: candidate.id.clone(),
            resulting_claim_id: Some(claim_id),
            reason,
            reason_codes: vec![code],
            confidence,
            evidence_weight,
            timestamp: Utc::now(),
        }
    }

    /// Build a pending evidence decision (not active claim).
    fn pending_decision(
        &self,
        candidate: &CandidateMemory,
        reason: AdmissionDecisionReason,
        code: ReasonCode,
        confidence: f64,
    ) -> MemoryAdmissionDecision {
        MemoryAdmissionDecision {
            admitted: false,
            storage_location: "pending_evidence".to_string(),
            candidate_id: candidate.id.clone(),
            resulting_claim_id: None,
            reason,
            reason_codes: vec![code],
            confidence,
            evidence_weight: 1,
            timestamp: Utc::now(),
        }
    }

    /// Build a rejection decision.
    fn reject_decision(
        &self,
        candidate: &CandidateMemory,
        reason: AdmissionDecisionReason,
        code: ReasonCode,
    ) -> MemoryAdmissionDecision {
        MemoryAdmissionDecision {
            admitted: false,
            storage_location: "rejected".to_string(),
            candidate_id: candidate.id.clone(),
            resulting_claim_id: None,
            reason,
            reason_codes: vec![code],
            confidence: 0.0,
            evidence_weight: 0,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admission_unsupported_raw_text() {
        let gate = MemoryAdmissionGate::default_policy();
        let candidate = CandidateMemory {
            id: "c1".to_string(),
            subject: "fact".to_string(),
            predicate: "is_true".to_string(),
            object: None,
            source_id: "unknown".to_string(),
            source_type: SourceTrustType::Unverified,
            confidence: 0.0,
            created_at: Utc::now(),
            evidence_id: None,
            provenance: Provenance::Contradicted,
        };

        let decision = gate.admit(&candidate);
        assert!(!decision.admitted);
        assert_eq!(decision.storage_location, "rejected");
    }

    #[test]
    fn test_admission_verified_evidence() {
        let gate = MemoryAdmissionGate::default_policy();
        let candidate = CandidateMemory {
            id: "c1".to_string(),
            subject: "fact".to_string(),
            predicate: "is_true".to_string(),
            object: None,
            source_id: "evidence_1".to_string(),
            source_type: SourceTrustType::VerifiedEvidence,
            confidence: 0.9,
            created_at: Utc::now(),
            evidence_id: Some("evidence_1".to_string()),
            provenance: Provenance::DirectEvidence {
                evidence_id: "evidence_1".to_string(),
                vault_entry_hash: "hash123".to_string(),
            },
        };

        let decision = gate.admit(&candidate);
        assert!(decision.admitted);
        assert_eq!(decision.storage_location, "active_claim");
        assert!(decision.resulting_claim_id.is_some());
        assert_eq!(decision.confidence, 0.9);
    }

    #[test]
    fn test_admission_low_confidence_survey() {
        let gate = MemoryAdmissionGate::default_policy();
        let candidate = CandidateMemory {
            id: "c1".to_string(),
            subject: "survey_fact".to_string(),
            predicate: "observation".to_string(),
            object: None,
            source_id: "survey".to_string(),
            source_type: SourceTrustType::LowConfidenceSurvey,
            confidence: 0.3,
            created_at: Utc::now(),
            evidence_id: None,
            provenance: Provenance::UserSupplied {
                user_id: "user1".to_string(),
                session_id: "sess1".to_string(),
                timestamp: Utc::now(),
            },
        };

        let decision = gate.admit(&candidate);
        assert!(!decision.admitted);
        assert_eq!(decision.storage_location, "pending_evidence");
    }
}
