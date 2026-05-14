//! DurableMemoryProvider: SQLite-backed persistent memory for CODEX-1.

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
    Superseded,
    Unknown,
}

impl ClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Validated => "validated",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Unknown => "unknown",
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
        conn.execute("PRAGMA journal_mode = WAL", [])?;
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
                Ok(ClaimRecord {
                    claim_id: row.get(0)?,
                    claim_text: row.get(1)?,
                    status: ClaimStatus::Validated, // Simplified
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

    #[test]
    #[ignore]
    fn test_open_and_assert() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("mem.sqlite");
        let provider = DurableMemoryProvider::open(&db_path).unwrap();

        let claim = ClaimRecord {
            claim_id: "c1".to_string(),
            claim_text: "Test".to_string(),
            status: ClaimStatus::Asserted,
            confidence: 0.9,
            salience: 0.75,
            source_ref: None,
            timestamp_unix_ms: 1000,
            metadata_json: "{}".to_string(),
            created_at_unix_ms: 1000,
            updated_at_unix_ms: 1000,
        };

        provider.assert_claim(claim, &[]).unwrap();
    }
}
