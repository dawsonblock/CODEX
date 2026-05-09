//! Evidence vault — immutable, append-only, hash-chained evidence store.
//!
//! Every evidence entry carries a source identifier, timestamp, confidence
//! score, and content hash. The vault is append-only — evidence is never
//! mutated after writing. A hash chain links entries: each entry's hash
//! includes the previous entry's hash, forming a tamper-evident chain.
//!
//! # Honesty boundaries
//!
//! - Evidence is **raw observation**, not finished truth.
//! - The vault does **not** understand its contents.
//! - A high confidence score means the **source** is trusted, not that the
//!   evidence is true.
//! - Do not claim the vault proves anything. It stores, it does not reason.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceSource
// ═══════════════════════════════════════════════════════════════════════════════

/// The origin of a piece of evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// Direct runtime observation.
    Observation,
    /// Retrieved from memory/archive.
    MemoryRetrieval,
    /// Output from a permitted tool.
    ToolOutput,
    /// Explicitly labeled by a human operator.
    HumanLabel,
    /// Internal diagnostic or self-report.
    InternalDiagnostic,
    /// Unknown or unspecified source.
    Unknown,
}

impl EvidenceSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceSource::Observation => "observation",
            EvidenceSource::MemoryRetrieval => "memory_retrieval",
            EvidenceSource::ToolOutput => "tool_output",
            EvidenceSource::HumanLabel => "human_label",
            EvidenceSource::InternalDiagnostic => "internal_diagnostic",
            EvidenceSource::Unknown => "unknown",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceEntry
// ═══════════════════════════════════════════════════════════════════════════════

/// One immutable piece of evidence in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    /// Unique entry identifier.
    pub id: String,
    /// Where the evidence came from.
    pub source: EvidenceSource,
    /// ISO 8601 timestamp.
    pub timestamp: DateTime<Utc>,
    /// The evidence payload (free-form JSON).
    pub content: serde_json::Value,
    /// Source confidence (0.0–1.0). Does NOT mean the evidence is true.
    pub confidence: f64,
    /// SHA-like hash of this entry's fields (excluding `prev_hash`).
    pub content_hash: String,
    /// Hash of the previous entry in the chain, or "0" for the genesis entry.
    pub prev_hash: String,
}

impl EvidenceEntry {
    /// Create a new entry. Content hash is computed automatically.
    /// `prev_hash` must be the content_hash of the previous entry, or "0" for genesis.
    pub fn new(
        id: impl Into<String>,
        source: EvidenceSource,
        content: serde_json::Value,
        confidence: f64,
        prev_hash: impl Into<String>,
    ) -> Self {
        let mut entry = Self {
            id: id.into(),
            source,
            timestamp: Utc::now(),
            content,
            confidence: confidence.clamp(0.0, 1.0),
            content_hash: String::new(),
            prev_hash: prev_hash.into(),
        };
        entry.content_hash = entry.compute_content_hash();
        entry
    }

    /// Recompute the content hash from fields (excluding prev_hash itself).
    fn compute_content_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.source.hash(&mut hasher);
        // Hash via serialized form for stability
        serde_json::to_string(&self.content)
            .unwrap_or_default()
            .hash(&mut hasher);
        self.confidence.to_bits().hash(&mut hasher);
        self.prev_hash.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Verify that this entry's content_hash matches a recomputed hash.
    pub fn verify_integrity(&self) -> bool {
        self.content_hash == self.compute_content_hash()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceQuery
// ═══════════════════════════════════════════════════════════════════════════════

/// Query parameters for searching the evidence vault.
#[derive(Debug, Clone)]
pub struct EvidenceQuery {
    /// Filter by source (None = all sources).
    pub source: Option<EvidenceSource>,
    /// Minimum confidence (inclusive).
    pub min_confidence: Option<f64>,
    /// Maximum confidence (inclusive).
    pub max_confidence: Option<f64>,
    /// Search content for this keyword (case-insensitive substring).
    pub keyword: Option<String>,
    /// Maximum number of results.
    pub limit: usize,
}

impl Default for EvidenceQuery {
    fn default() -> Self {
        Self {
            source: None,
            min_confidence: None,
            max_confidence: None,
            keyword: None,
            limit: 100,
        }
    }
}

impl EvidenceQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source(mut self, source: EvidenceSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn min_confidence(mut self, c: f64) -> Self {
        self.min_confidence = Some(c);
        self
    }

    pub fn max_confidence(mut self, c: f64) -> Self {
        self.max_confidence = Some(c);
        self
    }

    pub fn keyword(mut self, kw: impl Into<String>) -> Self {
        self.keyword = Some(kw.into());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }

    fn matches(&self, entry: &EvidenceEntry) -> bool {
        // Source filter
        if let Some(ref src) = self.source {
            if &entry.source != src {
                return false;
            }
        }
        // Confidence range
        if let Some(min) = self.min_confidence {
            if entry.confidence < min {
                return false;
            }
        }
        if let Some(max) = self.max_confidence {
            if entry.confidence > max {
                return false;
            }
        }
        // Keyword search
        if let Some(ref kw) = self.keyword {
            let hay = serde_json::to_string(&entry.content)
                .unwrap_or_default()
                .to_lowercase();
            if !hay.contains(&kw.to_lowercase()) {
                return false;
            }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceIntegrityReport
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of a full vault hash-chain integrity check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIntegrityReport {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub tampered_entries: usize,
    pub chain_broken_at: Option<usize>,
    pub all_valid: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceVault
// ═══════════════════════════════════════════════════════════════════════════════

/// The evidence vault — an append-only, hash-chained evidence store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceVault {
    entries: Vec<EvidenceEntry>,
}

impl EvidenceVault {
    /// Create an empty vault.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the vault is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append an evidence entry. Returns its index.
    ///
    /// The hash chain is maintained automatically: each entry's `prev_hash`
    /// is set to the previous entry's `content_hash`, or "0" for the first.
    pub fn append(
        &mut self,
        id: impl Into<String>,
        source: EvidenceSource,
        content: serde_json::Value,
        confidence: f64,
    ) -> usize {
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.content_hash.clone())
            .unwrap_or_else(|| "0".into());
        let entry = EvidenceEntry::new(id, source, content, confidence, prev_hash);
        let idx = self.entries.len();
        self.entries.push(entry);
        idx
    }

    /// Get an entry by index.
    pub fn get(&self, index: usize) -> Option<&EvidenceEntry> {
        self.entries.get(index)
    }

    /// Query entries matching the given parameters.
    pub fn query(&self, q: &EvidenceQuery) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| q.matches(e))
            .take(q.limit)
            .collect()
    }

    /// Verify the full hash chain. Returns an integrity report.
    ///
    /// Each entry's content_hash must match its recomputed hash, and
    /// each entry's prev_hash must match the previous entry's content_hash.
    pub fn verify_integrity(&self) -> EvidenceIntegrityReport {
        let total = self.entries.len();
        let mut valid = 0;
        let mut tampered = 0;
        let mut chain_broken_at: Option<usize> = None;

        for (i, entry) in self.entries.iter().enumerate() {
            // Check content integrity
            if !entry.verify_integrity() {
                tampered += 1;
                if chain_broken_at.is_none() {
                    chain_broken_at = Some(i);
                }
                continue;
            }
            // Check chain link (skip genesis)
            if i > 0 {
                let prev = &self.entries[i - 1];
                if entry.prev_hash != prev.content_hash {
                    tampered += 1;
                    if chain_broken_at.is_none() {
                        chain_broken_at = Some(i);
                    }
                    continue;
                }
            }
            valid += 1;
        }

        EvidenceIntegrityReport {
            total_entries: total,
            valid_entries: valid,
            tampered_entries: tampered,
            chain_broken_at,
            all_valid: tampered == 0 && valid == total,
        }
    }

    /// Create a vault from a list of entries (for replay/reconstruction).
    /// Does NOT verify the chain — call `verify_integrity()` separately.
    pub fn from_entries(entries: Vec<EvidenceEntry>) -> Self {
        Self { entries }
    }

    /// Iterator over all entries.
    pub fn iter(&self) -> impl Iterator<Item = &EvidenceEntry> {
        self.entries.iter()
    }
}

impl Default for EvidenceVault {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Events (re-exported for convenience)
// ═══════════════════════════════════════════════════════════════════════════════

/// Event payload for evidence storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStoredEvent {
    pub entry_id: String,
    pub source: String,
    pub confidence: f64,
    pub content_hash: String,
}

/// Event payload for integrity check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIntegrityCheckedEvent {
    pub total: usize,
    pub valid: usize,
    pub tampered: usize,
    pub all_valid: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── append & query ──────────────────────────────────────────────

    #[test]
    fn append_and_query_by_source() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"text": "hello"}),
            0.9,
        );
        vault.append(
            "e2",
            EvidenceSource::HumanLabel,
            serde_json::json!({"label": "spam"}),
            0.95,
        );

        let obs = vault.query(&EvidenceQuery::new().source(EvidenceSource::Observation));
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].id, "e1");

        let human = vault.query(&EvidenceQuery::new().source(EvidenceSource::HumanLabel));
        assert_eq!(human.len(), 1);
        assert_eq!(human[0].id, "e2");
    }

    #[test]
    fn query_by_confidence_range() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "low",
            EvidenceSource::Observation,
            serde_json::json!({}),
            0.2,
        );
        vault.append(
            "mid",
            EvidenceSource::Observation,
            serde_json::json!({}),
            0.5,
        );
        vault.append(
            "high",
            EvidenceSource::Observation,
            serde_json::json!({}),
            0.9,
        );

        let high = vault.query(&EvidenceQuery::new().min_confidence(0.7));
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].id, "high");

        let all = vault.query(&EvidenceQuery::new().min_confidence(0.0));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn query_by_keyword() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"msg": "resource low"}),
            0.8,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({"msg": "threat high"}),
            0.8,
        );

        let results = vault.query(&EvidenceQuery::new().keyword("threat"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "e2");
    }

    #[test]
    fn empty_vault_returns_empty_queries() {
        let vault = EvidenceVault::new();
        let results = vault.query(&EvidenceQuery::new());
        assert!(results.is_empty());
    }

    // ── hash-chain integrity ────────────────────────────────────────

    #[test]
    fn hash_chain_is_valid_after_append() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"n": 1}),
            0.9,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({"n": 2}),
            0.9,
        );
        vault.append(
            "e3",
            EvidenceSource::Observation,
            serde_json::json!({"n": 3}),
            0.9,
        );

        let report = vault.verify_integrity();
        assert!(report.all_valid);
        assert_eq!(report.tampered_entries, 0);
        assert_eq!(report.total_entries, 3);
        assert_eq!(report.valid_entries, 3);
    }

    #[test]
    fn tampered_entry_detected() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"n": 1}),
            0.9,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({"n": 2}),
            0.9,
        );

        // Tamper with entry 1: change its content without recomputing hash
        vault.entries[1].content = serde_json::json!({"n": 999});

        let report = vault.verify_integrity();
        assert!(!report.all_valid);
        assert!(report.tampered_entries >= 1);
        assert!(report.chain_broken_at.is_some());
    }

    #[test]
    fn chain_broken_when_prev_hash_mismatches() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"n": 1}),
            0.9,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({"n": 2}),
            0.9,
        );

        // Break the chain link
        vault.entries[1].prev_hash = "deadbeef00000000".into();

        let report = vault.verify_integrity();
        assert!(!report.all_valid);
        assert!(report.tampered_entries >= 1);
    }

    #[test]
    fn empty_vault_integrity_is_valid() {
        let vault = EvidenceVault::new();
        let report = vault.verify_integrity();
        assert!(report.all_valid);
        assert_eq!(report.total_entries, 0);
        assert_eq!(report.tampered_entries, 0);
    }

    #[test]
    fn single_entry_has_genesis_prev_hash() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "genesis",
            EvidenceSource::Observation,
            serde_json::json!({"first": true}),
            1.0,
        );

        assert_eq!(vault.entries[0].prev_hash, "0");
        assert!(vault.entries[0].verify_integrity());
    }

    #[test]
    fn confidence_is_clamped() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({}),
            1.5,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({}),
            -0.5,
        );

        assert!((vault.entries[0].confidence - 1.0).abs() < f64::EPSILON);
        assert!((vault.entries[1].confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_entries_reconstructs_vault() {
        let mut vault = EvidenceVault::new();
        vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"x": 1}),
            0.8,
        );
        vault.append(
            "e2",
            EvidenceSource::Observation,
            serde_json::json!({"x": 2}),
            0.8,
        );

        let entries = vault.entries.clone();
        let rebuilt = EvidenceVault::from_entries(entries);

        assert_eq!(rebuilt.len(), 2);
        assert!(rebuilt.verify_integrity().all_valid);
    }
}
