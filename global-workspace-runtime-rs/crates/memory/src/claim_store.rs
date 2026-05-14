//! Claim store — structured claim memory with lifecycle tracking.
//!
//! Claims move through: Unverified → Active → Contradicted → Superseded.
//! Claims link to evidence entries for auditability.
//!
//! # Honesty boundaries
//!
//! - Claims are structured assertions, not verified facts.
//! - The store does not believe its claims.
//! - The store does not have knowledge. It stores claims, not knowledge.
//! - A high confidence score means the evidence source is trusted,
//!   not that the claim is true.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use evidence::EvidenceEntry;

// Re-export types needed by the store
pub use crate::ClaimAuditEvent;
pub use crate::ClaimEvidenceLink;
pub use crate::ClaimLifecycleEvent;
pub use crate::ClaimStatus;
pub use crate::MemoryClaim;

// ═══════════════════════════════════════════════════════════════════════════════
// ClaimError
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors from claim store operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClaimError {
    /// A claim with this ID already exists.
    #[error("duplicate claim ID: {0}")]
    DuplicateClaimId(String),
    /// The referenced claim was not found.
    #[error("claim not found: {0}")]
    NotFound(String),
    /// The claim is not in the expected status for this operation.
    #[error("invalid status transition for claim {0}: expected {1:?}, got {2:?}")]
    InvalidStatus(String, ClaimStatus, ClaimStatus),
    /// Cannot supersede a non-Active claim.
    #[error("cannot supersede claim {0}: status is {1:?}")]
    CannotSupersede(String, ClaimStatus),
    /// Self-contradiction (claim A and A must differ).
    #[error("cannot contradict claim with itself: {0}")]
    SelfContradiction(String),
    /// Storage or serialization error.
    #[error("storage error: {0}")]
    Storage(String),
    /// Evidence payload cannot be converted into a bounded structured claim.
    #[error("unsupported structured evidence payload")]
    UnsupportedStructuredEvidence,
}

/// Compact claim reference returned by observation retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimRef {
    pub claim_id: String,
    pub evidence_id: Option<String>,
    pub status: ClaimStatus,
}

/// Retrieval result consumed by runtime scoring logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRetrievalResult {
    pub matched_claims: Vec<ClaimRef>,
    pub disputed_claims: Vec<ClaimRef>,
    pub missing_evidence: bool,
    pub confidence_summary: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ClaimStatusCounts
// ═══════════════════════════════════════════════════════════════════════════════

/// Count of claims by status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaimStatusCounts {
    pub total: usize,
    pub active: usize,
    pub contradicted: usize,
    pub superseded: usize,
    pub unverified: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ClaimStore
// ═══════════════════════════════════════════════════════════════════════════════

/// A store of claims with lifecycle management.
///
/// Claims flow through a bounded lifecycle:
/// 1. `assert()` creates an Unverified claim
/// 2. `validate()` moves Unverified → Active
/// 3. `contradict()` moves two Active claims to Contradicted
/// 4. `supersede()` moves Active → Superseded, creates new Active
///
/// All state transitions are recorded as events for replay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaimStore {
    claims: HashMap<String, MemoryClaim>,
}

impl ClaimStore {
    fn now_rfc3339() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn append_event(
        claim: &mut MemoryClaim,
        event: ClaimLifecycleEvent,
        reason: impl Into<String>,
    ) {
        let ts = Self::now_rfc3339();
        claim.updated_at = Some(ts.clone());
        claim.audit_trail.push(ClaimAuditEvent {
            timestamp: ts,
            event,
            reason: reason.into(),
        });
    }

    /// Create an empty claim store.
    pub fn new() -> Self {
        Self {
            claims: HashMap::new(),
        }
    }

    /// Number of claims in the store.
    pub fn len(&self) -> usize {
        self.claims.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    /// Get a claim by ID.
    pub fn get(&self, id: &str) -> Option<&MemoryClaim> {
        self.claims.get(id)
    }

    /// Assert a new claim. It starts as Unverified.
    ///
    /// # Errors
    /// Returns `DuplicateClaimId` if the ID already exists.
    pub fn assert(
        &mut self,
        id: impl Into<String>,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: Option<String>,
        confidence: f64,
        evidence_links: Vec<ClaimEvidenceLink>,
    ) -> Result<&MemoryClaim, ClaimError> {
        let id_str: String = id.into();
        if self.claims.contains_key(&id_str) {
            return Err(ClaimError::DuplicateClaimId(id_str));
        }

        let claim = MemoryClaim {
            id: id_str,
            subject: subject.into(),
            predicate: predicate.into(),
            object,
            status: ClaimStatus::Unverified,
            confidence: confidence.clamp(0.0, 1.0),
            evidence_ids: evidence_links
                .iter()
                .map(|l| l.evidence_id.clone())
                .collect(),
            evidence_hashes: Vec::new(),
            source_label: "claim_store_assert".into(),
            evidence_links,
            created_at: Self::now_rfc3339(),
            updated_at: None,
            audit_trail: vec![ClaimAuditEvent {
                timestamp: Self::now_rfc3339(),
                event: ClaimLifecycleEvent::Created,
                reason: "claim asserted into store".into(),
            }],
            superseded_by: None,
        };

        let claim_id = claim.id.clone();
        self.claims.insert(claim_id.clone(), claim);
        Ok(self.claims.get(&claim_id).unwrap())
    }

    /// Create or reuse a bounded claim from structured evidence.
    ///
    /// Supported payload shape:
    /// `{ "subject": "...", "predicate": "...", "object": "..." }`
    /// where object is optional. Unsupported free-form extraction is rejected.
    pub fn assert_from_evidence(
        &mut self,
        evidence: &EvidenceEntry,
    ) -> Result<&MemoryClaim, ClaimError> {
        if let Some(existing_id) = self
            .claims
            .values()
            .find(|c| c.evidence_ids.iter().any(|id| id == &evidence.id))
            .map(|c| c.id.clone())
        {
            return Ok(self.claims.get(&existing_id).unwrap());
        }

        let Some(subject) = evidence
            .content
            .get("subject")
            .and_then(serde_json::Value::as_str)
        else {
            return Err(ClaimError::UnsupportedStructuredEvidence);
        };
        let Some(predicate) = evidence
            .content
            .get("predicate")
            .and_then(serde_json::Value::as_str)
        else {
            return Err(ClaimError::UnsupportedStructuredEvidence);
        };
        let object = evidence
            .content
            .get("object")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string);

        let claim_id = format!("claim_from_{}", evidence.id);
        let evidence_link = ClaimEvidenceLink {
            evidence_id: evidence.id.clone(),
            weight: evidence.confidence.clamp(0.0, 1.0),
        };

        let _ = self.assert(
            &claim_id,
            subject,
            predicate,
            object,
            evidence.confidence,
            vec![evidence_link],
        )?;

        if let Some(created) = self.claims.get_mut(&claim_id) {
            created.evidence_ids = vec![evidence.id.clone()];
            created.evidence_hashes = vec![evidence.content_hash.clone()];
            created.source_label = evidence.source.as_str().to_string();
            created.created_at = evidence.timestamp.to_rfc3339();
            Self::append_event(
                created,
                ClaimLifecycleEvent::Created,
                format!("created from structured evidence {}", evidence.id),
            );
        }

        Ok(self.claims.get(&claim_id).unwrap())
    }

    /// Validate an Unverified claim, moving it to Active.
    /// Returns the updated claim.
    ///
    /// # Errors
    /// Returns `NotFound` or `InvalidStatus` if the claim is not Unverified.
    pub fn validate(&mut self, id: &str) -> Result<&MemoryClaim, ClaimError> {
        let claim = self
            .claims
            .get_mut(id)
            .ok_or_else(|| ClaimError::NotFound(id.into()))?;

        if claim.status != ClaimStatus::Unverified {
            return Err(ClaimError::InvalidStatus(
                id.into(),
                ClaimStatus::Unverified,
                claim.status,
            ));
        }

        claim.status = ClaimStatus::Active;
        Self::append_event(
            claim,
            ClaimLifecycleEvent::Validated,
            "claim validated to active",
        );
        Ok(claim)
    }

    /// Contradict two Active claims, moving both to Contradicted.
    ///
    /// # Errors
    /// Returns `NotFound` or `SelfContradiction` if the same claim ID is given twice.
    pub fn contradict(&mut self, claim_a: &str, claim_b: &str) -> Result<(), ClaimError> {
        if claim_a == claim_b {
            return Err(ClaimError::SelfContradiction(claim_a.into()));
        }

        // Verify both exist and are Active
        {
            let a = self
                .claims
                .get(claim_a)
                .ok_or_else(|| ClaimError::NotFound(claim_a.into()))?;
            if a.status != ClaimStatus::Active {
                return Err(ClaimError::InvalidStatus(
                    claim_a.into(),
                    ClaimStatus::Active,
                    a.status,
                ));
            }
        }
        {
            let b = self
                .claims
                .get(claim_b)
                .ok_or_else(|| ClaimError::NotFound(claim_b.into()))?;
            if b.status != ClaimStatus::Active {
                return Err(ClaimError::InvalidStatus(
                    claim_b.into(),
                    ClaimStatus::Active,
                    b.status,
                ));
            }
        }

        // Apply transition
        {
            let a = self.claims.get_mut(claim_a).unwrap();
            a.status = ClaimStatus::Contradicted;
            Self::append_event(
                a,
                ClaimLifecycleEvent::ContradictedBy {
                    other_claim_id: claim_b.to_string(),
                },
                "claim contradicted by another active claim",
            );
        }
        {
            let b = self.claims.get_mut(claim_b).unwrap();
            b.status = ClaimStatus::Contradicted;
            Self::append_event(
                b,
                ClaimLifecycleEvent::ContradictedBy {
                    other_claim_id: claim_a.to_string(),
                },
                "claim contradicted by another active claim",
            );
        }
        Ok(())
    }

    /// Supersede an Active claim with a new claim.
    /// The old claim moves to Superseded. The new claim is Active.
    ///
    /// # Errors
    /// Returns `NotFound`, `CannotSupersede`, or `DuplicateClaimId`.
    #[allow(clippy::too_many_arguments)]
    pub fn supersede(
        &mut self,
        old_id: &str,
        new_id: impl Into<String>,
        new_subject: impl Into<String>,
        new_predicate: impl Into<String>,
        new_object: Option<String>,
        confidence: f64,
        evidence_links: Vec<ClaimEvidenceLink>,
    ) -> Result<&MemoryClaim, ClaimError> {
        let new_id_str: String = new_id.into();

        // Check old claim exists and is Active
        let old = self
            .claims
            .get(old_id)
            .ok_or_else(|| ClaimError::NotFound(old_id.into()))?;
        if old.status != ClaimStatus::Active {
            return Err(ClaimError::CannotSupersede(old_id.into(), old.status));
        }

        // Check new ID doesn't collide
        if self.claims.contains_key(&new_id_str) {
            return Err(ClaimError::DuplicateClaimId(new_id_str));
        }

        // Create new claim
        let new_claim = MemoryClaim {
            id: new_id_str,
            subject: new_subject.into(),
            predicate: new_predicate.into(),
            object: new_object,
            status: ClaimStatus::Active,
            confidence: confidence.clamp(0.0, 1.0),
            evidence_ids: evidence_links
                .iter()
                .map(|l| l.evidence_id.clone())
                .collect(),
            evidence_hashes: Vec::new(),
            source_label: "claim_store_supersede".into(),
            evidence_links,
            created_at: Self::now_rfc3339(),
            updated_at: None,
            audit_trail: vec![ClaimAuditEvent {
                timestamp: Self::now_rfc3339(),
                event: ClaimLifecycleEvent::Created,
                reason: "claim created as superseding replacement".into(),
            }],
            superseded_by: None,
        };

        // Transition old → Superseded, link to new
        {
            let old_mut = self.claims.get_mut(old_id).unwrap();
            old_mut.status = ClaimStatus::Superseded;
            old_mut.superseded_by = Some(new_claim.id.clone());
            Self::append_event(
                old_mut,
                ClaimLifecycleEvent::SupersededBy {
                    new_claim_id: new_claim.id.clone(),
                },
                "claim superseded by newer active claim",
            );
        }

        let new_id = new_claim.id.clone();
        self.claims.insert(new_id.clone(), new_claim);
        Ok(self.claims.get(&new_id).unwrap())
    }

    /// Retract (remove) an Active claim. It is simply removed from the store.
    /// Returns an error if the claim is not found.
    ///
    /// # Errors
    /// Returns `NotFound` if no claim with this ID exists.
    pub fn retract(&mut self, id: &str) -> Result<MemoryClaim, ClaimError> {
        let mut removed = self
            .claims
            .remove(id)
            .ok_or_else(|| ClaimError::NotFound(id.into()))?;
        Self::append_event(
            &mut removed,
            ClaimLifecycleEvent::Retracted,
            "claim retracted from store",
        );
        Ok(removed)
    }

    /// Query claims by subject (exact match).
    pub fn query_by_subject(&self, subject: &str) -> Vec<&MemoryClaim> {
        self.claims
            .values()
            .filter(|c| c.subject == subject)
            .collect()
    }

    /// Query claims by status.
    pub fn query_by_status(&self, status: ClaimStatus) -> Vec<&MemoryClaim> {
        self.claims
            .values()
            .filter(|c| c.status == status)
            .collect()
    }

    /// All Active claims.
    pub fn active_claims(&self) -> Vec<&MemoryClaim> {
        self.query_by_status(ClaimStatus::Active)
    }

    /// All Unverified claims.
    pub fn unverified_claims(&self) -> Vec<&MemoryClaim> {
        self.query_by_status(ClaimStatus::Unverified)
    }

    /// All claims (borrowed).
    pub fn all_claims(&self) -> impl Iterator<Item = &MemoryClaim> {
        self.claims.values()
    }

    /// Count of claims by status.
    pub fn status_counts(&self) -> ClaimStatusCounts {
        let mut counts = ClaimStatusCounts {
            total: self.claims.len(),
            ..Default::default()
        };
        for claim in self.claims.values() {
            match claim.status {
                ClaimStatus::Active => counts.active += 1,
                ClaimStatus::Contradicted => counts.contradicted += 1,
                ClaimStatus::Superseded => counts.superseded += 1,
                ClaimStatus::Unverified => counts.unverified += 1,
            }
        }
        counts
    }

    /// Save claims to a JSONL file.
    pub fn save_jsonl(&self, path: impl AsRef<std::path::Path>) -> Result<(), ClaimError> {
        use std::io::Write;
        let mut f = std::fs::File::create(path).map_err(|e| ClaimError::Storage(e.to_string()))?;
        for claim in self.claims.values() {
            let line =
                serde_json::to_string(claim).map_err(|e| ClaimError::Storage(e.to_string()))?;
            writeln!(f, "{}", line).map_err(|e| ClaimError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    /// Load claims from a JSONL file. Replaces current store contents.
    pub fn load_jsonl(path: impl AsRef<std::path::Path>) -> Result<Self, ClaimError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ClaimError::Storage(e.to_string()))?;
        let mut store = Self::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let claim: MemoryClaim =
                serde_json::from_str(line).map_err(|e| ClaimError::Storage(e.to_string()))?;
            store.claims.insert(claim.id.clone(), claim);
        }
        Ok(store)
    }

    /// Check if any two Active claims contradict each other.
    /// Detects: same subject + different predicate (mutual exclusion),
    /// and same subject + same predicate + different object (object conflict).
    /// Returns pairs of claim IDs.
    pub fn detect_conflicts(&self) -> Vec<(String, String)> {
        let active: Vec<&MemoryClaim> = self.active_claims();
        let mut conflicts = Vec::new();
        for i in 0..active.len() {
            for j in (i + 1)..active.len() {
                let same_subject = active[i].subject == active[j].subject;
                let diff_predicate = active[i].predicate != active[j].predicate;
                let diff_object = active[i].object != active[j].object;
                // Conflict: same subject, different predicate OR same subject+predicate, different object
                if same_subject && (diff_predicate || diff_object) {
                    conflicts.push((active[i].id.clone(), active[j].id.clone()));
                }
            }
        }
        conflicts
    }

    /// Retrieve relevant claims for an observation using bounded lexical matching.
    pub fn retrieve_for_observation(&self, observation: &str) -> ClaimRetrievalResult {
        let obs = observation.to_lowercase();
        let mut matched_claims = Vec::new();
        let mut disputed_claims = Vec::new();
        let mut confidence_sum = 0.0;

        for claim in self.claims.values() {
            let relevant = claim.subject.to_lowercase().contains(&obs)
                || obs.contains(&claim.subject.to_lowercase())
                || claim.predicate.to_lowercase().contains(&obs)
                || obs.contains(&claim.predicate.to_lowercase())
                || claim
                    .object
                    .as_deref()
                    .map(|o| obs.contains(&o.to_lowercase()))
                    .unwrap_or(false);

            if !relevant {
                continue;
            }

            let claim_ref = ClaimRef {
                claim_id: claim.id.clone(),
                evidence_id: claim.evidence_ids.first().cloned(),
                status: claim.status,
            };

            match claim.status {
                ClaimStatus::Active => {
                    confidence_sum += claim.confidence;
                    matched_claims.push(claim_ref);
                }
                ClaimStatus::Contradicted => disputed_claims.push(claim_ref),
                ClaimStatus::Superseded | ClaimStatus::Unverified => {}
            }
        }

        // Prefer evidence-backed active claims first.
        matched_claims.sort_by_key(|c| c.evidence_id.is_none());

        let missing_evidence = matched_claims.iter().any(|c| c.evidence_id.is_none());
        let confidence_summary = if matched_claims.is_empty() {
            0.0
        } else {
            (confidence_sum / matched_claims.len() as f64).clamp(0.0, 1.0)
        };

        ClaimRetrievalResult {
            matched_claims,
            disputed_claims,
            missing_evidence,
            confidence_summary,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use evidence::EvidenceSource;

    fn make_link(evidence_id: &str, weight: f64) -> ClaimEvidenceLink {
        ClaimEvidenceLink {
            evidence_id: evidence_id.into(),
            weight,
        }
    }

    // ── assert ───────────────────────────────────────────────────────

    #[test]
    fn assert_creates_unverified_claim() {
        let mut store = ClaimStore::new();
        let claim = store
            .assert(
                "c1",
                "sky",
                "is blue",
                None,
                0.9,
                vec![make_link("e1", 0.8)],
            )
            .unwrap();

        assert_eq!(claim.id, "c1");
        assert_eq!(claim.subject, "sky");
        assert_eq!(claim.predicate, "is blue");
        assert_eq!(claim.status, ClaimStatus::Unverified);
        assert_eq!(claim.evidence_links.len(), 1);
    }

    #[test]
    fn duplicate_claim_id_is_rejected() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is Y", None, 0.5, vec![]).unwrap();
        let err = store
            .assert("c1", "X", "is Z", None, 0.5, vec![])
            .unwrap_err();
        assert!(matches!(err, ClaimError::DuplicateClaimId(ref id) if id == "c1"));
    }

    #[test]
    fn assert_from_structured_evidence_creates_linked_claim() {
        let mut store = ClaimStore::new();
        let entry = EvidenceEntry::new(
            "ev_1",
            EvidenceSource::Observation,
            serde_json::json!({"subject":"sky","predicate":"is blue","object":"daytime"}),
            0.9,
            "0",
        );

        let claim = store.assert_from_evidence(&entry).unwrap();
        assert_eq!(claim.id, "claim_from_ev_1");
        assert_eq!(claim.subject, "sky");
        assert_eq!(claim.predicate, "is blue");
        assert_eq!(claim.object.as_deref(), Some("daytime"));
        assert_eq!(claim.evidence_ids, vec!["ev_1"]);
        assert_eq!(claim.evidence_hashes, vec![entry.content_hash.clone()]);
        assert_eq!(claim.source_label, "observation");
    }

    #[test]
    fn assert_from_evidence_deduplicates_same_evidence() {
        let mut store = ClaimStore::new();
        let entry = EvidenceEntry::new(
            "ev_2",
            EvidenceSource::Observation,
            serde_json::json!({"subject":"ocean","predicate":"is deep"}),
            0.9,
            "0",
        );

        let c1 = store.assert_from_evidence(&entry).unwrap().id.clone();
        let c2 = store.assert_from_evidence(&entry).unwrap().id.clone();
        assert_eq!(c1, c2);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn unsupported_evidence_payload_is_rejected() {
        let mut store = ClaimStore::new();
        let entry = EvidenceEntry::new(
            "ev_bad",
            EvidenceSource::Observation,
            serde_json::json!({"free_text":"just some text"}),
            0.7,
            "0",
        );
        let err = store.assert_from_evidence(&entry).unwrap_err();
        assert!(matches!(err, ClaimError::UnsupportedStructuredEvidence));
    }

    // ── validate ─────────────────────────────────────────────────────

    #[test]
    fn validate_moves_unverified_to_active() {
        let mut store = ClaimStore::new();
        store.assert("c1", "A", "is B", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();

        let claim = store.get("c1").unwrap();
        assert_eq!(claim.status, ClaimStatus::Active);
    }

    #[test]
    fn validate_requires_unverified_status() {
        let mut store = ClaimStore::new();
        store.assert("c1", "A", "is B", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();

        // Already Active — validating again should fail
        let err = store.validate("c1").unwrap_err();
        assert!(matches!(err, ClaimError::InvalidStatus(..)));
    }

    #[test]
    fn validate_nonexistent_claim_fails() {
        let mut store = ClaimStore::new();
        let err = store.validate("nonexistent").unwrap_err();
        assert!(matches!(err, ClaimError::NotFound(..)));
    }

    // ── contradict ───────────────────────────────────────────────────

    #[test]
    fn contradict_moves_both_to_contradicted() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is A", None, 0.5, vec![]).unwrap();
        store.assert("c2", "X", "is B", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        store.contradict("c1", "c2").unwrap();

        assert_eq!(store.get("c1").unwrap().status, ClaimStatus::Contradicted);
        assert_eq!(store.get("c2").unwrap().status, ClaimStatus::Contradicted);
    }

    #[test]
    fn self_contradiction_is_rejected() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is A", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();

        let err = store.contradict("c1", "c1").unwrap_err();
        assert!(matches!(err, ClaimError::SelfContradiction(..)));
    }

    #[test]
    fn contradict_requires_both_active() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is A", None, 0.5, vec![]).unwrap();
        store.assert("c2", "X", "is B", None, 0.5, vec![]).unwrap();
        // c1 validated (Active), c2 stays Unverified

        store.validate("c1").unwrap();
        let err = store.contradict("c1", "c2").unwrap_err();
        assert!(matches!(err, ClaimError::InvalidStatus(..)));
    }

    // ── supersede ────────────────────────────────────────────────────

    #[test]
    fn supersede_moves_old_to_superseded_and_creates_new_active() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "weather", "is sunny", None, 0.7, vec![])
            .unwrap();
        store.validate("c1").unwrap();

        let new = store
            .supersede(
                "c1",
                "c2",
                "weather",
                "is raining",
                None,
                0.9,
                vec![make_link("e2", 0.85)],
            )
            .unwrap();

        assert_eq!(new.id, "c2");
        assert_eq!(new.status, ClaimStatus::Active);
        assert_eq!(store.get("c1").unwrap().status, ClaimStatus::Superseded);
        assert_eq!(store.get("c1").unwrap().superseded_by, Some("c2".into()));
    }

    #[test]
    fn supersede_requires_old_to_be_active() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is Y", None, 0.5, vec![]).unwrap();
        // Not validated — still Unverified

        let err = store
            .supersede("c1", "c2", "X", "is Z", None, 0.5, vec![])
            .unwrap_err();
        assert!(matches!(err, ClaimError::CannotSupersede(..)));
    }

    #[test]
    fn supersede_rejects_duplicate_new_id() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is Y", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();
        store
            .assert("c2", "other", "exists", None, 0.5, vec![])
            .unwrap();

        let err = store
            .supersede("c1", "c2", "X", "is Z", None, 0.5, vec![])
            .unwrap_err();
        assert!(matches!(err, ClaimError::DuplicateClaimId(ref id) if id == "c2"));
    }

    // ── query ────────────────────────────────────────────────────────

    #[test]
    fn query_by_subject_returns_matching_claims() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is cloudy", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c3", "ocean", "is deep", None, 0.5, vec![])
            .unwrap();

        let sky_claims = store.query_by_subject("sky");
        assert_eq!(sky_claims.len(), 2);
    }

    #[test]
    fn query_by_status_filters_correctly() {
        let mut store = ClaimStore::new();
        store.assert("c1", "A", "is 1", None, 0.5, vec![]).unwrap();
        store.assert("c2", "B", "is 2", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();

        let active = store.query_by_status(ClaimStatus::Active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "c1");

        let unverified = store.query_by_status(ClaimStatus::Unverified);
        assert_eq!(unverified.len(), 1);
        assert_eq!(unverified[0].id, "c2");
    }

    #[test]
    fn status_counts_are_accurate() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is A", None, 0.5, vec![]).unwrap();
        store.assert("c2", "X", "is B", None, 0.5, vec![]).unwrap();
        store.assert("c3", "Y", "is C", None, 0.5, vec![]).unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();
        store.contradict("c1", "c2").unwrap();

        let counts = store.status_counts();
        assert_eq!(counts.total, 3);
        assert_eq!(counts.active, 0);
        assert_eq!(counts.contradicted, 2);
        assert_eq!(counts.unverified, 1);
        assert_eq!(counts.superseded, 0);
    }

    // ── retract ──────────────────────────────────────────────────────

    #[test]
    fn retract_removes_claim() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is Y", None, 0.5, vec![]).unwrap();

        let removed = store.retract("c1").unwrap();
        assert_eq!(removed.id, "c1");
        assert!(store.get("c1").is_none());
    }

    #[test]
    fn retract_nonexistent_fails() {
        let mut store = ClaimStore::new();
        let err = store.retract("nope").unwrap_err();
        assert!(matches!(err, ClaimError::NotFound(..)));
    }

    // ── conflict detection ───────────────────────────────────────────

    #[test]
    fn detect_conflicts_finds_same_subject_different_predicate() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is red", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c3", "ocean", "is deep", None, 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();
        store.validate("c3").unwrap();

        let conflicts = store.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        // Either ordering is fine
        assert!(
            conflicts.contains(&("c1".into(), "c2".into()))
                || conflicts.contains(&("c2".into(), "c1".into()))
        );
    }

    #[test]
    fn detect_conflicts_no_false_positive_on_compatible_claims() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        let conflicts = store.detect_conflicts();
        // Same predicate → no conflict
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn detect_object_conflicts_same_subject_predicate_different_object() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is", Some("blue".into()), 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is", Some("red".into()), 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        let conflicts = store.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn detect_object_conflicts_no_false_positive_on_compatible_objects() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is", Some("blue".into()), 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is", Some("blue".into()), 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        let conflicts = store.detect_conflicts();
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn detect_conflicts_only_checks_active_claims() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is red", None, 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        // c2 stays Unverified — not checked

        let conflicts = store.detect_conflicts();
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn retrieve_for_observation_prefers_evidence_backed_active_claims() {
        let mut store = ClaimStore::new();
        let entry = EvidenceEntry::new(
            "ev_3",
            EvidenceSource::Observation,
            serde_json::json!({"subject":"invoice","predicate":"is overdue"}),
            0.8,
            "0",
        );
        let cid = store.assert_from_evidence(&entry).unwrap().id.clone();
        store.validate(&cid).unwrap();

        let result = store.retrieve_for_observation("invoice overdue status");
        assert_eq!(result.matched_claims.len(), 1);
        assert_eq!(result.matched_claims[0].claim_id, cid);
        assert!(!result.missing_evidence);
        assert!(result.confidence_summary > 0.0);
    }

    // ── evidence link traversal ──────────────────────────────────────

    #[test]
    fn confidence_is_stored_and_clamped() {
        let mut store = ClaimStore::new();
        store.assert("c1", "A", "is B", None, 1.5, vec![]).unwrap();
        let claim = store.get("c1").unwrap();
        assert!((claim.confidence - 1.0).abs() < f64::EPSILON);

        store.assert("c2", "B", "is C", None, -0.5, vec![]).unwrap();
        let claim2 = store.get("c2").unwrap();
        assert!((claim2.confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_survives_validation() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is Y", None, 0.75, vec![]).unwrap();
        store.validate("c1").unwrap();
        assert!((store.get("c1").unwrap().confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_survives_contradiction() {
        let mut store = ClaimStore::new();
        store.assert("c1", "X", "is A", None, 0.8, vec![]).unwrap();
        store.assert("c2", "X", "is B", None, 0.6, vec![]).unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();
        store.contradict("c1", "c2").unwrap();
        assert!((store.get("c1").unwrap().confidence - 0.8).abs() < f64::EPSILON);
        assert!((store.get("c2").unwrap().confidence - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn evidence_links_are_preserved_through_lifecycle() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "X", "is Y", None, 0.7, vec![make_link("e1", 0.9)])
            .unwrap();

        // Links survive validation
        store.validate("c1").unwrap();
        assert_eq!(store.get("c1").unwrap().evidence_links[0].evidence_id, "e1");

        // Links survive supersede (old claim preserves them)
        store
            .supersede(
                "c1",
                "c2",
                "X",
                "is Z",
                None,
                0.8,
                vec![make_link("e2", 0.85)],
            )
            .unwrap();
        assert_eq!(store.get("c1").unwrap().evidence_links[0].evidence_id, "e1");
        assert_eq!(store.get("c2").unwrap().evidence_links[0].evidence_id, "e2");
    }

    #[test]
    fn audit_trail_records_validate_contradict_and_supersede() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.7, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is red", None, 0.6, vec![])
            .unwrap();

        store.validate("c1").unwrap();
        store.validate("c2").unwrap();
        store.contradict("c1", "c2").unwrap();

        let c1 = store.get("c1").unwrap();
        assert!(c1
            .audit_trail
            .iter()
            .any(|e| matches!(e.event, ClaimLifecycleEvent::Validated)));
        assert!(c1
            .audit_trail
            .iter()
            .any(|e| matches!(e.event, ClaimLifecycleEvent::ContradictedBy { .. })));

        // Create a fresh active claim and supersede it.
        store
            .assert("c3", "weather", "is sunny", None, 0.8, vec![])
            .unwrap();
        store.validate("c3").unwrap();
        store
            .supersede("c3", "c4", "weather", "is rainy", None, 0.9, vec![])
            .unwrap();

        let old = store.get("c3").unwrap();
        assert!(old
            .audit_trail
            .iter()
            .any(|e| matches!(e.event, ClaimLifecycleEvent::SupersededBy { .. })));
    }

    #[test]
    fn retract_appends_retracted_event_on_returned_claim() {
        let mut store = ClaimStore::new();
        store
            .assert("c9", "temp", "is set", None, 0.5, vec![])
            .unwrap();
        let removed = store.retract("c9").unwrap();
        assert!(removed
            .audit_trail
            .iter()
            .any(|e| matches!(e.event, ClaimLifecycleEvent::Retracted)));
    }
}
