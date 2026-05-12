//! Source trust evaluation and confidence scoring.
//!
//! Determines confidence levels based on source type. Memory source trust affects
//! confidence but does NOT prove truth.

use crate::enums::SourceTrustType;

/// Source trust evaluation.
pub trait SourceTrust {
    /// Get confidence score for this source type (0.0-1.0).
    /// This is confidence in the source, NOT truth confirmation.
    fn confidence(&self) -> f64;

    /// Check if source is verified (evidence vault backed).
    fn is_verified(&self) -> bool;
}

impl SourceTrust for SourceTrustType {
    fn confidence(&self) -> f64 {
        self.default_confidence()
    }

    fn is_verified(&self) -> bool {
        matches!(self, SourceTrustType::VerifiedEvidence)
    }
}

/// Aggregate confidence from multiple sources for the same fact.
///
/// Combines confidence scores without resolving truth ambiguities.
/// Returns average confidence; does NOT use max or min.
pub fn aggregate_confidence(confidences: &[f64]) -> f64 {
    if confidences.is_empty() {
        return 0.0;
    }
    confidences.iter().sum::<f64>() / confidences.len() as f64
}

/// Check if a source is trusted enough to admit into active claims.
pub fn is_source_trusted_for_active_admission(
    source_type: SourceTrustType,
    min_confidence: f64,
) -> bool {
    source_type.confidence() >= min_confidence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_evidence_is_verified() {
        assert!(SourceTrustType::VerifiedEvidence.is_verified());
    }

    #[test]
    fn test_unverified_is_not_verified() {
        assert!(!SourceTrustType::Unverified.is_verified());
    }

    #[test]
    fn test_confidence_scores() {
        assert_eq!(SourceTrustType::VerifiedEvidence.confidence(), 0.9);
        assert_eq!(SourceTrustType::TrustedUser.confidence(), 0.7);
        assert_eq!(SourceTrustType::LowConfidenceSurvey.confidence(), 0.3);
    }

    #[test]
    fn test_aggregate_confidence() {
        let confidences = vec![0.9, 0.7, 0.5];
        assert_eq!(aggregate_confidence(&confidences), (0.9 + 0.7 + 0.5) / 3.0);
    }

    #[test]
    fn test_aggregate_confidence_empty() {
        assert_eq!(aggregate_confidence(&[]), 0.0);
    }

    #[test]
    fn test_source_trusted_for_admission() {
        assert!(is_source_trusted_for_active_admission(
            SourceTrustType::VerifiedEvidence,
            0.6
        ));
        assert!(!is_source_trusted_for_active_admission(
            SourceTrustType::Unverified,
            0.6
        ));
    }
}
