//! DurableMemoryProvider: SQLite-backed persistent memory for CODEX-1.

use crate::{MemoryKind, MemoryStatus};
use parking_lot::Mutex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("SQLite error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

type MemoryResult<T> = Result<T, MemoryError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ClaimStatus {
    Asserted,
    Validated,
    Rejected,
    Stale,
    Disputed,
    Superseded,
    Unknown,
}

impl ClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Validated => "validated",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Disputed => "disputed",
            Self::Superseded => "superseded",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_db_str(value: &str) -> Self {
        match value {
            "asserted" => Self::Asserted,
            "validated" => Self::Validated,
            "rejected" => Self::Rejected,
            "stale" => Self::Stale,
            "disputed" => Self::Disputed,
            "superseded" => Self::Superseded,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRecord {
    pub claim_id: String,
    pub claim_text: String,
    pub status: ClaimStatus,
    pub confidence: f32,
    pub salience: f32,
    pub source_ref: Option<String>,
    pub timestamp_unix_ms: i64,
    pub metadata_json: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ClaimQuery {
    pub status: Option<ClaimStatus>,
    pub min_confidence: Option<f32>,
    pub evidence_id: Option<String>,
    pub limit: Option<usize>,
}

/// A structured knowledge record with subject/predicate/object encoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub record_id: String,
    pub claim_id: Option<String>,
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub kind: MemoryKind,
    pub status: MemoryStatus,
    pub confidence: f32,
    pub source_ref: Option<String>,
    pub metadata_json: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub retrieval_score: f64,
    pub recency_score: f64,
    pub contradiction_ids: String, // JSON-encoded array e.g. "[]"
    pub governance_reason_code: Option<String>,
    pub is_stale: bool,
    pub is_disputed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRecordQuery<'a> {
    pub text_filter: Option<&'a str>,
    pub subject: Option<&'a str>,
    pub predicate: Option<&'a str>,
    pub object: Option<&'a str>,
    pub kind_filter: Option<MemoryKind>,
    pub status_filter: Option<MemoryStatus>,
    pub min_confidence: Option<f64>,
    pub max_confidence: Option<f64>,
    pub start_unix_ms: Option<i64>,
    pub end_unix_ms: Option<i64>,
    pub source_ref_filter: Option<&'a str>,
    pub limit: usize,
    pub offset: usize,
}

/// A link between a claim and a piece of supporting or refuting evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub link_id: i64,
    pub claim_id: String,
    pub evidence_id: String,
    pub support_type: String,
    pub confidence: f32,
}

#[allow(dead_code)]
pub struct DurableMemoryProvider {
    db_path: PathBuf,
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl DurableMemoryProvider {
    pub fn open<P: AsRef<Path>>(db_path: P) -> MemoryResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = rusqlite::Connection::open(&db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Self::init_schema(&conn)?;
        Ok(Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_schema(conn: &rusqlite::Connection) -> MemoryResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS claims (
                claim_id TEXT PRIMARY KEY,
                claim_text TEXT NOT NULL,
                status TEXT NOT NULL,
                confidence REAL NOT NULL,
                salience REAL NOT NULL DEFAULT 0.0,
                source_ref TEXT,
                timestamp_unix_ms INTEGER NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at_unix_ms INTEGER NOT NULL,
                updated_at_unix_ms INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_claims_status ON claims(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_claims_confidence ON claims(confidence DESC)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS claim_links (
                link_id INTEGER PRIMARY KEY AUTOINCREMENT,
                claim_id TEXT NOT NULL,
                evidence_id TEXT NOT NULL,
                link_type TEXT NOT NULL DEFAULT 'supports',
                weight REAL NOT NULL DEFAULT 1.0,
                UNIQUE (claim_id, evidence_id, link_type),
                FOREIGN KEY (claim_id) REFERENCES claims(claim_id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_claim_links_claim ON claim_links(claim_id)",
            [],
        )?;

        // ── claim_evidence_links: supersedes claim_links; adds support_type + confidence ──
        conn.execute(
            "CREATE TABLE IF NOT EXISTS claim_evidence_links (
                link_id   INTEGER PRIMARY KEY AUTOINCREMENT,
                claim_id  TEXT    NOT NULL,
                evidence_id TEXT  NOT NULL,
                support_type TEXT NOT NULL DEFAULT 'supports',
                confidence REAL   NOT NULL DEFAULT 1.0,
                UNIQUE (claim_id, evidence_id, support_type),
                FOREIGN KEY (claim_id) REFERENCES claims(claim_id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cel_claim ON claim_evidence_links(claim_id)",
            [],
        )?;

        // ── memory_records: structured SPO knowledge store ──
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_records (
                record_id   TEXT  PRIMARY KEY,
                claim_id    TEXT,
                subject     TEXT  NOT NULL,
                predicate   TEXT  NOT NULL,
                object      TEXT  NOT NULL,
                kind        TEXT  NOT NULL,
                status      TEXT  NOT NULL,
                confidence  REAL  NOT NULL DEFAULT 1.0,
                source_ref  TEXT,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at_unix_ms  INTEGER NOT NULL,
                updated_at_unix_ms  INTEGER NOT NULL,
                retrieval_score REAL NOT NULL DEFAULT 0.0,
                recency_score REAL NOT NULL DEFAULT 0.0,
                contradiction_ids TEXT NOT NULL DEFAULT '[]',
                governance_reason_code TEXT,
                is_stale INTEGER NOT NULL DEFAULT 0,
                is_disputed INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (claim_id) REFERENCES claims(claim_id) ON DELETE SET NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mr_claim_id  ON memory_records(claim_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mr_subject   ON memory_records(subject)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mr_kind      ON memory_records(kind)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mr_status    ON memory_records(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mr_created_at ON memory_records(created_at_unix_ms)",
            [],
        )?;

        Ok(())
    }

    pub fn assert_claim(&self, claim: ClaimRecord, evidence_ids: &[String]) -> MemoryResult<()> {
        let conn = self.conn.lock();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let mut stmt = conn.prepare("INSERT INTO claims (claim_id, claim_text, status, confidence, salience, source_ref, timestamp_unix_ms, metadata_json, created_at_unix_ms, updated_at_unix_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")?;
        stmt.execute(params![
            claim.claim_id,
            claim.claim_text,
            claim.status.as_str(),
            claim.confidence,
            claim.salience,
            claim.source_ref,
            claim.timestamp_unix_ms,
            claim.metadata_json,
            now,
            now,
        ])?;
        drop(stmt);

        for evidence_id in evidence_ids {
            let mut stmt = conn.prepare(
                "INSERT OR IGNORE INTO claim_links (claim_id, evidence_id, link_type, weight) VALUES (?, ?, 'supports', 1.0)",
            )?;
            stmt.execute(params![claim.claim_id, evidence_id])?;
            drop(stmt);
        }

        Ok(())
    }

    pub fn validate_claim(
        &self,
        claim_id: &str,
        new_status: ClaimStatus,
        confidence: Option<f32>,
    ) -> MemoryResult<()> {
        let conn = self.conn.lock();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        if let Some(conf) = confidence {
            conn.execute(
                "UPDATE claims SET status = ?, confidence = ?, updated_at_unix_ms = ? WHERE claim_id = ?",
                params![new_status.as_str(), conf, now, claim_id],
            )?;
        } else {
            conn.execute(
                "UPDATE claims SET status = ?, updated_at_unix_ms = ? WHERE claim_id = ?",
                params![new_status.as_str(), now, claim_id],
            )?;
        }

        Ok(())
    }

    pub fn get_by_status(
        &self,
        status: ClaimStatus,
        limit: usize,
    ) -> MemoryResult<Vec<ClaimRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT claim_id, claim_text, status, confidence, salience, source_ref, timestamp_unix_ms, metadata_json, created_at_unix_ms, updated_at_unix_ms FROM claims WHERE status = ? ORDER BY confidence DESC LIMIT ?"
        )?;

        let records = stmt
            .query_map(params![status.as_str(), limit as i32], |row| {
                let status_raw: String = row.get(2)?;
                Ok(ClaimRecord {
                    claim_id: row.get(0)?,
                    claim_text: row.get(1)?,
                    status: ClaimStatus::from_db_str(status_raw.as_str()),
                    confidence: row.get(3)?,
                    salience: row.get(4)?,
                    source_ref: row.get(5)?,
                    timestamp_unix_ms: row.get(6)?,
                    metadata_json: row.get(7)?,
                    created_at_unix_ms: row.get(8)?,
                    updated_at_unix_ms: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    // ── memory_records API ────────────────────────────────────────────────

    /// Insert a structured SPO knowledge record.
    pub fn insert_record(&self, record: MemoryRecord) -> MemoryResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO memory_records
                (record_id, claim_id, subject, predicate, object, kind, status,
                 confidence, source_ref, metadata_json, created_at_unix_ms, updated_at_unix_ms,
                 retrieval_score, recency_score, contradiction_ids,
                 governance_reason_code, is_stale, is_disputed)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.record_id,
                record.claim_id,
                record.subject,
                record.predicate,
                record.object,
                record.kind.as_str(),
                record.status.as_str(),
                record.confidence,
                record.source_ref,
                record.metadata_json,
                record.created_at_unix_ms,
                record.updated_at_unix_ms,
                record.retrieval_score,
                record.recency_score,
                record.contradiction_ids,
                record.governance_reason_code,
                record.is_stale as i64,
                record.is_disputed as i64,
            ],
        )?;
        Ok(())
    }

    /// Return all records of the given kind, ordered by confidence descending.
    pub fn get_by_kind(&self, kind: MemoryKind, limit: usize) -> MemoryResult<Vec<MemoryRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT record_id, claim_id, subject, predicate, object, kind, status,
                    confidence, source_ref, metadata_json, created_at_unix_ms, updated_at_unix_ms,
                    retrieval_score, recency_score, contradiction_ids,
                    governance_reason_code, is_stale, is_disputed
             FROM memory_records
             WHERE kind = ?
             ORDER BY confidence DESC
             LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![kind.as_str(), limit as i64], Self::map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Return all records whose subject exactly matches `subject`.
    pub fn get_by_subject(&self, subject: &str, limit: usize) -> MemoryResult<Vec<MemoryRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT record_id, claim_id, subject, predicate, object, kind, status,
                    confidence, source_ref, metadata_json, created_at_unix_ms, updated_at_unix_ms,
                    retrieval_score, recency_score, contradiction_ids,
                    governance_reason_code, is_stale, is_disputed
             FROM memory_records
             WHERE subject = ?
             ORDER BY confidence DESC
             LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![subject, limit as i64], Self::map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Update the status of a memory record.
    pub fn update_record_status(
        &self,
        record_id: &str,
        new_status: MemoryStatus,
    ) -> MemoryResult<()> {
        let conn = self.conn.lock();
        let now = Self::now_ms();
        conn.execute(
            "UPDATE memory_records SET status = ?, updated_at_unix_ms = ? WHERE record_id = ?",
            params![new_status.as_str(), now, record_id],
        )?;
        Ok(())
    }

    /// Link a piece of evidence to a claim via `claim_evidence_links`.
    pub fn link_evidence(
        &self,
        claim_id: &str,
        evidence_id: &str,
        support_type: &str,
        confidence: f32,
    ) -> MemoryResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO claim_evidence_links
                (claim_id, evidence_id, support_type, confidence)
             VALUES (?, ?, ?, ?)",
            params![claim_id, evidence_id, support_type, confidence],
        )?;
        Ok(())
    }

    /// Return all evidence links for the given claim.
    pub fn get_linked_evidence(&self, claim_id: &str) -> MemoryResult<Vec<EvidenceLink>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT link_id, claim_id, evidence_id, support_type, confidence
             FROM claim_evidence_links
             WHERE claim_id = ?
             ORDER BY confidence DESC",
        )?;
        let rows = stmt
            .query_map(params![claim_id], |row| {
                Ok(EvidenceLink {
                    link_id: row.get(0)?,
                    claim_id: row.get(1)?,
                    evidence_id: row.get(2)?,
                    support_type: row.get(3)?,
                    confidence: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Search records whose predicate contains `predicate` as a substring.
    pub fn search_by_predicate(
        &self,
        predicate: &str,
        limit: usize,
    ) -> MemoryResult<Vec<MemoryRecord>> {
        let conn = self.conn.lock();
        let pattern = format!("%{predicate}%");
        let mut stmt = conn.prepare(
            "SELECT record_id, claim_id, subject, predicate, object, kind, status,
                    confidence, source_ref, metadata_json, created_at_unix_ms, updated_at_unix_ms,
                    retrieval_score, recency_score, contradiction_ids,
                    governance_reason_code, is_stale, is_disputed
             FROM memory_records
             WHERE predicate LIKE ?
             ORDER BY confidence DESC
             LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![pattern, limit as i64], Self::map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Delete a memory record by its ID.
    pub fn delete_record(&self, record_id: &str) -> MemoryResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM memory_records WHERE record_id = ?",
            params![record_id],
        )?;
        Ok(())
    }

    /// Query `memory_records` with optional filters, returning matching records
    /// ordered by confidence descending.
    ///
    /// Text search targets subject, predicate, and object via LIKE.
    pub fn query_records(&self, query: &MemoryRecordQuery<'_>) -> MemoryResult<Vec<MemoryRecord>> {
        use rusqlite::types::Value;
        let conn = self.conn.lock();
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Value> = Vec::new();

        if let Some(text) = query.text_filter {
            if !text.is_empty() {
                let pattern = format!("%{text}%");
                conditions.push("(subject LIKE ? OR predicate LIKE ? OR object LIKE ?)".into());
                param_values.push(Value::Text(pattern.clone()));
                param_values.push(Value::Text(pattern.clone()));
                param_values.push(Value::Text(pattern));
            }
        }
        if let Some(s) = query.subject {
            conditions.push("subject LIKE ?".into());
            param_values.push(Value::Text(format!("%{s}%")));
        }
        if let Some(p) = query.predicate {
            conditions.push("predicate LIKE ?".into());
            param_values.push(Value::Text(format!("%{p}%")));
        }
        if let Some(o) = query.object {
            conditions.push("object LIKE ?".into());
            param_values.push(Value::Text(format!("%{o}%")));
        }
        if let Some(k) = query.kind_filter {
            conditions.push("kind = ?".into());
            param_values.push(Value::Text(k.as_str().to_string()));
        }
        if let Some(s) = query.status_filter {
            conditions.push("status = ?".into());
            param_values.push(Value::Text(s.as_str().to_string()));
        }
        if let Some(min) = query.min_confidence {
            conditions.push("confidence >= ?".into());
            param_values.push(Value::Real(min));
        }
        if let Some(max) = query.max_confidence {
            conditions.push("confidence <= ?".into());
            param_values.push(Value::Real(max));
        }
        if let Some(start) = query.start_unix_ms {
            conditions.push("created_at_unix_ms >= ?".into());
            param_values.push(Value::Integer(start));
        }
        if let Some(end) = query.end_unix_ms {
            conditions.push("created_at_unix_ms <= ?".into());
            param_values.push(Value::Integer(end));
        }
        if let Some(sr) = query.source_ref_filter {
            conditions.push("source_ref LIKE ?".into());
            param_values.push(Value::Text(format!("%{sr}%")));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT record_id, claim_id, subject, predicate, object, kind, status, \
             confidence, source_ref, metadata_json, created_at_unix_ms, updated_at_unix_ms, \
             retrieval_score, recency_score, contradiction_ids, \
             governance_reason_code, is_stale, is_disputed \
             FROM memory_records \
             {where_clause} \
             ORDER BY confidence DESC \
             LIMIT ? OFFSET ?",
        );

        param_values.push(Value::Integer(query.limit as i64));
        param_values.push(Value::Integer(query.offset as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(param_values), Self::map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    fn map_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryRecord> {
        let kind_raw: String = row.get(5)?;
        let status_raw: String = row.get(6)?;
        Ok(MemoryRecord {
            record_id: row.get(0)?,
            claim_id: row.get(1)?,
            subject: row.get(2)?,
            predicate: row.get(3)?,
            object: row.get(4)?,
            kind: MemoryKind::from_db_str(&kind_raw),
            status: MemoryStatus::from_db_str(&status_raw),
            confidence: row.get(7)?,
            source_ref: row.get(8)?,
            metadata_json: row.get(9)?,
            created_at_unix_ms: row.get(10)?,
            updated_at_unix_ms: row.get(11)?,
            retrieval_score: row.get(12)?,
            recency_score: row.get(13)?,
            contradiction_ids: row
                .get::<_, Option<String>>(14)?
                .unwrap_or_else(|| "[]".to_string()),
            governance_reason_code: row.get(15)?,
            is_stale: row.get::<_, i64>(16)? != 0,
            is_disputed: row.get::<_, i64>(17)? != 0,
        })
    }

    pub fn stats(&self) -> MemoryResult<serde_json::Value> {
        let conn = self.conn.lock();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM claims", [], |row| row.get(0))?;

        Ok(serde_json::json!({ "total_claims": total }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn claim(id: &str, status: ClaimStatus, confidence: f32) -> ClaimRecord {
        ClaimRecord {
            claim_id: id.to_string(),
            claim_text: format!("Claim {id}"),
            status,
            confidence,
            salience: 0.75,
            source_ref: None,
            timestamp_unix_ms: 1000,
            metadata_json: "{}".to_string(),
            created_at_unix_ms: 1000,
            updated_at_unix_ms: 1000,
        }
    }

    #[test]
    fn test_open_and_assert() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("mem.sqlite");
        let provider = DurableMemoryProvider::open(&db_path).unwrap();

        let claim = claim("c1", ClaimStatus::Asserted, 0.9);

        provider.assert_claim(claim, &[]).unwrap();

        let asserted = provider.get_by_status(ClaimStatus::Asserted, 10).unwrap();
        assert_eq!(asserted.len(), 1);
        assert_eq!(asserted[0].status, ClaimStatus::Asserted);
        assert_eq!(asserted[0].claim_id, "c1");
    }

    #[test]
    fn get_by_status_filters_rejected_stale_disputed_superseded() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("mem.sqlite");
        let provider = DurableMemoryProvider::open(&db_path).unwrap();

        provider
            .assert_claim(claim("c_rejected", ClaimStatus::Rejected, 0.8), &[])
            .unwrap();
        provider
            .assert_claim(claim("c_stale", ClaimStatus::Stale, 0.7), &[])
            .unwrap();
        provider
            .assert_claim(claim("c_disputed", ClaimStatus::Disputed, 0.6), &[])
            .unwrap();
        provider
            .assert_claim(claim("c_superseded", ClaimStatus::Superseded, 0.5), &[])
            .unwrap();

        let rejected = provider.get_by_status(ClaimStatus::Rejected, 10).unwrap();
        let stale = provider.get_by_status(ClaimStatus::Stale, 10).unwrap();
        let disputed = provider.get_by_status(ClaimStatus::Disputed, 10).unwrap();
        let superseded = provider.get_by_status(ClaimStatus::Superseded, 10).unwrap();

        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].claim_id, "c_rejected");
        assert_eq!(rejected[0].status, ClaimStatus::Rejected);

        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].claim_id, "c_stale");
        assert_eq!(stale[0].status, ClaimStatus::Stale);

        assert_eq!(disputed.len(), 1);
        assert_eq!(disputed[0].claim_id, "c_disputed");
        assert_eq!(disputed[0].status, ClaimStatus::Disputed);

        assert_eq!(superseded.len(), 1);
        assert_eq!(superseded[0].claim_id, "c_superseded");
        assert_eq!(superseded[0].status, ClaimStatus::Superseded);
    }

    #[test]
    fn get_by_status_returns_ordered_by_confidence_desc() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("mem.sqlite");
        let provider = DurableMemoryProvider::open(&db_path).unwrap();

        provider
            .assert_claim(claim("c1", ClaimStatus::Rejected, 0.20), &[])
            .unwrap();
        provider
            .assert_claim(claim("c2", ClaimStatus::Rejected, 0.80), &[])
            .unwrap();
        provider
            .assert_claim(claim("c3", ClaimStatus::Rejected, 0.50), &[])
            .unwrap();

        let rejected = provider.get_by_status(ClaimStatus::Rejected, 10).unwrap();
        assert_eq!(rejected.len(), 3);
        assert_eq!(rejected[0].claim_id, "c2");
        assert_eq!(rejected[1].claim_id, "c3");
        assert_eq!(rejected[2].claim_id, "c1");
    }

    // ── memory_records tests ──────────────────────────────────────────────

    fn record(
        id: &str,
        subject: &str,
        predicate: &str,
        object: &str,
        kind: MemoryKind,
    ) -> MemoryRecord {
        MemoryRecord {
            record_id: id.to_string(),
            claim_id: None,
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: Some(object.to_string()),
            kind,
            status: MemoryStatus::Active,
            confidence: 0.9,
            source_ref: None,
            metadata_json: "{}".to_string(),
            created_at_unix_ms: 1000,
            updated_at_unix_ms: 1000,
            retrieval_score: 0.0,
            recency_score: 0.0,
            contradiction_ids: "[]".to_string(),
            governance_reason_code: None,
            is_stale: false,
            is_disputed: false,
        }
    }

    #[test]
    fn test_insert_record_basic() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        let rec = record("r1", "codex", "is_type", "runtime", MemoryKind::Factual);
        provider.insert_record(rec).unwrap();
        let results = provider.get_by_kind(MemoryKind::Factual, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record_id, "r1");
    }

    #[test]
    fn test_get_by_kind_factual() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("f1", "s", "p", "o", MemoryKind::Factual))
            .unwrap();
        provider
            .insert_record(record("p1", "s", "p", "o", MemoryKind::Procedural))
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Factual, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record_id, "f1");
        assert_eq!(results[0].kind, MemoryKind::Factual);
    }

    #[test]
    fn test_get_by_kind_procedural() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("p1", "s", "p", "o", MemoryKind::Procedural))
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Procedural, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Procedural);
    }

    #[test]
    fn test_get_by_kind_episodic() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("e1", "s", "p", "o", MemoryKind::Episodic))
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Episodic, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Episodic);
    }

    #[test]
    fn test_get_by_kind_semantic() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("s1", "s", "p", "o", MemoryKind::Semantic))
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Semantic, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Semantic);
    }

    #[test]
    fn test_get_by_kind_contextual() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("c1", "s", "p", "o", MemoryKind::Contextual))
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Contextual, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Contextual);
    }

    #[test]
    fn test_get_by_kind_empty() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        let results = provider.get_by_kind(MemoryKind::Factual, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_by_subject_exact_match() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("r1", "alpha", "is", "x", MemoryKind::Factual))
            .unwrap();
        provider
            .insert_record(record("r2", "beta", "is", "y", MemoryKind::Factual))
            .unwrap();
        let results = provider.get_by_subject("alpha", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record_id, "r1");
    }

    #[test]
    fn test_get_by_subject_no_match() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("r1", "alpha", "is", "x", MemoryKind::Factual))
            .unwrap();
        let results = provider.get_by_subject("gamma", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_update_record_status() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("r1", "s", "p", "o", MemoryKind::Factual))
            .unwrap();
        provider
            .update_record_status("r1", MemoryStatus::Validated)
            .unwrap();
        let results = provider.get_by_kind(MemoryKind::Factual, 10).unwrap();
        assert_eq!(results[0].status, MemoryStatus::Validated);
    }

    #[test]
    fn test_link_evidence_and_get_linked() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        let c = claim("c1", ClaimStatus::Asserted, 0.9);
        provider.assert_claim(c, &[]).unwrap();

        provider
            .link_evidence("c1", "ev-001", "supports", 0.85)
            .unwrap();
        provider
            .link_evidence("c1", "ev-002", "refutes", 0.30)
            .unwrap();

        let links = provider.get_linked_evidence("c1").unwrap();
        assert_eq!(links.len(), 2);
        // ordered by confidence desc
        assert_eq!(links[0].evidence_id, "ev-001");
        assert_eq!(links[0].support_type, "supports");
        assert_eq!(links[1].evidence_id, "ev-002");
    }

    #[test]
    fn test_link_evidence_duplicate_ignored() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        let c = claim("c1", ClaimStatus::Asserted, 0.9);
        provider.assert_claim(c, &[]).unwrap();

        provider
            .link_evidence("c1", "ev-001", "supports", 0.9)
            .unwrap();
        // identical (claim_id, evidence_id, support_type) → should be silently ignored
        provider
            .link_evidence("c1", "ev-001", "supports", 0.9)
            .unwrap();

        let links = provider.get_linked_evidence("c1").unwrap();
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn test_search_by_predicate() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record(
                "r1",
                "s",
                "causes_shutdown",
                "o",
                MemoryKind::Factual,
            ))
            .unwrap();
        provider
            .insert_record(record("r2", "s2", "causes_delay", "o", MemoryKind::Factual))
            .unwrap();
        provider
            .insert_record(record(
                "r3",
                "s3",
                "monitors_latency",
                "o",
                MemoryKind::Factual,
            ))
            .unwrap();
        let results = provider.search_by_predicate("causes", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_predicate_no_match() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("r1", "s", "causes", "o", MemoryKind::Factual))
            .unwrap();
        let results = provider.search_by_predicate("monitors", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_delete_record() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        provider
            .insert_record(record("r1", "s", "p", "o", MemoryKind::Factual))
            .unwrap();
        provider.delete_record("r1").unwrap();
        let results = provider.get_by_kind(MemoryKind::Factual, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_delete_record_not_found_is_ok() {
        let dir = tempdir().unwrap();
        let provider = DurableMemoryProvider::open(dir.path().join("m.sqlite")).unwrap();
        // Deleting a nonexistent record should not return an error
        provider.delete_record("nonexistent-id").unwrap();
    }
}
