//! CODEX adapter tests.
//!
//! Tests for bidirectional conversion between CODEX types and governed-memory:
//! - Evidence entry to candidate preserves hash
//! - Candidate to claim validates evidence exists
//! - Claim to governed record includes audit codes
//! - Retrieval result to hits preserves links
//! - No provider metadata in conversions
//! - Contradiction to conflict metadata includes evidence

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use governed_memory::*;

    #[test]
    fn test_adapter_evidence_to_candidate_preserves_hash() {
        let candidate = evidence_entry_to_candidate(
            "evidence_1",
            "weather",
            "is_raining",
            Some("true"),
            "hash_vault_abc123",
        );

        assert_eq!(candidate.subject, "weather");
        assert_eq!(candidate.predicate, "is_raining");
        assert_eq!(candidate.evidence_id, Some("evidence_1".to_string()));

        // Check provenance preserves hash
        match &candidate.provenance {
            Provenance::DirectEvidence {
                evidence_id,
                vault_entry_hash,
            } => {
                assert_eq!(evidence_id, "evidence_1");
                assert_eq!(vault_entry_hash, "hash_vault_abc123");
            }
            _ => panic!("Expected DirectEvidence provenance"),
        }
    }

    #[test]
    fn test_adapter_candidate_to_claim_validates_evidence_exists() {
        let candidate_with_evidence = CandidateMemory {
            id: "c_1".to_string(),
            subject: "fact".to_string(),
            predicate: "verified".to_string(),
            object: None,
            source_id: "evidence_1".to_string(),
            source_type: SourceTrustType::VerifiedEvidence,
            confidence: 0.9,
            created_at: Utc::now(),
            evidence_id: Some("evidence_1".to_string()),
            provenance: Provenance::DirectEvidence {
                evidence_id: "evidence_1".to_string(),
                vault_entry_hash: "hash_x".to_string(),
            },
        };

        let result = candidate_to_claim_candidate(&candidate_with_evidence);
        assert!(result.is_ok());

        // Now test missing evidence
        let candidate_missing_evidence = CandidateMemory {
            id: "c_2".to_string(),
            subject: "fact".to_string(),
            predicate: "unverified".to_string(),
            object: None,
            source_id: "evidence_missing".to_string(),
            source_type: SourceTrustType::VerifiedEvidence,
            confidence: 0.9,
            created_at: Utc::now(),
            evidence_id: None, // Missing!
            provenance: Provenance::Contradicted,
        };

        let result = candidate_to_claim_candidate(&candidate_missing_evidence);
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_claim_to_governed_record_includes_audit_codes() {
        let reason_codes = vec![
            ReasonCode::admission_verified_evidence(),
            ReasonCode::admission_duplicate_evidence(),
        ];

        let governed = claim_to_governed_record(
            "claim_1",
            "subject",
            "predicate",
            Some("object"),
            "Active",
            0.9,
            vec!["evidence_1".to_string()],
            reason_codes,
        );

        assert_eq!(governed.claim_id, "claim_1");
        assert_eq!(governed.status, "Active");
        assert_eq!(governed.confidence, 0.9);
        assert!(!governed.audit_reasons.is_empty());
        assert_eq!(governed.audit_reasons.len(), 2);
    }

    #[test]
    fn test_adapter_retrieval_result_to_hits_preserves_links() {
        let claims = vec![
            ("claim_1".to_string(), 0.95, "ev_1".to_string()),
            ("claim_2".to_string(), 0.78, "ev_2".to_string()),
        ];

        let hits = claim_result_to_retrieval_hits("query_1", claims);

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].claim_id, Some("claim_1".to_string()));
        assert_eq!(hits[0].match_score, 0.95);
        assert_eq!(hits[1].claim_id, Some("claim_2".to_string()));
        assert_eq!(hits[1].match_score, 0.78);
    }

    #[test]
    fn test_adapter_no_provider_metadata_in_conversion() {
        let candidate = evidence_entry_to_candidate("ev_1", "fact", "true", None, "hash_x");

        // Serialize and check no API keys, embeddings, etc.
        let json_str = serde_json::to_string(&candidate).expect("serialize");
        assert!(!json_str.contains("api_key"));
        assert!(!json_str.contains("embedding"));
        assert!(!json_str.contains("model_name"));
        assert!(!json_str.contains("provider"));
    }

    #[test]
    fn test_adapter_contradiction_to_conflict_metadata() {
        let conflict = contradiction_to_governed_conflict(
            "conflict_1",
            "claim_a",
            "claim_b",
            vec!["ev_a1".to_string(), "ev_a2".to_string()],
            vec!["ev_b1".to_string()],
        );

        assert_eq!(conflict.conflict_id, "conflict_1");
        assert_eq!(conflict.claim_a_id, "claim_a");
        assert_eq!(conflict.claim_b_id, "claim_b");
        assert_eq!(conflict.evidence_a_ids.len(), 2);
        assert_eq!(conflict.evidence_b_ids.len(), 1);
        assert_eq!(conflict.nature, ConflictNature::DirectContradiction);
    }

    #[test]
    fn test_adapter_reasoning_audit_to_governed_record() {
        let reason_codes = vec![
            ReasonCode::admission_verified_evidence(),
            ReasonCode::retrieval_memory_lookup(),
        ];
        let actions = vec!["answer".to_string(), "retrieve_memory".to_string()];

        let audit = reasoning_audit_to_governed_record(
            42,
            vec!["query_1".to_string()],
            vec!["fact_1".to_string()],
            reason_codes,
            actions,
        );

        assert_eq!(audit.cycle_id, 42);
        assert_eq!(audit.queried_facts.len(), 1);
        assert_eq!(audit.admitted_facts.len(), 1);
        assert!(!audit.reason_codes.is_empty());
        assert_eq!(audit.actions_selected.len(), 2);
    }
}
