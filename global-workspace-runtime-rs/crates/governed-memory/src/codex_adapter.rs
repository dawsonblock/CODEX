//! CODEX adapter: convert between CODEX internal types and governed-memory types.
//!
//! This module provides bidirectional conversion functions that integrate
//! governed-memory with CODEX evidence vault, claim store, contradiction engine,
//! and runtime systems. All conversions maintain CODEX authority.
//!
//! NO provider metadata, API keys, or embedding vectors are passed through.

use crate::enums::*;
use crate::reason_codes::ReasonCode;
use crate::schemas::*;
use chrono::Utc;

/// Convert evidence entry ID to a candidate memory.
///
/// Maps from the evidence vault's entry format to a local CandidateMemory
/// for policy evaluation. The hash is preserved for verification.
pub fn evidence_entry_to_candidate(
    entry_id: &str,
    subject: &str,
    predicate: &str,
    object: Option<&str>,
    vault_entry_hash: &str,
) -> CandidateMemory {
    CandidateMemory {
        id: format!("candidate_{}", entry_id),
        subject: subject.to_string(),
        predicate: predicate.to_string(),
        object: object.map(|s| s.to_string()),
        source_id: entry_id.to_string(),
        source_type: SourceTrustType::VerifiedEvidence,
        confidence: 0.9,
        created_at: Utc::now(),
        evidence_id: Some(entry_id.to_string()),
        provenance: Provenance::DirectEvidence {
            evidence_id: entry_id.to_string(),
            vault_entry_hash: vault_entry_hash.to_string(),
        },
    }
}

/// Convert candidate memory to a governed claim record candidate.
///
/// Validates that evidence backing exists before allowing promotion to claim.
/// Returns error if evidence cannot be verified.
pub fn candidate_to_claim_candidate(candidate: &CandidateMemory) -> Result<String, String> {
    // Validation: if source requires evidence, check that evidence_id is present
    if candidate.source_type == SourceTrustType::VerifiedEvidence && candidate.evidence_id.is_none()
    {
        return Err("VerifiedEvidence without evidence_id".to_string());
    }

    // Build a claim record JSON representation
    let claim_json = format!(
        r#"{{"id":"claim_{}","subject":"{}","predicate":"{}","source":"{}","confidence":{}}}"#,
        candidate.id,
        candidate.subject,
        candidate.predicate,
        candidate.source_id,
        candidate.confidence
    );

    Ok(claim_json)
}

/// Convert CODEX MemoryClaim to governed claim record.
///
/// Wraps the claim with audit codes and metadata for proof generation.
#[allow(clippy::too_many_arguments)]
pub fn claim_to_governed_record(
    claim_id: &str,
    subject: &str,
    predicate: &str,
    object: Option<&str>,
    status: &str,
    confidence: f64,
    evidence_ids: Vec<String>,
    reason_codes: Vec<ReasonCode>,
) -> GovernedClaimRecord {
    GovernedClaimRecord {
        claim_id: claim_id.to_string(),
        subject: subject.to_string(),
        predicate: predicate.to_string(),
        object: object.map(|s| s.to_string()),
        status: status.to_string(),
        confidence,
        evidence_ids,
        audit_reasons: reason_codes,
        contradictions: vec![],
        created_at: Utc::now(),
        last_accessed: None,
        retrieval_count: 0,
    }
}

/// Convert claim retrieval result to governed retrieval hits.
///
/// Maps matched claims to RetrievalHit with metadata about why they matched.
pub fn claim_result_to_retrieval_hits(
    query_id: &str,
    matched_claim_ids: Vec<(String, f64, String)>, // (claim_id, match_score, evidence_id_csv)
) -> Vec<RetrievalHit> {
    matched_claim_ids
        .into_iter()
        .enumerate()
        .map(|(idx, (claim_id, score, _evidence_csv))| RetrievalHit {
            hit_id: format!("hit_{}_{}", query_id, idx),
            query_id: query_id.to_string(),
            memory_id: format!("mem_{}", claim_id),
            claim_id: Some(claim_id),
            match_score: score,
            confidence: score,
            retrieval_hit_reason: RetrievalHitReason::DirectMatch,
            reason: "Matched by claim store query".to_string(),
            created_at: Utc::now(),
        })
        .collect()
}

/// Convert contradiction result to governed conflict metadata.
///
/// Records conflict with evidence links on both sides.
pub fn contradiction_to_governed_conflict(
    conflict_id: &str,
    claim_a_id: &str,
    claim_b_id: &str,
    evidence_a_ids: Vec<String>,
    evidence_b_ids: Vec<String>,
) -> GovernedConflictMetadata {
    GovernedConflictMetadata {
        conflict_id: conflict_id.to_string(),
        claim_a_id: claim_a_id.to_string(),
        claim_b_id: claim_b_id.to_string(),
        evidence_a_ids,
        evidence_b_ids,
        nature: ConflictNature::DirectContradiction,
        suggested_resolution: ConflictResolution::InvestigateFurther,
        detected_at: Utc::now(),
    }
}

/// Convert reasoning audit to governed memory audit record.
///
/// Packages admission, retrieval, and conflict decisions into audit trail.
pub fn reasoning_audit_to_governed_record(
    cycle_id: u64,
    queried_facts: Vec<String>,
    admitted_facts: Vec<String>,
    reason_codes: Vec<ReasonCode>,
    actions_selected: Vec<String>,
) -> crate::audit::GovernedMemoryAuditRecord {
    crate::audit::GovernedMemoryAuditRecord {
        audit_id: format!("audit_cycle_{}", cycle_id),
        cycle_id,
        timestamp: Utc::now(),
        queried_facts,
        admitted_facts,
        reason_codes,
        contradictions_detected: vec![],
        actions_selected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_to_candidate() {
        let candidate = evidence_entry_to_candidate(
            "ev_1",
            "weather",
            "is_raining",
            Some("true"),
            "hash_abc123",
        );

        assert_eq!(candidate.subject, "weather");
        assert_eq!(candidate.predicate, "is_raining");
        assert_eq!(candidate.source_type, SourceTrustType::VerifiedEvidence);
        assert_eq!(candidate.confidence, 0.9);
        assert_eq!(candidate.evidence_id, Some("ev_1".to_string()));
    }

    #[test]
    fn test_candidate_to_claim_candidate_verified() {
        let candidate = CandidateMemory {
            id: "c_1".to_string(),
            subject: "fact".to_string(),
            predicate: "holds".to_string(),
            object: None,
            source_id: "ev_1".to_string(),
            source_type: SourceTrustType::VerifiedEvidence,
            confidence: 0.9,
            created_at: Utc::now(),
            evidence_id: Some("ev_1".to_string()),
            provenance: Provenance::DirectEvidence {
                evidence_id: "ev_1".to_string(),
                vault_entry_hash: "hash123".to_string(),
            },
        };

        let result = candidate_to_claim_candidate(&candidate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_candidate_to_claim_candidate_missing_evidence() {
        let candidate = CandidateMemory {
            id: "c_1".to_string(),
            subject: "fact".to_string(),
            predicate: "holds".to_string(),
            object: None,
            source_id: "ev_1".to_string(),
            source_type: SourceTrustType::VerifiedEvidence,
            confidence: 0.9,
            created_at: Utc::now(),
            evidence_id: None, // Missing!
            provenance: Provenance::DirectEvidence {
                evidence_id: "ev_1".to_string(),
                vault_entry_hash: "hash123".to_string(),
            },
        };

        let result = candidate_to_claim_candidate(&candidate);
        assert!(result.is_err());
    }

    #[test]
    fn test_claim_to_governed_record() {
        let reason_codes = vec![ReasonCode::admission_verified_evidence()];
        let governed = claim_to_governed_record(
            "claim_1",
            "weather",
            "is_raining",
            Some("true"),
            "Active",
            0.9,
            vec!["ev_1".to_string()],
            reason_codes,
        );

        assert_eq!(governed.claim_id, "claim_1");
        assert_eq!(governed.subject, "weather");
        assert_eq!(governed.status, "Active");
        assert_eq!(governed.confidence, 0.9);
        assert!(!governed.audit_reasons.is_empty());
    }

    #[test]
    fn test_claim_result_to_retrieval_hits() {
        let claims = vec![
            ("claim_1".to_string(), 0.95, "ev_1".to_string()),
            ("claim_2".to_string(), 0.78, "ev_2".to_string()),
        ];

        let hits = claim_result_to_retrieval_hits("query_1", claims);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].match_score, 0.95);
        assert_eq!(hits[1].match_score, 0.78);
    }
}
