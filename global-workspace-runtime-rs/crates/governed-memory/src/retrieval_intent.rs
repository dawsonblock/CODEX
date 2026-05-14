//! Retrieval intent analysis and routing.
//!
//! Analyzes memory queries and routes them to appropriate action types:
//! retrieve_memory, defer_insufficient_evidence, ask_clarification, defer_provider_unavailable.

use crate::enums::RetrievalIntentCategory;
use crate::reason_codes::ReasonCode;
use crate::schemas::RetrievalQuery;
use serde::{Deserialize, Serialize};

/// Result of retrieval intent analysis: recommended action and reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalDecision {
    /// Inferred intent category
    pub intent: RetrievalIntentCategory,

    /// Recommended action from runtime-core
    pub recommended_action: String, // "retrieve_memory", "defer_insufficient_evidence", "ask_clarification", "defer_provider_unavailable"

    /// Confidence in this routing (0.0-1.0)
    pub confidence_in_route: f64,

    /// Reason codes for the decision
    pub reason_codes: Vec<ReasonCode>,
}

/// Retrieval intent router.
/// Categorizes queries and routes them to appropriate actions.
pub struct RetrievalRouter;

impl RetrievalRouter {
    /// Analyze query and recommend action.
    pub fn route(query: &RetrievalQuery) -> RetrievalDecision {
        let mut reason_codes = vec![];
        let (intent, action, confidence_in_route) = match &query.intent_category {
            RetrievalIntentCategory::MemoryLookup => {
                reason_codes.push(ReasonCode::retrieval_memory_lookup());
                (
                    RetrievalIntentCategory::MemoryLookup,
                    "retrieve_memory",
                    0.95,
                )
            }
            RetrievalIntentCategory::UnsupportedFactual => {
                reason_codes.push(ReasonCode::retrieval_unsupported_factual());
                (
                    RetrievalIntentCategory::UnsupportedFactual,
                    "defer_insufficient_evidence",
                    0.9,
                )
            }
            RetrievalIntentCategory::HighStakesLowEvidence => {
                reason_codes.push(ReasonCode::retrieval_high_stakes_low_evidence());
                (
                    RetrievalIntentCategory::HighStakesLowEvidence,
                    "defer_insufficient_evidence",
                    0.85,
                )
            }
            RetrievalIntentCategory::Ambiguous => {
                reason_codes.push(ReasonCode::retrieval_ambiguous_match());
                (RetrievalIntentCategory::Ambiguous, "ask_clarification", 0.8)
            }
            RetrievalIntentCategory::ProviderGated => {
                reason_codes.push(ReasonCode::retrieval_provider_gated());
                (
                    RetrievalIntentCategory::ProviderGated,
                    "defer_provider_unavailable",
                    0.95,
                )
            }
        };

        RetrievalDecision {
            intent,
            recommended_action: action.to_string(),
            confidence_in_route,
            reason_codes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_memory_lookup() {
        let query = RetrievalQuery {
            query_id: "q1".to_string(),
            query_text: "do I know X?".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::MemoryLookup,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.6,
            created_at: chrono::Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "retrieve_memory");
        assert_eq!(decision.confidence_in_route, 0.95);
    }

    #[test]
    fn test_route_unsupported_factual() {
        let query = RetrievalQuery {
            query_id: "q2".to_string(),
            query_text: "what is new_thing?".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::UnsupportedFactual,
            requires_verification: true,
            max_candidates: 5,
            confidence_threshold: 0.8,
            created_at: chrono::Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "defer_insufficient_evidence");
    }

    #[test]
    fn test_route_provider_gated() {
        let query = RetrievalQuery {
            query_id: "q3".to_string(),
            query_text: "find latest news".to_string(),
            context: None,
            intent_category: RetrievalIntentCategory::ProviderGated,
            requires_verification: false,
            max_candidates: 10,
            confidence_threshold: 0.5,
            created_at: chrono::Utc::now(),
        };

        let decision = RetrievalRouter::route(&query);
        assert_eq!(decision.recommended_action, "defer_provider_unavailable");
        assert_eq!(decision.confidence_in_route, 0.95);
    }
}
