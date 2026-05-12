//! Policy rules and decision logic for memory governance.
//!
//! Implements the admission, retrieval, and conflict policies that govern
//! how memories are accepted, routed, and resolved.

use crate::schemas::*;

/// Admission policy decision rules.
impl AdmissionPolicy {
    /// Evaluate if a candidate meets minimum confidence threshold.
    pub fn passes_confidence_check(&self, confidence: f64) -> bool {
        confidence >= self.min_confidence_for_active
    }

    /// Evaluate if custom rules should allow admission.
    ///
    /// In this basic implementation, all custom rules default to pass (no custom logic).
    /// Override via subclassing or callback in real usage.
    pub fn passes_custom_rules(&self) -> bool {
        // Placeholder: no custom rules block admission by default
        true
    }

    /// Combined check: confidence + custom rules.
    pub fn allows_admission(&self, confidence: f64) -> bool {
        self.passes_confidence_check(confidence) && self.passes_custom_rules()
    }
}

/// Retrieval policy decision rules.
impl RetrievalPolicy {
    /// Check if query confidence is high-stakes critical.
    pub fn is_high_stakes_query(&self, confidence: f64) -> bool {
        confidence < self.high_stakes_threshold
    }

    /// Check if custom rules block retrieval.
    pub fn passes_custom_rules(&self) -> bool {
        // Placeholder: no custom rules block retrieval by default
        true
    }

    /// Combined check for retrieval allowance.
    pub fn allows_retrieval(&self) -> bool {
        self.passes_custom_rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admission_policy_confidence_check() {
        let policy = AdmissionPolicy::default();
        assert!(policy.passes_confidence_check(0.7));
        assert!(!policy.passes_confidence_check(0.5));
    }

    #[test]
    fn test_retrieval_policy_high_stakes() {
        let policy = RetrievalPolicy::default();
        assert!(policy.is_high_stakes_query(0.4));
        assert!(!policy.is_high_stakes_query(0.8));
    }
}
