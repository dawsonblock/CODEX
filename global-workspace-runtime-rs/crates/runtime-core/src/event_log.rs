use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use thiserror::Error;

use crate::event::RuntimeEvent;

#[derive(Debug, Error)]
pub enum EventLogError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Append-only JSONL event log.  Writes to a `.gwlog` file if a path is given.
#[derive(Debug, Default, Clone)]
pub struct EventLog {
    events: Vec<RuntimeEvent>,
    path: Option<PathBuf>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            events: Vec::new(),
            path: Some(path),
        }
    }

    /// Append one event.  If a path is set, also writes it to the file.
    pub fn append(&mut self, event: RuntimeEvent) -> Result<(), EventLogError> {
        if let Some(ref p) = self.path {
            let line = serde_json::to_string(&event)?;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)?;
            writeln!(f, "{}", line)?;
        }
        self.events.push(event);
        Ok(())
    }

    pub fn events(&self) -> &[RuntimeEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Serialize entire log to JSONL string.
    pub fn to_jsonl(&self) -> Result<String, EventLogError> {
        let mut out = String::new();
        for ev in &self.events {
            out.push_str(&serde_json::to_string(ev)?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Parse a JSONL string back into an EventLog (no path set).
    pub fn from_jsonl(s: &str) -> Result<Self, EventLogError> {
        let mut log = Self::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let ev: RuntimeEvent = serde_json::from_str(line)?;
            log.events.push(ev);
        }
        Ok(log)
    }

    /// Load from a `.gwlog` or `.jsonl` file. Never reads `.mv2` files.
    pub fn load(path: &PathBuf) -> Result<Self, EventLogError> {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "mv2" {
            return Err(EventLogError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("EventLog does not read .mv2 files: {}", path.display()),
            )));
        }
        let f = std::fs::File::open(path)?;
        let rdr = BufReader::new(f);
        let mut log = Self::new();
        for line in rdr.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let ev: RuntimeEvent = serde_json::from_str(line)?;
            log.events.push(ev);
        }
        Ok(log)
    }
}
