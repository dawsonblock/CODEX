//! Assertion conflict handling (renamed from belief_conflict).
//!
//! Records and manages conflicts between memory assertions.
//! Note: "belief" in this context means a structured memory assertion record,
//! not a subjective belief or consciousness claim.

use crate::enums::*;
use crate::schemas::GovernedConflictMetadata;
use chrono::Utc;

/// Assertion conflict handler (renamed from belief_conflict_resolver for clarity).
pub struct AssertionConflictHandler;

impl AssertionConflictHandler {
    /// Record a conflict between two claims.
    pub fn record_conflict(
        claim_a_id: &str,
        claim_b_id: &str,
        evidence_a_ids: Vec<String>,
        evidence_b_ids: Vec<String>,
        nature: ConflictNature,
    ) -> GovernedConflictMetadata {
        let suggested_resolution = match nature {
            ConflictNature::DirectContradiction => ConflictResolution::InvestigateFurther,
            ConflictNature::BoundaryCase => ConflictResolution::AcceptBothDisputed,
            ConflictNature::ProvisionalDisagreement => ConflictResolution::InvestigateFurther,
        };

        GovernedConflictMetadata {
            conflict_id: format!("conflict_{}_{}", claim_a_id, claim_b_id),
            claim_a_id: claim_a_id.to_string(),
            claim_b_id: claim_b_id.to_string(),
            evidence_a_ids,
            evidence_b_ids,
            nature,
            suggested_resolution,
            detected_at: Utc::now(),
        }
    }

    /// Mark a claim as disputed due to conflict (does not resolve).
    pub fn mark_as_disputed(conflict_id: &str) -> String {
        format!(
            "claim_status UPDATE disputed WHERE conflict_id={}",
            conflict_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_direct_contradiction() {
        let conflict = AssertionConflictHandler::record_conflict(
            "claim_1",
            "claim_2",
            vec!["evidence_1".to_string()],
            vec!["evidence_2".to_string()],
            ConflictNature::DirectContradiction,
        );

        assert_eq!(conflict.claim_a_id, "claim_1");
        assert_eq!(conflict.claim_b_id, "claim_2");
        assert_eq!(conflict.nature, ConflictNature::DirectContradiction);
        assert_eq!(
            conflict.suggested_resolution,
            ConflictResolution::InvestigateFurther
        );
    }

    #[test]
    fn test_record_boundary_case() {
        let conflict = AssertionConflictHandler::record_conflict(
            "claim_a",
            "claim_b",
            vec!["ev_a".to_string()],
            vec!["ev_b".to_string()],
            ConflictNature::BoundaryCase,
        );

        assert_eq!(conflict.nature, ConflictNature::BoundaryCase);
        assert_eq!(
            conflict.suggested_resolution,
            ConflictResolution::AcceptBothDisputed
        );
    }
}
