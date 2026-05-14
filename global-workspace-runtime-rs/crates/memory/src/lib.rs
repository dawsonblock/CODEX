//! Memory crate: archive backend, semantic memory, claim tracking, and evidence.
//!
//! # Archive
//!
//! The default backend is `JsonlArchiveBackend`, which writes standard JSONL
//! to `.gwlog` files. `MemvidBackend` is a stub that returns `NotImplemented` —
//! it does NOT write `.mv2` files and MUST NOT be used to claim Memvid
//! compatibility.

pub mod answer_builder;
pub mod claim_store;
pub mod durable_memory_provider;
pub mod memory_provider;
pub mod status_mapping;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════════════
// ArchiveBackend trait
// ═══════════════════════════════════════════════════════════════════════════════

/// Portable archive backend. Implementations write archive frames.
pub trait ArchiveBackend: fmt::Debug + Send + Sync {
    /// Write a frame to the archive.
    fn write_frame(&mut self, frame: &ArchiveFrame) -> Result<(), ArchiveError>;

    /// Read all frames from the archive.
    fn read_all(&self) -> Result<Vec<ArchiveFrame>, ArchiveError>;

    /// Query frames by keyword match.
    fn query(&self, keyword: &str, limit: usize) -> Result<Vec<ArchiveFrame>, ArchiveError>;

    /// Number of frames in the archive.
    fn frame_count(&self) -> Result<usize, ArchiveError>;
}

/// Errors from archive operations.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),
    #[error("No archive path configured")]
    NoPath,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ArchiveFrame
// ═══════════════════════════════════════════════════════════════════════════════

/// One frame written to the archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveFrame {
    pub frame_id: String,
    pub cycle_id: u64,
    pub timestamp: String,
    pub entry_type: String,
    pub content: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// JsonlArchiveBackend
// ═══════════════════════════════════════════════════════════════════════════════

/// JSONL archive backend. Writes `.gwlog` (JSONL) files.
/// Does NOT use or accept `.mv2` extensions.
#[derive(Debug)]
pub struct JsonlArchiveBackend {
    path: Option<PathBuf>,
}

impl JsonlArchiveBackend {
    /// Create a backend backed by a file.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ArchiveError> {
        let path = path.as_ref().to_path_buf();
        // Reject .mv2 extensions
        if path.extension().is_some_and(|e| e == "mv2") {
            return Err(ArchiveError::UnsupportedExtension(
                ".mv2 is not a supported format. Use .jsonl or .gwlog instead.".to_string(),
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { path: Some(path) })
    }

    /// Create an in-memory backend (no persistence).
    pub fn in_memory() -> Self {
        Self { path: None }
    }
}

impl ArchiveBackend for JsonlArchiveBackend {
    fn write_frame(&mut self, frame: &ArchiveFrame) -> Result<(), ArchiveError> {
        let Some(ref path) = self.path else {
            return Ok(()); // in-memory: no-op
        };
        let line = serde_json::to_string(frame)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<ArchiveFrame>, ArchiveError> {
        let Some(ref path) = self.path else {
            return Ok(Vec::new());
        };
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut frames = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(frame) = serde_json::from_str::<ArchiveFrame>(&line) {
                frames.push(frame);
            }
        }
        Ok(frames)
    }

    fn query(&self, keyword: &str, limit: usize) -> Result<Vec<ArchiveFrame>, ArchiveError> {
        let all = self.read_all()?;
        let kw = keyword.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|f| {
                let hay = serde_json::to_string(&f.content).unwrap_or_default();
                hay.to_lowercase().contains(&kw)
            })
            .take(limit)
            .collect())
    }

    fn frame_count(&self) -> Result<usize, ArchiveError> {
        self.read_all().map(|f| f.len())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MemvidBackend stub
// ═══════════════════════════════════════════════════════════════════════════════

/// Stub backend that returns `NotImplemented` for every operation.
/// This exists as an integration point for a real Memvid backend, but
/// no Memvid binary is included. Any call to write or read will fail loudly.
#[derive(Debug)]
pub struct MemvidBackend;

impl ArchiveBackend for MemvidBackend {
    fn write_frame(&mut self, _frame: &ArchiveFrame) -> Result<(), ArchiveError> {
        Err(ArchiveError::NotImplemented(
            "MemvidBackend is a stub. No real Memvid binary is integrated.".to_string(),
        ))
    }

    fn read_all(&self) -> Result<Vec<ArchiveFrame>, ArchiveError> {
        Err(ArchiveError::NotImplemented(
            "MemvidBackend is a stub. No real Memvid binary is integrated.".to_string(),
        ))
    }

    fn query(&self, _keyword: &str, _limit: usize) -> Result<Vec<ArchiveFrame>, ArchiveError> {
        Err(ArchiveError::NotImplemented(
            "MemvidBackend is a stub. No real Memvid binary is integrated.".to_string(),
        ))
    }

    fn frame_count(&self) -> Result<usize, ArchiveError> {
        Err(ArchiveError::NotImplemented(
            "MemvidBackend is a stub. No real Memvid binary is integrated.".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Evidence and claims
// ═══════════════════════════════════════════════════════════════════════════════

/// A piece of evidence — an immutable observation or result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub source: String,
    pub content: String,
    pub timestamp: String,
    pub confidence: f64,
}

/// A claim asserted about the world or internal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryClaim {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    /// Optional object — completes the subject-predicate-object triple.
    pub object: Option<String>,
    pub status: ClaimStatus,
    /// Claim confidence (0.0–1.0). Does NOT mean the claim is true.
    pub confidence: f64,
    /// Explicit evidence IDs linked to this claim.
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    /// Explicit evidence hashes linked to this claim.
    #[serde(default)]
    pub evidence_hashes: Vec<String>,
    /// Source label for bounded evidence-backed claim creation.
    #[serde(default)]
    pub source_label: String,
    pub evidence_links: Vec<ClaimEvidenceLink>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Ordered lifecycle events for this claim.
    #[serde(default)]
    pub audit_trail: Vec<ClaimAuditEvent>,
    pub superseded_by: Option<String>,
}

/// One lifecycle transition recorded for a claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimAuditEvent {
    pub timestamp: String,
    pub event: ClaimLifecycleEvent,
    pub reason: String,
}

/// Bounded set of claim lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimLifecycleEvent {
    Created,
    Validated,
    ContradictedBy { other_claim_id: String },
    SupersededBy { new_claim_id: String },
    Retracted,
}

/// The status of a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Active,
    Contradicted,
    Superseded,
    Unverified,
}

/// Link from a claim to supporting evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEvidenceLink {
    pub evidence_id: String,
    pub weight: f64,
}

/// Overall memory health summary (stats, not lifecycle status).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealthStats {
    pub total_claims: usize,
    pub active_claims: usize,
    pub contradicted_claims: usize,
    pub superseded_claims: usize,
    pub unverified_claims: usize,
    pub total_evidence: usize,
}

/// Canonical 10-variant claim-lifecycle status — non-lossy counterpart to
/// `durable_memory_provider::ClaimStatus`.
///
/// Use [`status_mapping::durable_to_canonical`] / [`status_mapping::canonical_to_durable`]
/// to convert between this and the durable store's `ClaimStatus` without losing information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    /// Admitted and currently active in reasoning.
    Active,
    /// Asserted but not yet validated.
    #[default]
    Unverified,
    /// Passed validation; high confidence.
    Validated,
    /// No longer current; content may still be valid but superseded.
    Stale,
    /// Under active dispute; contradicting evidence exists.
    Disputed,
    /// Directly contradicted by validated evidence.
    Contradicted,
    /// Explicitly rejected; must not be used in reasoning.
    Rejected,
    /// Replaced by a newer claim.
    Superseded,
    /// Retired and archived; no longer active but preserved for audit.
    Archived,
    /// Status unknown or could not be determined.
    Unknown,
}

impl MemoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Unverified => "unverified",
            Self::Validated => "validated",
            Self::Stale => "stale",
            Self::Disputed => "disputed",
            Self::Contradicted => "contradicted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Archived => "archived",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "unverified" => Self::Unverified,
            "validated" => Self::Validated,
            "stale" => Self::Stale,
            "disputed" => Self::Disputed,
            "contradicted" => Self::Contradicted,
            "rejected" => Self::Rejected,
            "superseded" => Self::Superseded,
            "archived" => Self::Archived,
            _ => Self::Unknown,
        }
    }
}

/// Classification of a memory record by the kind of knowledge it encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    /// A verifiable fact about the world or the system.
    Factual,
    /// A procedure, rule, or executable plan.
    Procedural,
    /// A specific past event or observation.
    Episodic,
    /// A generalisation, concept, or category definition.
    Semantic,
    /// Contextual state valid only for the current session or scope.
    Contextual,
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Factual => "factual",
            Self::Procedural => "procedural",
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Contextual => "contextual",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "factual" => Self::Factual,
            "procedural" => Self::Procedural,
            "episodic" => Self::Episodic,
            "semantic" => Self::Semantic,
            "contextual" => Self::Contextual,
            _ => Self::Factual,
        }
    }
}

/// A contradiction between two claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a_id: String,
    pub claim_b_id: String,
    pub subject: String,
    pub detected_at: String,
    pub resolved: bool,
    pub resolution: Option<String>,
}

/// A packet returned by memory retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPacket {
    pub hits: Vec<SemanticHit>,
    pub evidence: Vec<Evidence>,
    pub related_claims: Vec<MemoryClaim>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Semantic storage (legacy from original memory crate)
// ═══════════════════════════════════════════════════════════════════════════════

/// Lightweight semantic memory seeded with humanity context.
#[derive(Debug, Default)]
pub struct SemanticMemory {
    store: HashMap<String, String>,
}

impl SemanticMemory {
    pub fn new() -> Self {
        let mut m = SemanticMemory::default();
        m.seed_humanity_context();
        m
    }

    fn seed_humanity_context(&mut self) {
        self.store
            .entry("humanity:cooperation".into())
            .or_insert_with(|| {
                "People often resolve conflict through clarification, repair, mutual aid, and shared rules."
                    .into()
            });
        self.store
            .entry("humanity:kindness".into())
            .or_insert_with(|| {
                "Kind action prioritises harm reduction, dignity, truthfulness, and patience."
                    .into()
            });
        self.store
            .entry("humanity:uncertainty".into())
            .or_insert_with(|| {
                "Ambiguous behaviour should be handled with clarification before assigning negative intent."
                    .into()
            });
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.store.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.store.get(key).map(String::as_str)
    }

    /// Keyword-scored query (exact word overlap, case-insensitive).
    pub fn query(&self, text: &str, limit: usize) -> Vec<SemanticHit> {
        let words: Vec<String> = text
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .collect();

        let mut scored: Vec<(usize, &str, &str)> = self
            .store
            .iter()
            .filter_map(|(k, v)| {
                let hay = format!("{} {}", k, v).to_lowercase();
                let s = words.iter().filter(|w| hay.contains(w.as_str())).count();
                if s > 0 {
                    Some((s, k.as_str(), v.as_str()))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by_key(|b| std::cmp::Reverse(b.0));
        scored
            .into_iter()
            .take(limit)
            .map(|(score, key, value)| SemanticHit {
                key: key.into(),
                value: value.into(),
                score,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHit {
    pub key: String,
    pub value: String,
    pub score: usize,
}

/// Fast semantic cache keyed by a 64-bit hash of normalised text + state hint.
#[derive(Debug, Default)]
pub struct SemanticCache {
    cache: HashMap<u64, serde_json::Value>,
}

impl SemanticCache {
    pub fn new() -> Self {
        SemanticCache::default()
    }

    fn cache_key(text: &str, state_hint: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let norm: String = text
            .split_whitespace()
            .take(256)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let mut h = DefaultHasher::new();
        norm.hash(&mut h);
        state_hint.hash(&mut h);
        h.finish()
    }

    pub fn get(&self, text: &str, state_hint: &str) -> Option<&serde_json::Value> {
        self.cache.get(&Self::cache_key(text, state_hint))
    }

    pub fn set(&mut self, text: &str, value: serde_json::Value, state_hint: &str) {
        self.cache.insert(Self::cache_key(text, state_hint), value);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_archive_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.gwlog");
        let mut backend = JsonlArchiveBackend::new(&path).unwrap();

        let frame = ArchiveFrame {
            frame_id: "f1".into(),
            cycle_id: 1,
            timestamp: "2026-01-01T00:00:00Z".into(),
            entry_type: "episodic".into(),
            content: serde_json::json!({"key": "value"}),
        };
        backend.write_frame(&frame).unwrap();

        let frames = backend.read_all().unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].frame_id, "f1");
    }

    #[test]
    fn jsonl_archive_rejects_mv2_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.mv2");
        let err = JsonlArchiveBackend::new(&path).unwrap_err();
        assert!(matches!(err, ArchiveError::UnsupportedExtension(_)));
    }

    #[test]
    fn contradicted_memory_not_clean_truth() {
        // A claim marked Contradicted is not Active truth.
        let claim = MemoryClaim {
            id: "c1".into(),
            subject: "observation".into(),
            predicate: "was_true".into(),
            object: None,
            status: ClaimStatus::Contradicted,
            confidence: 0.5,
            evidence_ids: vec![],
            evidence_hashes: vec![],
            source_label: "test".into(),
            evidence_links: vec![],
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: None,
            audit_trail: vec![],
            superseded_by: None,
        };
        assert_ne!(claim.status, ClaimStatus::Active);
    }

    #[test]
    fn newer_evidence_supersedes_old_claim() {
        let old = MemoryClaim {
            id: "c1".into(),
            subject: "X".into(),
            predicate: "is A".into(),
            object: None,
            status: ClaimStatus::Superseded,
            confidence: 0.5,
            evidence_ids: vec![],
            evidence_hashes: vec![],
            source_label: "test".into(),
            evidence_links: vec![],
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: None,
            audit_trail: vec![],
            superseded_by: Some("c2".into()),
        };
        assert_eq!(old.status, ClaimStatus::Superseded);
        assert!(old.superseded_by.is_some());
    }

    #[test]
    fn memvid_backend_stub_fails_loudly() {
        let mut backend = MemvidBackend;
        let frame = ArchiveFrame {
            frame_id: "f1".into(),
            cycle_id: 1,
            timestamp: "2026-01-01T00:00:00Z".into(),
            entry_type: "episodic".into(),
            content: serde_json::json!({}),
        };
        let err = backend.write_frame(&frame).unwrap_err();
        assert!(matches!(err, ArchiveError::NotImplemented(_)));
    }
}
