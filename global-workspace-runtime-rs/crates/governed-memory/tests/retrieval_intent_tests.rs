//! Retrieval intent routing tests.
//!
//! Tests for routing queries to appropriate actions:
//! - Memory lookup routes to retrieve_memory
//! - Unsupported factual routes to defer_insufficient_evidence
//! - High-stakes low-evidence routes to defer or ask
//! - Ambiguous routes to ask_clarification
//! - Provider-gated routes to defer_provider_unavailable

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use governed_memory::*;

    #[test]
    fn test_retrieval_intent_memory_lookup_routes() {
        let query = RetrievalQuery {
            query_id: "q_1".to_string(),
            query_text: "do I know X?".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::MemoryLookup,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.6,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "retrieve_memory");
        assert_eq!(decision.intent, RetrievalIntentCategory::MemoryLookup);
    }

    #[test]
    fn test_retrieval_intent_unsupported_factual_routes() {
        let query = RetrievalQuery {
            query_id: "q_2".to_string(),
            query_text: "what is new_fact?".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::UnsupportedFactual,
            requires_verification: true,
            max_candidates: 5,
            confidence_threshold: 0.8,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "defer_insufficient_evidence");
    }

    #[test]
    fn test_retrieval_intent_high_stakes_low_evidence_routes() {
        let query = RetrievalQuery {
            query_id: "q_3".to_string(),
            query_text: "should I take drug X?".to_string(),
            context: Some("medical decision".to_string()),
            intent_category: RetrievalIntentCategory::HighStakesLowEvidence,
            requires_verification: true,
            max_candidates: 10,
            confidence_threshold: 0.9,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "defer_insufficient_evidence");
    }

    #[test]
    fn test_retrieval_intent_ambiguous_routes() {
        let query = RetrievalQuery {
            query_id: "q_4".to_string(),
            query_text: "what is X?".to_string(),
            context: Some("could be multiple meanings".to_string()),
            intent_category: RetrievalIntentCategory::Ambiguous,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.6,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "ask_clarification");
    }

    #[test]
    fn test_retrieval_intent_provider_gated_routes() {
        let query = RetrievalQuery {
            query_id: "q_5".to_string(),
            query_text: "find latest news".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::ProviderGated,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.5,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "defer_provider_unavailable");
        assert_eq!(decision.confidence_in_route, 0.95);
    }

    #[test]
    fn test_retrieval_router_produces_reason_codes() {
        let query = RetrievalQuery {
            query_id: "q_6".to_string(),
            query_text: "test".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::MemoryLookup,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.6,
            created_at: Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert!(!decision.reason_codes.is_empty());
        assert_eq!(decision.reason_codes[0].code, "RETRIEVAL_MEMORY_LOOKUP");
    }
}
