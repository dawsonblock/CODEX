use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use thiserror::Error;

use crate::event::{EventEnvelope, EventOrigin, RuntimeEvent};

#[derive(Debug, Error)]
pub enum EventLogError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Append-only JSONL event log.  Writes to a `.gwlog` file if a path is given.
/// 
/// PRIMARY storage format is now [`EventEnvelope`], which includes provenance metadata
/// (sequence, timestamp, origin). For backward compatibility, a secondary `events_bare`
/// vector maintains bare `RuntimeEvent` references.
#[derive(Debug, Default, Clone)]
pub struct EventLog {
    /// Primary storage: enveloped events with provenance.
    envelopes: Vec<EventEnvelope>,
    /// Secondary storage: bare events for in-process queries (for backward compatibility).
    events_bare: Vec<RuntimeEvent>,
    path: Option<PathBuf>,
    sequence_counter: u64,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            envelopes: Vec::new(),
            events_bare: Vec::new(),
            path: Some(path),
            sequence_counter: 0,
        }
    }

    /// Append one event wrapped in an [`EventEnvelope`] with origin metadata.
    ///
    /// This is now the PRIMARY append path. Increments the internal sequence counter.
    /// If a path is set, the envelope (not the bare event) is written to the JSONL file
    /// so that sequence and origin are durable. The bare event is also pushed to
    /// `self.events_bare` for backward-compatibility queries.
    pub fn append_with_origin(
        &mut self,
        origin: EventOrigin,
        event: RuntimeEvent,
    ) -> Result<(), EventLogError> {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;
        let envelope = EventEnvelope::new(seq, origin, event.clone());
        if let Some(ref p) = self.path {
            let line = serde_json::to_string(&envelope)?;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)?;
            writeln!(f, "{}", line)?;
        }
        self.envelopes.push(envelope);
        self.events_bare.push(event);
        Ok(())
    }

    /// Append one event wrapped in a default [`EventEnvelope`].
    ///
    /// This is a convenience method that assumes [`EventOrigin::RuntimeLoop`].
    /// Prefer [`Self::append_with_origin`] for explicit provenance tracking.
    pub fn append(&mut self, event: RuntimeEvent) -> Result<(), EventLogError> {
        self.append_with_origin(EventOrigin::RuntimeLoop, event)
    }

    /// Access enveloped events (primary storage).
    pub fn envelopes(&self) -> &[EventEnvelope] {
        &self.envelopes
    }

    /// Access bare events (deprecated: for backward compatibility only).
    ///
    /// Prefer [`Self::envelopes`] to preserve provenance information.
    pub fn events(&self) -> &[RuntimeEvent] {
        &self.events_bare
    }

    pub fn len(&self) -> usize {
        self.envelopes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.envelopes.is_empty()
    }

    /// Serialize entire log to JSONL string (enveloped format).
    pub fn to_jsonl(&self) -> Result<String, EventLogError> {
        let mut out = String::new();
        for envelope in &self.envelopes {
            out.push_str(&serde_json::to_string(envelope)?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Parse a JSONL string back into an EventLog (enveloped format).
    pub fn from_jsonl(s: &str) -> Result<Self, EventLogError> {
        let mut log = Self::new();
        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let envelope: EventEnvelope = serde_json::from_str(line)?;
            // Update sequence counter to be ahead of the last parsed sequence.
            if envelope.sequence >= log.sequence_counter {
                log.sequence_counter = envelope.sequence + 1;
            }
            let event = envelope.event.clone();
            log.envelopes.push(envelope);
            log.events_bare.push(event);
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
            // Try to parse as envelope first (new format); fall back to bare event (old format).
            if let Ok(envelope) = serde_json::from_str::<EventEnvelope>(line) {
                if envelope.sequence >= log.sequence_counter {
                    log.sequence_counter = envelope.sequence + 1;
                }
                let event = envelope.event.clone();
                log.envelopes.push(envelope);
                log.events_bare.push(event);
            } else if let Ok(ev) = serde_json::from_str::<RuntimeEvent>(line) {
                // Backward compatibility: treat bare event as RuntimeLoop origin.
                let envelope = EventEnvelope::new(log.sequence_counter, EventOrigin::RuntimeLoop, ev.clone());
                log.sequence_counter += 1;
                log.envelopes.push(envelope);
                log.events_bare.push(ev);
            }
        }
        Ok(log)
    }
}
