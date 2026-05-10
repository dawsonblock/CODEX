//! Evidence vault — immutable, append-only, hash-chained evidence store.
//!
//! Every evidence entry carries a source identifier, timestamp, confidence
//! score, and content hash. The vault is append-only — evidence is never
//! mutated after writing. A hash chain links entries: each entry's hash
//! includes the previous entry's hash, forming a tamper-evident chain.
//!
//! The content hash is a SHA-256 digest over canonical evidence fields:
//! id, source, timestamp, content (serialized JSON), confidence, and
//! prev_hash. Timestamp tampering is therefore detected by integrity checks.
//!
//! # Current limitations
//!
//! - **In-memory + JSONL**: The vault supports JSONL persistence (save/load).
//! - **Scaffold**: This is a Phase-1A evidence scaffold, not a complete
//!   evidence-grounded memory system. It does not support claims, contradiction
//!   detection, or long-term archival.
//! - **Duplicate ID rejection**: Implemented. Duplicate evidence IDs are
//!   rejected with `EvidenceError::DuplicateEvidenceId`.
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
use sha2::{Digest, Sha256};

// ═══════════════════════════════════════════════════════════════════════════════
// EvidenceError
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors that can occur in evidence vault operations.
#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    /// An evidence entry with this ID already exists in the vault.
    #[error("duplicate evidence ID: {0}")]
    DuplicateEvidenceId(String),
    /// Entry content hash does not match recomputed hash.
    #[error("evidence integrity failure for entry {0}")]
    IntegrityFailure(String),
    /// Generic storage error.
    #[error("storage error: {0}")]
    StorageError(String),
    /// Query returned no results (not always an error; use for strict lookups).
    #[error("query returned no results")]
    QueryError,
    /// I/O error during persistence.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Corrupt JSONL line in loaded file.
    #[error("corrupt JSONL at line {0}: {1}")]
    CorruptJsonl(usize, String),
}

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
    /// ISO 8601 timestamp (hash-protected — tampering is detected).
    pub timestamp: DateTime<Utc>,
    /// The evidence payload (free-form JSON).
    pub content: serde_json::Value,
    /// Source confidence (0.0–1.0). Does NOT mean the evidence is true.
    pub confidence: f64,
    /// SHA-256 digest over canonical entry fields including `prev_hash`.
    /// This is the link in the hash chain.
    pub content_hash: String,
    /// Hash of the previous entry in the chain, or "0" for the genesis entry.
    pub prev_hash: String,
}

impl EvidenceEntry {
    /// Create a new entry. Content hash is computed automatically via SHA-256.
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

    /// Compute the SHA-256 content hash over canonical fields.
    ///
    /// Fields included: id, source (as_str), timestamp (RFC 3339),
    /// content (canonical JSON), confidence (as bits), prev_hash.
    fn compute_content_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // ID — canonical string
        hasher.update(b"id:");
        hasher.update(self.id.as_bytes());
        hasher.update(b"\n");

        // Source — as_str for stability
        hasher.update(b"source:");
        hasher.update(self.source.as_str().as_bytes());
        hasher.update(b"\n");

        // Timestamp — RFC 3339 for deterministic serialization
        let ts = self.timestamp.to_rfc3339();
        hasher.update(b"timestamp:");
        hasher.update(ts.as_bytes());
        hasher.update(b"\n");

        // Content — canonical JSON (sorted keys via serde_json)
        let content_json = serde_json::to_string(&self.content).unwrap_or_default();
        hasher.update(b"content:");
        hasher.update(content_json.as_bytes());
        hasher.update(b"\n");

        // Confidence — raw bits for exactness
        hasher.update(b"confidence:");
        hasher.update(self.confidence.to_bits().to_le_bytes());
        hasher.update(b"\n");

        // Previous hash — the chain link
        hasher.update(b"prev_hash:");
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(b"\n");

        format!("{:x}", hasher.finalize())
    }

    /// Verify that this entry's content_hash matches a recomputed SHA-256 hash.
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

/// The evidence vault — an append-only, SHA-256 hash-chained evidence store.
///
/// Supports JSONL persistence via `save_jsonl()` and `load_jsonl()`.
/// The vault is a scaffold for evidence-grounded memory, not a complete system.
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

    /// Append an evidence entry. Returns its index on success.
    ///
    /// The hash chain is maintained automatically: each entry's `prev_hash`
    /// is set to the previous entry's `content_hash`, or "0" for the first.
    ///
    /// # Errors
    ///
    /// Returns `EvidenceError::DuplicateEvidenceId` if an entry with the
    /// same `id` already exists.
    pub fn append(
        &mut self,
        id: impl Into<String>,
        source: EvidenceSource,
        content: serde_json::Value,
        confidence: f64,
    ) -> Result<usize, EvidenceError> {
        let id_str = id.into();

        // Reject duplicate IDs
        if self.entries.iter().any(|e| e.id == id_str) {
            return Err(EvidenceError::DuplicateEvidenceId(id_str));
        }

        let prev_hash = self
            .entries
            .last()
            .map(|e| e.content_hash.clone())
            .unwrap_or_else(|| "0".into());
        let entry = EvidenceEntry::new(id_str, source, content, confidence, prev_hash);
        let idx = self.entries.len();
        self.entries.push(entry);
        Ok(idx)
    }

    /// Get an entry by index.
    pub fn get(&self, index: usize) -> Option<&EvidenceEntry> {
        self.entries.get(index)
    }

    /// Get an entry by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&EvidenceEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Query entries matching the given parameters.
    pub fn query(&self, q: &EvidenceQuery) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| q.matches(e))
            .take(q.limit)
            .collect()
    }

    /// Verify the full SHA-256 hash chain. Returns an integrity report.
    ///
    /// Each entry's content_hash must match its recomputed hash, and
    /// each entry's prev_hash must match the previous entry's content_hash.
    /// Timestamp tampering and content tampering are both detected.
    pub fn verify_integrity(&self) -> EvidenceIntegrityReport {
        let total = self.entries.len();
        let mut valid = 0;
        let mut tampered = 0;
        let mut chain_broken_at: Option<usize> = None;

        for (i, entry) in self.entries.iter().enumerate() {
            // Check content integrity (includes timestamp, confidence, prev_hash)
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

    /// Save the vault to a JSONL file. Each line is one evidence entry.
    pub fn save_jsonl(&self, path: impl AsRef<std::path::Path>) -> Result<(), EvidenceError> {
        let mut f = std::fs::File::create(path)?;
        for entry in &self.entries {
            let line = serde_json::to_string(entry)?;
            use std::io::Write;
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }

    /// Load a vault from a JSONL file, rebuilding chain and verifying integrity.
    pub fn load_jsonl(path: impl AsRef<std::path::Path>) -> Result<Self, EvidenceError> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: EvidenceEntry = serde_json::from_str(line)
                .map_err(|e| EvidenceError::CorruptJsonl(i + 1, e.to_string()))?;
            entries.push(entry);
        }
        let vault = Self::from_entries(entries);
        let report = vault.verify_integrity();
        if !report.all_valid {
            return Err(EvidenceError::IntegrityFailure(
                "loaded vault has integrity failure".into(),
            ));
        }
        Ok(vault)
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
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"text": "hello"}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::HumanLabel,
                serde_json::json!({"label": "spam"}),
                0.95,
            )
            .unwrap();

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
        vault
            .append(
                "low",
                EvidenceSource::Observation,
                serde_json::json!({}),
                0.2,
            )
            .unwrap();
        vault
            .append(
                "mid",
                EvidenceSource::Observation,
                serde_json::json!({}),
                0.5,
            )
            .unwrap();
        vault
            .append(
                "high",
                EvidenceSource::Observation,
                serde_json::json!({}),
                0.9,
            )
            .unwrap();

        let high = vault.query(&EvidenceQuery::new().min_confidence(0.7));
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].id, "high");

        let all = vault.query(&EvidenceQuery::new().min_confidence(0.0));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn query_by_keyword() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"msg": "resource low"}),
                0.8,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"msg": "threat high"}),
                0.8,
            )
            .unwrap();

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

    // ── duplicate ID rejection ──────────────────────────────────────

    #[test]
    fn duplicate_id_is_rejected() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "dup",
                EvidenceSource::Observation,
                serde_json::json!({}),
                0.5,
            )
            .unwrap();

        let err = vault
            .append(
                "dup",
                EvidenceSource::Observation,
                serde_json::json!({}),
                0.5,
            )
            .unwrap_err();
        assert!(matches!(err, EvidenceError::DuplicateEvidenceId(ref id) if id == "dup"));
    }

    #[test]
    fn original_entry_unchanged_after_duplicate_rejection() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"val": 1}),
                0.8,
            )
            .unwrap();

        let _ = vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"val": 999}),
            0.5,
        );

        assert_eq!(vault.len(), 1);
        let entry = vault.get(0).unwrap();
        assert_eq!(entry.content, serde_json::json!({"val": 1}));
        assert!((entry.confidence - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn duplicate_rejection_does_not_alter_hash_chain() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();

        let report_before = vault.verify_integrity();
        assert!(report_before.all_valid);

        let _ = vault.append(
            "e1",
            EvidenceSource::Observation,
            serde_json::json!({"n": 999}),
            0.5,
        );

        let report_after = vault.verify_integrity();
        assert!(report_after.all_valid);
        assert_eq!(vault.len(), 1);
    }

    #[test]
    fn integrity_check_still_passes_after_rejected_duplicate() {
        let mut vault = EvidenceVault::new();
        vault
            .append("a", EvidenceSource::Observation, serde_json::json!({}), 0.9)
            .unwrap();
        vault
            .append("b", EvidenceSource::Observation, serde_json::json!({}), 0.9)
            .unwrap();

        let _ = vault.append("a", EvidenceSource::Observation, serde_json::json!({}), 0.5);

        let report = vault.verify_integrity();
        assert!(report.all_valid);
        assert_eq!(vault.len(), 2);
    }

    // ── hash-chain integrity ────────────────────────────────────────

    #[test]
    fn hash_chain_is_valid_after_append() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"n": 2}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e3",
                EvidenceSource::Observation,
                serde_json::json!({"n": 3}),
                0.9,
            )
            .unwrap();

        let report = vault.verify_integrity();
        assert!(report.all_valid);
        assert_eq!(report.tampered_entries, 0);
        assert_eq!(report.total_entries, 3);
        assert_eq!(report.valid_entries, 3);
    }

    #[test]
    fn tampered_entry_detected() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"n": 2}),
                0.9,
            )
            .unwrap();

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
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"n": 2}),
                0.9,
            )
            .unwrap();

        // Break the chain link
        vault.entries[1].prev_hash =
            "deadbeef0000000000000000000000000000000000000000000000000000000000".into();

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
        vault
            .append(
                "genesis",
                EvidenceSource::Observation,
                serde_json::json!({"first": true}),
                1.0,
            )
            .unwrap();

        assert_eq!(vault.entries[0].prev_hash, "0");
        assert!(vault.entries[0].verify_integrity());
    }

    #[test]
    fn confidence_is_clamped() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({}),
                1.5,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({}),
                -0.5,
            )
            .unwrap();

        assert!((vault.entries[0].confidence - 1.0).abs() < f64::EPSILON);
        assert!((vault.entries[1].confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_entries_reconstructs_vault() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"x": 1}),
                0.8,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"x": 2}),
                0.8,
            )
            .unwrap();

        let entries = vault.entries.clone();
        let rebuilt = EvidenceVault::from_entries(entries);

        assert_eq!(rebuilt.len(), 2);
        assert!(rebuilt.verify_integrity().all_valid);
    }

    // ── SHA-256 hash property tests ─────────────────────────────────

    #[test]
    fn identical_entry_produces_identical_hash() {
        let entry1 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );
        let entry2 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );

        // Different timestamps will produce different hashes,
        // so we compare after equalizing timestamps
        let mut e1 = entry1;
        let mut e2 = entry2;
        e2.timestamp = e1.timestamp;
        e1.content_hash = e1.compute_content_hash();
        e2.content_hash = e2.compute_content_hash();
        assert_eq!(e1.content_hash, e2.content_hash);
    }

    #[test]
    fn changed_content_changes_hash() {
        let mut e1 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );
        let mut e2 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 999}),
            0.5,
            "0",
        );
        e2.timestamp = e1.timestamp;
        e1.content_hash = e1.compute_content_hash();
        e2.content_hash = e2.compute_content_hash();
        assert_ne!(e1.content_hash, e2.content_hash);
    }

    #[test]
    fn changed_timestamp_changes_hash() {
        let entry = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );
        let mut later = entry.clone();
        later.timestamp += chrono::Duration::seconds(60);
        later.content_hash = later.compute_content_hash();
        assert_ne!(entry.content_hash, later.content_hash);
    }

    #[test]
    fn changed_confidence_changes_hash() {
        let mut e1 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );
        let mut e2 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.9,
            "0",
        );
        e2.timestamp = e1.timestamp;
        e1.content_hash = e1.compute_content_hash();
        e2.content_hash = e2.compute_content_hash();
        assert_ne!(e1.content_hash, e2.content_hash);
    }

    #[test]
    fn changed_prev_hash_changes_hash() {
        let mut e1 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "0",
        );
        let mut e2 = EvidenceEntry::new(
            "id1",
            EvidenceSource::Observation,
            serde_json::json!({"a": 1}),
            0.5,
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        e2.timestamp = e1.timestamp;
        e1.content_hash = e1.compute_content_hash();
        e2.content_hash = e2.compute_content_hash();
        assert_ne!(e1.content_hash, e2.content_hash);
    }

    #[test]
    fn timestamp_tampering_is_detected() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();

        // Tamper timestamp without recomputing hash
        vault.entries[0].timestamp += chrono::Duration::seconds(3600);

        let report = vault.verify_integrity();
        assert!(!report.all_valid);
        assert!(report.tampered_entries >= 1);
    }

    #[test]
    fn prev_hash_tampering_is_detected() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();

        // Tamper prev_hash without recomputing hash
        vault.entries[0].prev_hash =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();

        let report = vault.verify_integrity();
        assert!(!report.all_valid);
        assert!(report.tampered_entries >= 1);
    }

    #[test]
    fn hash_is_lowercase_hex() {
        let entry = EvidenceEntry::new(
            "test",
            EvidenceSource::Observation,
            serde_json::json!({"x": 1}),
            0.5,
            "0",
        );
        assert!(entry.content_hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(entry.content_hash, entry.content_hash.to_lowercase());
    }

    // ── JSONL persistence ──────────────────────────────────────────

    #[test]
    fn save_and_load_roundtrip() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();
        vault
            .append(
                "e2",
                EvidenceSource::Observation,
                serde_json::json!({"n": 2}),
                0.8,
            )
            .unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("vault.jsonl");
        vault.save_jsonl(&path).unwrap();

        let loaded = EvidenceVault::load_jsonl(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.verify_integrity().all_valid);
        assert_eq!(loaded.get(0).unwrap().id, "e1");
    }

    #[test]
    fn loaded_vault_detects_tampering() {
        let mut vault = EvidenceVault::new();
        vault
            .append(
                "e1",
                EvidenceSource::Observation,
                serde_json::json!({"n": 1}),
                0.9,
            )
            .unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("vault.jsonl");
        vault.save_jsonl(&path).unwrap();

        // Tamper the file
        let corrupted = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\"n\":1", "\"n\":999");
        std::fs::write(&path, corrupted).unwrap();

        let result = EvidenceVault::load_jsonl(&path);
        assert!(result.is_err());
    }

    #[test]
    fn corrupt_jsonl_line_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.jsonl");
        std::fs::write(&path, "not valid json\n").unwrap();

        let result = EvidenceVault::load_jsonl(&path);
        assert!(result.is_err());
    }

    #[test]
    fn hash_is_64_chars_sha256() {
        let entry = EvidenceEntry::new(
            "test",
            EvidenceSource::Observation,
            serde_json::json!({"x": 1}),
            0.5,
            "0",
        );
        // SHA-256 produces 256 bits = 32 bytes = 64 hex chars
        assert_eq!(entry.content_hash.len(), 64);
    }
}
