//! Retrieval planning and query optimization.
//!
//! Optimizes memory retrieval queries and plans retrieval strategy.

use crate::schemas::RetrievalQuery;
use serde::{Deserialize, Serialize};

/// Retrieval plan after analyzing query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPlan {
    pub query_id: String,
    pub search_strategy: String, // "exact_match", "semantic_contains", "history_scan"
    pub max_candidates: usize,
    pub confidence_threshold: f64,
    pub include_disputed: bool,
    pub include_archived: bool,
}

/// Retrieval planner.
/// Plans retrieval strategy based on query properties.
pub struct RetrievalPlanner;

impl RetrievalPlanner {
    /// Plan retrieval based on query.
    pub fn plan(query: &RetrievalQuery) -> RetrievalPlan {
        let search_strategy = if query.context.is_some() {
            "semantic_contains".to_string()
        } else {
            "exact_match".to_string()
        };

        RetrievalPlan {
            query_id: query.query_id.clone(),
            search_strategy,
            max_candidates: query.max_candidates,
            confidence_threshold: query.confidence_threshold,
            include_disputed: false,
            include_archived: false,
        }
    }

    /// Readonly retrieve from store (never mutates).
    /// This is read-only by design; applies the plan to execute retrieval.
    pub fn execute_readonly(plan: &RetrievalPlan) -> String {
        format!(
            "SELECT FROM claims WHERE strategy={}, threshold={}, max={}",
            plan.search_strategy, plan.confidence_threshold, plan.max_candidates
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_plan_exact_match() {
        let query = RetrievalQuery {
            query_id: "q1".to_string(),
            query_text: "is X true?".to_string(),
            context: None,
            intent_category: crate::enums::RetrievalIntentCategory::MemoryLookup,
            requires_verification: false,
            max_candidates: 5,
            confidence_threshold: 0.6,
            created_at: Utc::now(),
        };

        let plan = RetrievalPlanner::plan(&query);
        assert_eq!(plan.search_strategy, "exact_match");
    }

    #[test]
    fn test_plan_semantic_with_context() {
        let query = RetrievalQuery {
            query_id: "q2".to_string(),
            query_text: "is X true?".to_string(),
            context: Some("recent observations".to_string()),
            intent_category: crate::enums::RetrievalIntentCategory::MemoryLookup,
            requires_verification: false,
            max_candidates: 5,
            confidence_threshold: 0.6,
            created_at: Utc::now(),
        };

        let plan = RetrievalPlanner::plan(&query);
        assert_eq!(plan.search_strategy, "semantic_contains");
    }
}
