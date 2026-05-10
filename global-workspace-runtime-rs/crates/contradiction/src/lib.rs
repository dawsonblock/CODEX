//! Contradiction engine — detect, track, and resolve contradictions between claims.
//!
//! The engine detects contradictions when two active claims make mutually
//! exclusive assertions about the same subject. It tracks resolution outcomes
//! and produces reports.
//!
//! # Honesty boundaries
//!
//! - The engine detects conflicts between structured claims, nothing more.
//! - It does not "understand" the claims or resolve ambiguity.
//! - It does not reach truth. It flags inconsistency.
//! - Resolution is rule-based, not intelligent.

use chrono::Utc;
use memory::claim_store::ClaimStore;
use memory::MemoryClaim;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// ContradictionPattern
// ═══════════════════════════════════════════════════════════════════════════════

/// How two claims contradict each other.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionPattern {
    /// Same subject, different predicate (A is X vs A is Y).
    MutualExclusion,
    /// Same subject, same predicate, different object (A has X vs A has Y).
    SubjectObjectConflict,
    /// One claim's evidence confidence is significantly higher than the other's.
    ConfidenceInversion,
    /// The evidence supporting each claim comes from conflicting sources.
    EvidenceConflict,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Contradiction
// ═══════════════════════════════════════════════════════════════════════════════

/// A detected contradiction between two claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: String,
    pub claim_a: String,
    pub claim_b: String,
    pub subject: String,
    pub pattern: ContradictionPattern,
    /// 0.0–1.0 severity
    pub severity: f64,
    pub detected_at: String,
    pub resolved: bool,
    pub resolution: Option<ContradictionResolution>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ContradictionResolution
// ═══════════════════════════════════════════════════════════════════════════════

/// How a contradiction was resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionResolution {
    pub resolution_type: ResolutionType,
    pub resolved_at: String,
    pub note: String,
}

/// Resolution strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionType {
    /// Newer evidence supersedes older.
    NewerEvidence,
    /// Higher-confidence evidence wins.
    StrongerEvidence,
    /// Human operator resolved it.
    HumanOverride,
    /// Timed out without resolution (stale).
    TimeoutRetire,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ContradictionReport
// ═══════════════════════════════════════════════════════════════════════════════

/// Summary of contradiction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionReport {
    pub total: usize,
    pub active: usize,
    pub resolved: usize,
    pub unresolved: usize,
    pub severities: Vec<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ContradictionEngine
// ═══════════════════════════════════════════════════════════════════════════════

/// The contradiction engine — detects, tracks, and resolves contradictions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContradictionEngine {
    contradictions: Vec<Contradiction>,
    next_id: u64,
}

impl ContradictionEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self {
            contradictions: Vec::new(),
            next_id: 0,
        }
    }

    /// Scan a claim store for contradictions among active claims.
    /// Returns newly detected contradiction IDs.
    pub fn detect(&mut self, store: &ClaimStore) -> Vec<String> {
        let conflicts = store.detect_conflicts();
        let mut new_ids = Vec::new();

        for (a_id, b_id) in conflicts {
            // Skip if already tracked (unresolved)
            let already_tracked = self
                .contradictions
                .iter()
                .any(|c| !c.resolved && Self::matches(c, &a_id, &b_id));

            if already_tracked {
                continue;
            }

            let claim_a = store.get(&a_id);
            let claim_b = store.get(&b_id);
            let subject = claim_a
                .map(|c| c.subject.clone())
                .unwrap_or_else(|| "unknown".into());

            let severity = self.compute_severity(claim_a, claim_b);

            let pattern = if claim_a.and_then(|c| c.object.as_deref())
                != claim_b.and_then(|c| c.object.as_deref())
                && claim_a.map(|c| &c.predicate) == claim_b.map(|c| &c.predicate)
            {
                ContradictionPattern::SubjectObjectConflict
            } else {
                ContradictionPattern::MutualExclusion
            };

            let id = self.next_id();
            let contradiction = Contradiction {
                id: id.clone(),
                claim_a: a_id,
                claim_b: b_id,
                subject,
                pattern,
                severity,
                detected_at: Utc::now().to_rfc3339(),
                resolved: false,
                resolution: None,
            };
            self.contradictions.push(contradiction);
            new_ids.push(id);
        }

        new_ids
    }

    /// Resolve a contradiction with a given strategy.
    /// Returns the superseded claim ID if resolution picks a winner.
    pub fn resolve(
        &mut self,
        contradiction_id: &str,
        resolution_type: ResolutionType,
        note: impl Into<String>,
    ) -> Result<Option<String>, ResolutionError> {
        let c = self
            .contradictions
            .iter_mut()
            .find(|c| c.id == contradiction_id)
            .ok_or(ResolutionError::NotFound(contradiction_id.into()))?;

        if c.resolved {
            return Err(ResolutionError::AlreadyResolved(contradiction_id.into()));
        }

        // Determine winner based on resolution strategy
        let winner_id = match resolution_type {
            ResolutionType::NewerEvidence => Some(c.claim_b.clone()),
            ResolutionType::StrongerEvidence => {
                // Without claim store, default to claim_b (newer).
                // Use resolve_with_confidence for evidence-weighted resolution.
                Some(c.claim_b.clone())
            }
            ResolutionType::HumanOverride => None,
            ResolutionType::TimeoutRetire => None,
        };

        c.resolved = true;
        c.resolution = Some(ContradictionResolution {
            resolution_type,
            resolved_at: Utc::now().to_rfc3339(),
            note: note.into(),
        });

        Ok(winner_id)
    }

    /// Resolve with confidence-weighted comparison using claim store.
    /// The claim with higher confidence wins.
    pub fn resolve_with_confidence(
        &mut self,
        contradiction_id: &str,
        store: &ClaimStore,
    ) -> Result<Option<String>, ResolutionError> {
        let c = self
            .contradictions
            .iter_mut()
            .find(|c| c.id == contradiction_id)
            .ok_or(ResolutionError::NotFound(contradiction_id.into()))?;

        if c.resolved {
            return Err(ResolutionError::AlreadyResolved(contradiction_id.into()));
        }

        let conf_a = store.get(&c.claim_a).map(|cl| cl.confidence).unwrap_or(0.5);
        let conf_b = store.get(&c.claim_b).map(|cl| cl.confidence).unwrap_or(0.5);

        let winner = if conf_a >= conf_b {
            Some(c.claim_a.clone())
        } else {
            Some(c.claim_b.clone())
        };

        c.resolved = true;
        c.resolution = Some(ContradictionResolution {
            resolution_type: ResolutionType::StrongerEvidence,
            resolved_at: Utc::now().to_rfc3339(),
            note: format!(
                "confidence-weighted: claim_a={:.2}, claim_b={:.2}",
                conf_a, conf_b
            ),
        });

        Ok(winner)
    }

    /// Get all active (unresolved) contradictions.
    pub fn active(&self) -> Vec<&Contradiction> {
        self.contradictions.iter().filter(|c| !c.resolved).collect()
    }

    /// Get all resolved contradictions.
    pub fn resolved(&self) -> Vec<&Contradiction> {
        self.contradictions.iter().filter(|c| c.resolved).collect()
    }

    /// Get a specific contradiction by ID.
    pub fn get(&self, id: &str) -> Option<&Contradiction> {
        self.contradictions.iter().find(|c| c.id == id)
    }

    /// Produce a report summarizing contradiction state.
    pub fn report(&self) -> ContradictionReport {
        let total = self.contradictions.len();
        let resolved_count = self.contradictions.iter().filter(|c| c.resolved).count();
        let unresolved_count = total - resolved_count;
        let severities: Vec<f64> = self
            .contradictions
            .iter()
            .filter(|c| !c.resolved)
            .map(|c| c.severity)
            .collect();

        ContradictionReport {
            total,
            active: unresolved_count,
            resolved: resolved_count,
            unresolved: unresolved_count,
            severities,
        }
    }

    /// Number of tracked contradictions.
    pub fn len(&self) -> usize {
        self.contradictions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contradictions.is_empty()
    }

    // ── private ──────────────────────────────────────────────────────

    fn next_id(&mut self) -> String {
        let id = format!("contra_{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn compute_severity(
        &self,
        claim_a: Option<&MemoryClaim>,
        claim_b: Option<&MemoryClaim>,
    ) -> f64 {
        // Higher severity when predicates differ more and evidence is strong
        let base = 0.5;
        let evidence_bonus = match (claim_a, claim_b) {
            (Some(a), Some(b)) => {
                let links = a.evidence_links.len() + b.evidence_links.len();
                if links > 0 {
                    0.3
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        let result: f64 = base + evidence_bonus;
        result.clamp(0.0, 1.0)
    }

    fn matches(c: &Contradiction, a_id: &str, b_id: &str) -> bool {
        (c.claim_a == a_id && c.claim_b == b_id) || (c.claim_a == b_id && c.claim_b == a_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ResolutionError
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResolutionError {
    #[error("contradiction not found: {0}")]
    NotFound(String),
    #[error("contradiction already resolved: {0}")]
    AlreadyResolved(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use memory::claim_store::ClaimStore;

    fn populate_store(store: &mut ClaimStore) {
        // Two contradictory active claims
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is red", None, 0.5, vec![])
            .unwrap();
        // One compatible active claim
        store
            .assert("c3", "ocean", "is deep", None, 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();
        store.validate("c3").unwrap();
    }

    #[test]
    fn detect_finds_contradictions_in_store() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();

        let new_ids = engine.detect(&store);
        assert_eq!(new_ids.len(), 1);
        assert_eq!(engine.len(), 1);
        assert_eq!(engine.report().active, 1);
    }

    #[test]
    fn no_false_positives_on_compatible_claims() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is blue", None, 0.5, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        let mut engine = ContradictionEngine::new();
        let new_ids = engine.detect(&store);
        assert!(new_ids.is_empty());
        assert!(engine.is_empty());
    }

    #[test]
    fn detect_idempotent_on_same_store() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();

        engine.detect(&store);
        engine.detect(&store);
        // Only one contradiction, not two
        assert_eq!(engine.len(), 1);
    }

    #[test]
    fn resolve_marks_contradiction_resolved() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();

        let new_ids = engine.detect(&store);
        let cid = &new_ids[0];

        let winner = engine
            .resolve(cid, ResolutionType::NewerEvidence, "newer claim wins")
            .unwrap();
        assert!(winner.is_some());

        let c = engine.get(cid).unwrap();
        assert!(c.resolved);
        assert_eq!(engine.report().active, 0);
        assert_eq!(engine.report().resolved, 1);
    }

    #[test]
    fn resolve_already_resolved_fails() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();
        let new_ids = engine.detect(&store);
        let cid = &new_ids[0];

        engine
            .resolve(cid, ResolutionType::HumanOverride, "done")
            .unwrap();
        let err = engine
            .resolve(cid, ResolutionType::HumanOverride, "again")
            .unwrap_err();
        assert!(matches!(err, ResolutionError::AlreadyResolved(..)));
    }

    #[test]
    fn resolve_nonexistent_fails() {
        let mut engine = ContradictionEngine::new();
        let err = engine
            .resolve("nope", ResolutionType::TimeoutRetire, "")
            .unwrap_err();
        assert!(matches!(err, ResolutionError::NotFound(..)));
    }

    #[test]
    fn report_summarizes_correctly() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();
        engine.detect(&store);

        let report = engine.report();
        assert_eq!(report.total, 1);
        assert_eq!(report.active, 1);
        assert_eq!(report.resolved, 0);
        assert_eq!(report.unresolved, 1);
        assert_eq!(report.severities.len(), 1);
    }

    #[test]
    fn resolve_with_confidence_picks_higher_confidence() {
        let mut store = ClaimStore::new();
        store
            .assert("c1", "sky", "is blue", None, 0.3, vec![])
            .unwrap();
        store
            .assert("c2", "sky", "is red", None, 0.9, vec![])
            .unwrap();
        store.validate("c1").unwrap();
        store.validate("c2").unwrap();

        let mut engine = ContradictionEngine::new();
        let ids = engine.detect(&store);
        let winner = engine.resolve_with_confidence(&ids[0], &store).unwrap();
        assert_eq!(winner, Some("c2".into()));
    }

    #[test]
    fn severity_is_between_zero_and_one() {
        let mut store = ClaimStore::new();
        populate_store(&mut store);
        let mut engine = ContradictionEngine::new();
        engine.detect(&store);

        let active = engine.active();
        let c = active.first().unwrap();
        assert!(c.severity >= 0.0 && c.severity <= 1.0);
    }
}
