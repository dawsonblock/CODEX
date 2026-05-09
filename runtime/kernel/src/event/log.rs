use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::event::types::RuntimeEvent;

/// A timestamped entry in the event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEntry {
    /// Unix timestamp in milliseconds.
    pub ts_ms: u64,
    pub event: RuntimeEvent,
}

/// Append-only event log.
///
/// Events are the single source of truth for `RuntimeState`. No module may
/// mutate `RuntimeState` directly — it must emit an event and the reducer
/// applies it.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    entries: Vec<EventEntry>,
    /// Optional path for JSONL persistence.
    path: Option<PathBuf>,
}

impl EventLog {
    /// Create an in-memory log with no persistence.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a log backed by a JSONL file at `path`.
    ///
    /// If the file already exists its contents are loaded on construction so
    /// that the in-memory view starts in a consistent state.
    pub fn with_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let entries = if path.exists() {
            Self::load_jsonl(&path)?
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Vec::new()
        };
        Ok(Self {
            entries,
            path: Some(path),
        })
    }

    /// Append an event, recording the current timestamp.
    pub fn append(&mut self, event: RuntimeEvent) -> std::io::Result<()> {
        let entry = EventEntry {
            ts_ms: current_ts_ms(),
            event,
        };
        if let Some(ref path) = self.path {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let line = serde_json::to_string(&entry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            writeln!(file, "{}", line)?;
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Return all entries (oldest first).
    pub fn entries(&self) -> &[EventEntry] {
        &self.entries
    }

    /// Total number of logged events.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the log contains no events.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize all entries to a JSONL string.
    pub fn to_jsonl(&self) -> String {
        self.entries
            .iter()
            .map(|e| serde_json::to_string(e).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Deserialize entries from a JSONL string.
    ///
    /// Lines that fail to parse are silently skipped so that partially-written
    /// files do not prevent replay.
    pub fn from_jsonl(data: &str) -> Self {
        let entries = data
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<EventEntry>(l).ok())
            .collect();
        Self {
            entries,
            path: None,
        }
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn load_jsonl(path: &Path) -> std::io::Result<Vec<EventEntry>> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let entries = reader
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                if line.trim().is_empty() {
                    return None;
                }
                serde_json::from_str::<EventEntry>(&line).ok()
            })
            .collect();
        Ok(entries)
    }
}

fn current_ts_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ── round-trip test ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::RuntimeEvent;

    #[test]
    fn jsonl_round_trip_preserves_events() {
        let mut log = EventLog::new();
        log.append(RuntimeEvent::ObservationReceived {
            input: "hello".into(),
            source: "test".into(),
            cycle_id: 1,
        })
        .unwrap();
        log.append(RuntimeEvent::MemoryQueried {
            query: "hello".into(),
            cycle_id: 1,
        })
        .unwrap();

        let jsonl = log.to_jsonl();
        let restored = EventLog::from_jsonl(&jsonl);
        assert_eq!(restored.entries(), log.entries());
    }

    #[test]
    fn empty_log_is_empty() {
        let log = EventLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }
}
