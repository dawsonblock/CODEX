//! Concrete memory-provider contracts shared by in-memory and durable stores.

use crate::claim_store::ClaimStore;
use crate::durable_memory_provider::{DurableMemoryProvider, MemoryRecordQuery};
use crate::status_mapping::{durable_to_canonical, legacy_to_canonical};
use crate::{MemoryKind, MemoryStatus};

/// An inclusive time range used to filter by `created_at_unix_ms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    pub start_unix_ms: i64,
    pub end_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuery {
    pub observation: String,
    pub status: Option<MemoryStatus>,
    pub min_confidence: Option<f64>,
    pub limit: usize,
    // ── extended filter fields ────────────────────────────────────────────
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
    pub kind: Option<MemoryKind>,
    pub time_range: Option<TimeRange>,
    pub evidence_id: Option<String>,
    pub source_ref: Option<String>,
    pub max_confidence: Option<f64>,
    pub offset: usize,
    pub include_evidence_ids: bool,
    // ── admission policy flags ────────────────────────────────────────────
    pub include_stale: bool,
    pub include_disputed: bool,
    pub require_evidence: bool,
    pub exclude_denied: bool,
    pub governance_only: bool,
}

impl MemoryQuery {
    pub fn new(observation: impl Into<String>) -> Self {
        Self {
            observation: observation.into(),
            status: None,
            min_confidence: None,
            limit: 10,
            subject: None,
            predicate: None,
            object: None,
            kind: None,
            time_range: None,
            evidence_id: None,
            source_ref: None,
            max_confidence: None,
            offset: 0,
            include_evidence_ids: true,
            include_stale: false,
            include_disputed: false,
            require_evidence: false,
            exclude_denied: true,
            governance_only: false,
        }
    }

    pub fn with_status(mut self, status: MemoryStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_min_confidence(mut self, min: f64) -> Self {
        self.min_confidence = Some(min);
        self
    }

    pub fn with_max_confidence(mut self, max: f64) -> Self {
        self.max_confidence = Some(max);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    pub fn with_object(mut self, object: impl Into<String>) -> Self {
        self.object = Some(object.into());
        self
    }

    pub fn with_kind(mut self, kind: MemoryKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_time_range(mut self, range: TimeRange) -> Self {
        self.time_range = Some(range);
        self
    }

    pub fn with_evidence_id(mut self, evidence_id: impl Into<String>) -> Self {
        self.evidence_id = Some(evidence_id.into());
        self
    }

    pub fn with_source_ref(mut self, source_ref: impl Into<String>) -> Self {
        self.source_ref = Some(source_ref.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemoryHit {
    pub claim_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub status: MemoryStatus,
    pub confidence: f64,
    pub evidence_ids: Vec<String>,
    // ── extended provenance and policy fields ─────────────────────────────
    pub source_ref: Option<String>,
    pub kind: Option<MemoryKind>,
    pub created_at_unix_ms: Option<i64>,
    pub retrieval_score: Option<f64>,
    pub recency_score: Option<f64>,
    pub contradiction_ids: Vec<String>,
    pub governance_reason_code: Option<String>,
    pub is_stale: Option<bool>,
    pub is_disputed: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryProviderError {
    #[error("durable provider error: {0}")]
    Durable(String),
}

pub trait MemoryProvider: Send + Sync {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError>;
    fn get_by_status(
        &self,
        status: MemoryStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError>;
}

impl MemoryProvider for ClaimStore {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let obs = query.observation.to_lowercase();
        let subject_filter = query.subject.as_deref().map(str::to_lowercase);
        let predicate_filter = query.predicate.as_deref().map(str::to_lowercase);
        let object_filter = query.object.as_deref().map(str::to_lowercase);

        let mut hits = self
            .all_claims()
            .filter(|claim| {
                let canonical = legacy_to_canonical(claim.status);
                let status_ok = query.status.is_none_or(|s| canonical == s);
                let confidence_ok = query
                    .min_confidence
                    .is_none_or(|min| claim.confidence >= min)
                    && query
                        .max_confidence
                        .is_none_or(|max| claim.confidence <= max);
                let subject_ok = subject_filter
                    .as_deref()
                    .is_none_or(|f| claim.subject.to_lowercase().contains(f));
                let predicate_ok = predicate_filter
                    .as_deref()
                    .is_none_or(|f| claim.predicate.to_lowercase().contains(f));
                let object_ok = object_filter.as_deref().is_none_or(|f| {
                    claim
                        .object
                        .as_deref()
                        .is_some_and(|o| o.to_lowercase().contains(f))
                });
                let evidence_ok = query
                    .evidence_id
                    .as_deref()
                    .is_none_or(|eid| claim.evidence_ids.iter().any(|e| e == eid));
                let text = format!(
                    "{} {} {}",
                    claim.subject,
                    claim.predicate,
                    claim.object.as_deref().unwrap_or_default()
                )
                .to_lowercase();
                let relevance_ok = obs.is_empty()
                    || text.contains(&obs)
                    || obs.contains(&claim.subject.to_lowercase());
                status_ok
                    && confidence_ok
                    && subject_ok
                    && predicate_ok
                    && object_ok
                    && evidence_ok
                    && relevance_ok
            })
            .map(|claim| MemoryHit {
                claim_id: claim.id.clone(),
                subject: claim.subject.clone(),
                predicate: claim.predicate.clone(),
                object: claim.object.clone(),
                status: legacy_to_canonical(claim.status),
                confidence: claim.confidence,
                evidence_ids: if query.include_evidence_ids {
                    claim.evidence_ids.clone()
                } else {
                    Vec::new()
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if query.offset > 0 {
            hits = hits.into_iter().skip(query.offset).collect();
        }
        hits.truncate(query.limit);
        Ok(hits)
    }

    fn get_by_status(
        &self,
        status: MemoryStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let mut hits = self
            .query_by_status(crate::status_mapping::canonical_to_legacy(status))
            .into_iter()
            .map(|claim| MemoryHit {
                claim_id: claim.id.clone(),
                subject: claim.subject.clone(),
                predicate: claim.predicate.clone(),
                object: claim.object.clone(),
                status: legacy_to_canonical(claim.status),
                confidence: claim.confidence,
                evidence_ids: claim.evidence_ids.clone(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

impl MemoryProvider for DurableMemoryProvider {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let records = self
            .query_records(&MemoryRecordQuery {
                text_filter: if query.observation.is_empty() {
                    None
                } else {
                    Some(query.observation.as_str())
                },
                subject: query.subject.as_deref(),
                predicate: query.predicate.as_deref(),
                object: query.object.as_deref(),
                kind_filter: query.kind,
                status_filter: query.status,
                min_confidence: query.min_confidence,
                max_confidence: query.max_confidence,
                start_unix_ms: query.time_range.map(|r| r.start_unix_ms),
                end_unix_ms: query.time_range.map(|r| r.end_unix_ms),
                source_ref_filter: query.source_ref.as_deref(),
                limit: query.limit,
                offset: query.offset,
            })
            .map_err(|e| MemoryProviderError::Durable(e.to_string()))?;

        let mut hits = Vec::new();
        for record in records {
            let linked_evidence_ids: Vec<String> = if let Some(ref cid) = record.claim_id {
                self.get_linked_evidence(cid)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|l| l.evidence_id)
                    .collect()
            } else {
                Vec::new()
            };
            if let Some(ref eid) = query.evidence_id {
                if !linked_evidence_ids.iter().any(|e| e == eid) {
                    continue;
                }
            }
            let evidence_ids: Vec<String> = if query.include_evidence_ids {
                linked_evidence_ids
            } else {
                Vec::new()
            };

            hits.push(MemoryHit {
                claim_id: record.claim_id.unwrap_or_else(|| record.record_id.clone()),
                subject: record.subject,
                predicate: record.predicate,
                object: record.object.filter(|object| !object.is_empty()),
                status: record.status,
                confidence: record.confidence as f64,
                evidence_ids,
                ..Default::default()
            });
        }
        Ok(hits)
    }

    fn get_by_status(
        &self,
        status: MemoryStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let durable_status = crate::status_mapping::canonical_to_durable(status);
        let rows = DurableMemoryProvider::get_by_status(self, durable_status, limit)
            .map_err(|e| MemoryProviderError::Durable(e.to_string()))?;

        let mut hits: Vec<MemoryHit> = rows
            .into_iter()
            .map(|row| MemoryHit {
                claim_id: row.claim_id,
                subject: row.claim_text,
                predicate: "matches".to_string(),
                object: None,
                status: durable_to_canonical(row.status),
                confidence: row.confidence as f64,
                evidence_ids: Vec::new(),
                ..Default::default()
            })
            .collect();

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim_store::ClaimStore;
    use crate::durable_memory_provider::{DurableMemoryProvider, MemoryRecord};
    use crate::{ClaimEvidenceLink, MemoryKind, MemoryStatus};
    use tempfile::tempdir;

    fn populated_store() -> ClaimStore {
        let mut store = ClaimStore::new();
        store
            .assert(
                "c1",
                "codex",
                "has_feature",
                Some("logging".into()),
                0.9,
                vec![ClaimEvidenceLink {
                    evidence_id: "e1".into(),
                    weight: 1.0,
                }],
            )
            .unwrap();
        store
            .assert(
                "c2",
                "codex",
                "lacks_feature",
                Some("gui".into()),
                0.7,
                vec![],
            )
            .unwrap();
        store
            .assert(
                "c3",
                "runtime",
                "has_feature",
                Some("scheduling".into()),
                0.5,
                vec![],
            )
            .unwrap();
        store
    }

    #[test]
    fn subject_filter_narrows_results() {
        let store = populated_store();
        let q = MemoryQuery::new("").with_limit(10).with_subject("codex");
        let hits = store.query(&q).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.subject == "codex"));
    }

    #[test]
    fn predicate_filter_narrows_results() {
        let store = populated_store();
        let q = MemoryQuery::new("")
            .with_limit(10)
            .with_predicate("has_feature");
        let hits = store.query(&q).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.predicate == "has_feature"));
    }

    #[test]
    fn object_filter_narrows_results() {
        let store = populated_store();
        let q = MemoryQuery::new("").with_limit(10).with_object("gui");
        let hits = store.query(&q).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].object.as_deref(), Some("gui"));
    }

    #[test]
    fn max_confidence_excludes_high_confidence() {
        let store = populated_store();
        let q = MemoryQuery::new("")
            .with_limit(10)
            .with_max_confidence(0.75);
        let hits = store.query(&q).unwrap();
        assert!(hits.iter().all(|h| h.confidence <= 0.75));
        // c1 (0.9) must not appear
        assert!(hits
            .iter()
            .all(|h| h.subject != "codex" || h.predicate != "has_feature"));
    }

    #[test]
    fn offset_paginates_results() {
        let store = populated_store();
        let all = store.query(&MemoryQuery::new("").with_limit(100)).unwrap();
        let paged = store
            .query(&MemoryQuery::new("").with_limit(100).with_offset(1))
            .unwrap();
        assert_eq!(paged.len(), all.len().saturating_sub(1));
    }

    #[test]
    fn evidence_id_filter_returns_only_linked() {
        let store = populated_store();
        let q = MemoryQuery::new("").with_limit(10).with_evidence_id("e1");
        let hits = store.query(&q).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].evidence_ids.contains(&"e1".to_string()));
    }

    #[test]
    fn include_evidence_ids_false_strips_list() {
        let store = populated_store();
        let mut q = MemoryQuery::new("").with_limit(10);
        q.include_evidence_ids = false;
        let hits = store.query(&q).unwrap();
        assert!(hits.iter().all(|h| h.evidence_ids.is_empty()));
    }

    // ── DurableMemoryProvider::query() — real-data tests ──────────────────

    fn make_record(
        id: &str,
        subject: &str,
        predicate: &str,
        object: &str,
        kind: MemoryKind,
        status: MemoryStatus,
        confidence: f32,
    ) -> MemoryRecord {
        MemoryRecord {
            record_id: id.into(),
            claim_id: None,
            subject: subject.into(),
            predicate: predicate.into(),
            object: Some(object.into()),
            kind,
            status,
            confidence,
            source_ref: None,
            metadata_json: "{}".into(),
            created_at_unix_ms: 1_000_000,
            updated_at_unix_ms: 1_000_000,
            retrieval_score: 0.0,
            recency_score: 0.0,
            contradiction_ids: "[]".into(),
            governance_reason_code: None,
            is_stale: false,
            is_disputed: false,
        }
    }

    fn durable_with_records() -> (DurableMemoryProvider, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("mp_test.sqlite");
        let p = DurableMemoryProvider::open(&db_path).unwrap();
        p.insert_record(make_record(
            "r1",
            "codex",
            "has_feature",
            "logging",
            MemoryKind::Factual,
            MemoryStatus::Active,
            0.9,
        ))
        .unwrap();
        p.insert_record(make_record(
            "r2",
            "codex",
            "lacks_feature",
            "gui",
            MemoryKind::Factual,
            MemoryStatus::Active,
            0.7,
        ))
        .unwrap();
        p.insert_record(make_record(
            "r3",
            "runtime",
            "has_feature",
            "scheduling",
            MemoryKind::Procedural,
            MemoryStatus::Active,
            0.5,
        ))
        .unwrap();
        (p, dir)
    }

    #[test]
    fn durable_query_returns_real_subject_predicate_object() {
        let (p, _dir) = durable_with_records();
        let hits = p.query(&MemoryQuery::new("").with_limit(10)).unwrap();
        assert_eq!(hits.len(), 3);
        assert!(hits
            .iter()
            .any(|h| h.subject == "codex" && h.predicate == "has_feature"));
        assert!(hits.iter().any(|h| h.object == Some("gui".into())));
    }

    #[test]
    fn durable_query_subject_filter() {
        let (p, _dir) = durable_with_records();
        let hits = p
            .query(&MemoryQuery::new("").with_limit(10).with_subject("codex"))
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.subject.contains("codex")));
    }

    #[test]
    fn durable_query_predicate_filter() {
        let (p, _dir) = durable_with_records();
        let hits = p
            .query(
                &MemoryQuery::new("")
                    .with_limit(10)
                    .with_predicate("has_feature"),
            )
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.predicate == "has_feature"));
    }

    #[test]
    fn durable_query_object_filter() {
        let (p, _dir) = durable_with_records();
        let hits = p
            .query(&MemoryQuery::new("").with_limit(10).with_object("gui"))
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].object.as_deref(), Some("gui"));
    }

    #[test]
    fn durable_query_confidence_range() {
        let (p, _dir) = durable_with_records();
        let hits = p
            .query(
                &MemoryQuery::new("")
                    .with_limit(10)
                    .with_min_confidence(0.6)
                    .with_max_confidence(0.8),
            )
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert!((hits[0].confidence - 0.7).abs() < 0.01);
    }

    #[test]
    fn durable_query_kind_filter() {
        let (p, _dir) = durable_with_records();
        let hits = p
            .query(
                &MemoryQuery::new("")
                    .with_limit(10)
                    .with_kind(MemoryKind::Procedural),
            )
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].subject, "runtime");
    }

    #[test]
    fn durable_query_empty_observation_returns_all() {
        let (p, _dir) = durable_with_records();
        let hits = p.query(&MemoryQuery::new("").with_limit(100)).unwrap();
        assert_eq!(hits.len(), 3);
    }
}
