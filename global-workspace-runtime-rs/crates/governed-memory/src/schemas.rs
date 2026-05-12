//! Core schemas for governed memory types.
//!
//! These types bridge between CODEX internal vaults (evidence, claims, contradictions)
//! and governance policies. They are ported from memvid-Human with provider/embedding
//! fields removed.

use crate::enums::*;
use crate::reason_codes::ReasonCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Provenance metadata for a memory candidate.
///
/// Explains where a memory came from and what evidence backs it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provenance_type")]
pub enum Provenance {
    /// Direct evidence from the vault (hash-verified)
    DirectEvidence {
        evidence_id: String,
        vault_entry_hash: String,
    },

    /// User-supplied input with session context
    UserSupplied {
        user_id: String,
        session_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Inferred from other claims (reasoning chain)
    ClaimInferred { from_claims: Vec<String> },

    /// Memory has contradicting evidence
    Contradicted,
}

/// Candidate memory input for admission consideration.
///
/// Represents a potential memory before policy decides whether to admit it
/// as an active claim, store it as pending evidence, or reject it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateMemory {
    /// Unique ID for this candidate
    pub id: String,

    /// Subject of the assertion
    pub subject: String,

    /// Predicate/property
    pub predicate: String,

    /// Object value (optional)
    pub object: Option<String>,

    /// ID of source (evidence_id, user_id, etc.)
    pub source_id: String,

    /// Trust classification of the source
    pub source_type: SourceTrustType,

    /// Confidence score (0.0-1.0), NOT a truth indicator
    pub confidence: f64,

    /// When this candidate was created
    pub created_at: DateTime<Utc>,

    /// Optional link to evidence vault entry
    pub evidence_id: Option<String>,

    /// Provenance metadata
    pub provenance: Provenance,
}

/// Durable memory record in the claim store.
///
/// Persistent record of an admitted claim with metadata about
/// storage, retrieval history, and lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableMemory {
    /// Unique memory ID
    pub id: String,

    /// Link to corresponding claim in ClaimStore
    pub claim_id: String,

    /// Current status
    pub status: MemoryStatus,

    /// Was this backed by a direct claim admission?
    pub backed_by_claim: bool,

    /// Evidence IDs supporting this memory
    pub evidence_ids: Vec<String>,

    /// Last retrieval timestamp
    pub last_retrieved: Option<DateTime<Utc>>,

    /// Count of how many times retrieved
    pub retrieval_count: u64,

    /// When created
    pub created_at: DateTime<Utc>,

    /// Last update
    pub updated_at: DateTime<Utc>,
}

/// PolicySet configuration.
///
/// Collection of policies that govern memory admission and retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    /// Unique ID for this policy set
    pub id: String,

    /// Name of the policy set
    pub name: String,

    /// Description of purpose
    pub description: String,

    /// Admission policy rules
    pub admission_policy: AdmissionPolicy,

    /// Retrieval policy rules
    pub retrieval_policy: RetrievalPolicy,

    /// Is this policy set active?
    pub enabled: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update
    pub updated_at: DateTime<Utc>,
}

/// Admission policy rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionPolicy {
    /// Minimum confidence threshold to admit as active claim
    pub min_confidence_for_active: f64,

    /// Should we check for duplicates?
    pub check_duplicates: bool,

    /// Should we mark contradictions as disputed?
    pub mark_contradictions_disputed: bool,

    /// Reject raw unverified input?
    pub reject_unverified_input: bool,

    /// Custom rules (string descriptions)
    pub custom_rules: Vec<String>,
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            min_confidence_for_active: 0.6,
            check_duplicates: true,
            mark_contradictions_disputed: true,
            reject_unverified_input: true,
            custom_rules: vec![],
        }
    }
}

/// Retrieval policy rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPolicy {
    /// Confidence threshold for high-stakes decisions
    pub high_stakes_threshold: f64,

    /// Route unsupported factual queries to defer?
    pub defer_unsupported_factual: bool,

    /// Route ambiguous queries to ask clarification?
    pub ask_on_ambiguous: bool,

    /// Refuse provider-gated queries?
    pub refuse_provider_gated: bool,

    /// Custom rules
    pub custom_rules: Vec<String>,
}

impl Default for RetrievalPolicy {
    fn default() -> Self {
        Self {
            high_stakes_threshold: 0.6,
            defer_unsupported_factual: true,
            ask_on_ambiguous: true,
            refuse_provider_gated: true,
            custom_rules: vec![],
        }
    }
}

/// Request to retrieve memory.
///
/// Parameterizes a memory lookup with intent, confidence requirements,
/// and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalQuery {
    /// Unique ID for this query
    pub query_id: String,

    /// Query text
    pub query_text: String,

    /// Optional user context
    pub context: Option<String>,

    /// Intent classification
    pub intent_category: RetrievalIntentCategory,

    /// Require verification backing for results?
    pub requires_verification: bool,

    /// Max candidates to return
    pub max_candidates: usize,

    /// Minimum confidence threshold
    pub confidence_threshold: f64,

    /// When created
    pub created_at: DateTime<Utc>,
}

/// Single memory retrieval hit.
///
/// Result of a memory lookup with match details and reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    /// Unique ID for this hit
    pub hit_id: String,

    /// Query that generated this hit
    pub query_id: String,

    /// Memory ID matched
    pub memory_id: String,

    /// Corresponding claim ID (if any)
    pub claim_id: Option<String>,

    /// Match score (0.0-1.0)
    pub match_score: f64,

    /// Confidence in the match
    pub confidence: f64,

    /// Why this hit is relevant/ambiguous
    pub retrieval_hit_reason: RetrievalHitReason,

    /// Explanation of the hit
    pub reason: String,

    /// When hit was created
    pub created_at: DateTime<Utc>,
}

/// Claim record wrapped with governance metadata.
///
/// Associates a CODEX MemoryClaim with audit reason codes and
/// contradiction metadata for proof and explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedClaimRecord {
    /// CODEX claim ID
    pub claim_id: String,

    /// Subject
    pub subject: String,

    /// Predicate
    pub predicate: String,

    /// Object value
    pub object: Option<String>,

    /// Claim status
    pub status: String,

    /// Confidence (not truth indicator)
    pub confidence: f64,

    /// Evidence IDs supporting this claim
    pub evidence_ids: Vec<String>,

    /// Reason codes from admission/retrieval/conflict decisions
    pub audit_reasons: Vec<ReasonCode>,

    /// Contradiction metadata if disputed
    pub contradictions: Vec<GovernedConflictMetadata>,

    /// When created
    pub created_at: DateTime<Utc>,

    /// Last access time
    pub last_accessed: Option<DateTime<Utc>>,

    /// Number of times retrieved
    pub retrieval_count: u64,
}

/// Conflict metadata for contradicted claims.
///
/// Records what evidence conflicts with what, nature of the conflict,
/// and suggested resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedConflictMetadata {
    /// Unique conflict ID
    pub conflict_id: String,

    /// First claim in the conflict
    pub claim_a_id: String,

    /// Second claim in the conflict
    pub claim_b_id: String,

    /// Evidence IDs supporting claim A
    pub evidence_a_ids: Vec<String>,

    /// Evidence IDs supporting claim B
    pub evidence_b_ids: Vec<String>,

    /// Nature of the conflict
    pub nature: ConflictNature,

    /// Suggested resolution
    pub suggested_resolution: ConflictResolution,

    /// When conflict was detected
    pub detected_at: DateTime<Utc>,
}

/// Governance audit record.
///
/// Records admission, retrieval, and conflict decisions for proof, compliance,
/// and reasoning transparency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedMemoryAuditRecord {
    /// Unique audit ID
    pub audit_id: String,

    /// Runtime cycle this audit covers
    pub cycle_id: u64,

    /// Timestamp of this audit
    pub timestamp: DateTime<Utc>,

    /// Queried facts (what was asked)
    pub queried_facts: Vec<String>,

    /// Admitted facts (what was accepted)
    pub admitted_facts: Vec<String>,

    /// Reason codes for decisions
    pub reason_codes: Vec<ReasonCode>,

    /// Contradictions detected
    pub contradictions_detected: Vec<GovernedConflictMetadata>,

    /// Actions selected (from runtime-core)
    pub actions_selected: Vec<String>,
}
