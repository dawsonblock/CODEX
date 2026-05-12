//! Source trust evaluation tests.
//!
//! Tests for source trust classification and confidence scoring:
//! - Verified evidence has high confidence
//! - Trusted user has medium confidence
//! - Low confidence survey has low confidence
//! - Source trust affects confidence but does not prove truth
//! - Aggregate confidence from multiple sources

#[cfg(test)]
mod tests {
    use governed_memory::*;

    #[test]
    fn test_source_trust_verified_evidence_high_confidence() {
        assert_eq!(SourceTrustType::VerifiedEvidence.confidence(), 0.9);
    }

    #[test]
    fn test_source_trust_trusted_user_medium_confidence() {
        assert_eq!(SourceTrustType::TrustedUser.confidence(), 0.7);
    }

    #[test]
    fn test_source_trust_low_confidence_survey_low() {
        assert_eq!(SourceTrustType::LowConfidenceSurvey.confidence(), 0.3);
    }

    #[test]
    fn test_source_trust_unverified_zero_confidence() {
        assert_eq!(SourceTrustType::Unverified.confidence(), 0.0);
    }

    #[test]
    fn test_source_trust_affects_confidence_not_truth() {
        // High source trust does NOT guarantee truth
        let high_trust_confidence = SourceTrustType::TrustedUser.confidence();
        assert_eq!(high_trust_confidence, 0.7);
        // Confidence is confidence IN SOURCE, not truth confirmation
    }

    #[test]
    fn test_aggregate_confidence_multiple_sources() {
        let confidences = vec![0.9, 0.7, 0.5];
        let avg = aggregate_confidence(&confidences);
        let expected = (0.9 + 0.7 + 0.5) / 3.0;
        assert!((avg - expected).abs() < 0.0001);
    }

    #[test]
    fn test_aggregate_confidence_uses_average_not_max() {
        let confidences = vec![0.9, 0.1, 0.1];
        let avg = aggregate_confidence(&confidences);
        // Should be 0.367, not 0.9 (max)
        assert!(avg < 0.4);
        assert!(avg > 0.36);
    }

    #[test]
    fn test_is_source_trusted_for_active_admission() {
        assert!(is_source_trusted_for_active_admission(
            SourceTrustType::VerifiedEvidence,
            0.6
        ));
        assert!(is_source_trusted_for_active_admission(
            SourceTrustType::TrustedUser,
            0.6
        ));
        assert!(!is_source_trusted_for_active_admission(
            SourceTrustType::LowConfidenceSurvey,
            0.6
        ));
        assert!(!is_source_trusted_for_active_admission(
            SourceTrustType::Unverified,
            0.6
        ));
    }

    #[test]
    fn test_source_trust_verified_flag() {
        assert!(SourceTrustType::VerifiedEvidence.is_verified());
        assert!(!SourceTrustType::TrustedUser.is_verified());
        assert!(!SourceTrustType::Unverified.is_verified());
    }
}
