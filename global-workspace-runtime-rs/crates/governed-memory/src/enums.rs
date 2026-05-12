//! Core enums for governed memory system.
//!
//! These enums define the classification of memory sources, statuses, retrieval patterns,
//! and admission decisions. They form the vocabulary for policy and routing decisions.

use serde::{Deserialize, Serialize};

/// Source trust classification for memory input.
///
/// Determines how much confidence to assign to memories based on their origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceTrustType {
    /// Evidence verified against the evidence vault (hash-chain verified)
    VerifiedEvidence,

    /// Input from explicitly trusted source (user, admin policy)
    TrustedUser,

    /// Low-confidence observation (survey, heuristic, unverified feedback)
    LowConfidenceSurvey,

    /// Evidence that contradicts verified claims (flagged for review)
    ContradictionSuspect,

    /// Raw input with no verification backing
    Unverified,
}

impl SourceTrustType {
    /// Default confidence score for this source type.
    /// Not a truth indicator; confidence only.
    pub fn default_confidence(&self) -> f64 {
        match self {
            SourceTrustType::VerifiedEvidence => 0.9,
            SourceTrustType::TrustedUser => 0.7,
            SourceTrustType::LowConfidenceSurvey => 0.3,
            SourceTrustType::ContradictionSuspect => 0.4,
            SourceTrustType::Unverified => 0.0,
        }
    }
}

/// Memory claim status in the store.
///
/// Tracks the lifecycle of a claim from admission through potential contradiction
/// or supersession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryStatus {
    /// Actively used in reasoning (trusted, verified, or policy-admitted)
    Active,

    /// Stored for evidence but not yet promoted to active claim
    Pending,

    /// Contradicted by other evidence; marked but not removed
    Disputed,

    /// Superseded by a newer/better claim
    Archived,

    /// Failed admission policy
    Rejected,
}

/// Reason for not matching in retrieval.
///
/// Helps understand why a memory hit is ambiguous or needs clarification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrievalHitReason {
    /// Direct match on subject/predicate
    DirectMatch,

    /// Evidence supporting the target claim
    EvidentSupport,

    /// Conflict with other claim
    Contradicting,

    /// Partial match, semantically ambiguous
    AmbiguousMatch,

    /// Archived/historical reference
    HistoricalReference,
}

/// Intent category for a retrieval query.
///
/// Routes memory lookup requests to the appropriate handler (retrieve, defer, ask, refuse).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalIntentCategory {
    /// User asking for a memory ("do I know X?")
    MemoryLookup,

    /// Factual query not backed by evidence ("what is new_thing?")
    UnsupportedFactual,

    /// High-stakes query with low confidence evidence (medical, financial)
    HighStakesLowEvidence,

    /// Query is semantically ambiguous or multi-sense
    Ambiguous,

    /// Would require external tool/provider (policy-blocked)
    ProviderGated,
}

/// Reason for admission decision.
///
/// Used to populate audit records and reason codes explaining why a memory was
/// accepted, pending, or rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionDecisionReason {
    /// Backed by verified evidence (hash-chain valid)
    VerifiedByEvidence,

    /// Trusted user input (policy allows)
    TrustedInput,

    /// Duplicate of existing claim (same fact already in store)
    DuplicateEvidence,

    /// No evidence in vault to back the claim
    InsufficientEvidence,

    /// Contradicts verified active claim
    ConflictingClaim,

    /// Custom policy rule triggered rejection
    PolicyRejection,

    /// Retraction of prior claim
    Retraction,
}

/// Conflict nature classification.
///
/// Helps distinguish between direct contradictions, boundary cases, and provisional disagreements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictNature {
    /// Direct logical negation
    DirectContradiction,

    /// Edge case or boundary condition
    BoundaryCase,

    /// Provisional disagreement pending investigation
    ProvisionalDisagreement,
}

/// Suggested resolution for a conflict.
///
/// Informs but does not dictate how contradictions should be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Need deeper investigation before resolving
    InvestigateFurther,

    /// Accept both as valid in different contexts (disputed status)
    AcceptBothDisputed,

    /// Older verified claim is more reliable
    PreferOlderVerified,

    /// Newer evidence overrides older (rare; requires high confidence)
    PreferNewer,
}
