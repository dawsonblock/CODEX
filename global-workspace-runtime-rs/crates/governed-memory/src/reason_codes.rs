//! Reason codes for auditable admission, retrieval, and conflict decisions.
//!
//! These codes provide a standardized vocabulary for logging why a specific decision
//! was made (admit, defer, refuse, etc.). They feed into audit trails and proof generation.

use serde::{Deserialize, Serialize};

/// Source of the reason code / policy category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasonCodeSource {
    PrivacyPolicy,
    SecurityPolicy,
    EvidencePolicy,
    SignalPolicy,
    UserPolicy,
    SystemPolicy,
}

/// Severity level of the reason code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasonSeverity {
    Info,
    Warning,
    Critical,
}

/// Auditable reason code for memory decisions.
///
/// Each admission, retrieval routing, or conflict decision logs at least one reason code
/// explaining the choice to subsequent audit or proof systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonCode {
    /// Canonical code identifier (e.g., "ADMISSION_VERIFIED_EVIDENCE")
    pub code: String,

    /// Policy category that triggered this code
    pub source: ReasonCodeSource,

    /// Severity of this decision
    pub severity: ReasonSeverity,

    /// Human-readable explanation
    pub message: String,
}

impl ReasonCode {
    /// Create a new reason code.
    pub fn new(
        code: &str,
        source: ReasonCodeSource,
        severity: ReasonSeverity,
        message: &str,
    ) -> Self {
        Self {
            code: code.to_string(),
            source,
            severity,
            message: message.to_string(),
        }
    }

    // Admission reason codes

    pub fn admission_verified_evidence() -> Self {
        Self::new(
            "ADMISSION_VERIFIED_EVIDENCE",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Info,
            "Input verified against evidence vault; admitted as active claim",
        )
    }

    pub fn admission_trusted_input() -> Self {
        Self::new(
            "ADMISSION_TRUSTED_INPUT",
            ReasonCodeSource::UserPolicy,
            ReasonSeverity::Info,
            "Input from trusted user/source; admitted as active claim",
        )
    }

    pub fn admission_duplicate_evidence() -> Self {
        Self::new(
            "ADMISSION_DUPLICATE_EVIDENCE",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Info,
            "Duplicate of existing claim; linked to existing claim_id",
        )
    }

    pub fn admission_insufficient_evidence() -> Self {
        Self::new(
            "ADMISSION_INSUFFICIENT_EVIDENCE",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Warning,
            "No backing evidence in vault; stored as pending only",
        )
    }

    pub fn admission_conflicting_claim() -> Self {
        Self::new(
            "ADMISSION_CONFLICTING_CLAIM",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Warning,
            "Contradicts verified active claim; marked as disputed",
        )
    }

    pub fn admission_policy_rejection() -> Self {
        Self::new(
            "ADMISSION_POLICY_REJECTION",
            ReasonCodeSource::SecurityPolicy,
            ReasonSeverity::Warning,
            "Custom policy rule rejection",
        )
    }

    pub fn admission_unsupported_raw_text() -> Self {
        Self::new(
            "ADMISSION_UNSUPPORTED_RAW_TEXT",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Info,
            "Unsupported raw text input; no evidence/claim created",
        )
    }

    pub fn admission_low_source_trust() -> Self {
        Self::new(
            "ADMISSION_LOW_SOURCE_TRUST",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Info,
            "Low trust source; stored as pending evidence, not active claim",
        )
    }

    // Retrieval routing reason codes

    pub fn retrieval_memory_lookup() -> Self {
        Self::new(
            "RETRIEVAL_MEMORY_LOOKUP",
            ReasonCodeSource::SignalPolicy,
            ReasonSeverity::Info,
            "User asking for a memory; route to retrieve_memory action",
        )
    }

    pub fn retrieval_unsupported_factual() -> Self {
        Self::new(
            "RETRIEVAL_UNSUPPORTED_FACTUAL",
            ReasonCodeSource::SignalPolicy,
            ReasonSeverity::Info,
            "Factual query not backed by evidence; defer_insufficient_evidence",
        )
    }

    pub fn retrieval_high_stakes_low_evidence() -> Self {
        Self::new(
            "RETRIEVAL_HIGH_STAKES_LOW_EVIDENCE",
            ReasonCodeSource::SignalPolicy,
            ReasonSeverity::Warning,
            "High-stakes query with low confidence; defer for verification",
        )
    }

    pub fn retrieval_ambiguous_match() -> Self {
        Self::new(
            "RETRIEVAL_AMBIGUOUS_MATCH",
            ReasonCodeSource::SignalPolicy,
            ReasonSeverity::Info,
            "Query is ambiguous or multi-sense; ask_clarification",
        )
    }

    pub fn retrieval_provider_gated() -> Self {
        Self::new(
            "RETRIEVAL_PROVIDER_GATED",
            ReasonCodeSource::SecurityPolicy,
            ReasonSeverity::Warning,
            "Query would require external tool/provider; refuse_unsafe (tools disabled)",
        )
    }

    // Conflict reason codes

    pub fn conflict_direct_contradiction() -> Self {
        Self::new(
            "CONFLICT_DIRECT_CONTRADICTION",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Warning,
            "Direct logical negation detected between claims",
        )
    }

    pub fn conflict_boundary_case() -> Self {
        Self::new(
            "CONFLICT_BOUNDARY_CASE",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Info,
            "Edge case or boundary condition; both claims may be valid in context",
        )
    }

    pub fn conflict_provisional_disagreement() -> Self {
        Self::new(
            "CONFLICT_PROVISIONAL_DISAGREEMENT",
            ReasonCodeSource::EvidencePolicy,
            ReasonSeverity::Warning,
            "Provisionally disagreeing claims; marked as disputed pending investigation",
        )
    }
}
