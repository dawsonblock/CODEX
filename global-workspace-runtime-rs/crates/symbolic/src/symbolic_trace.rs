use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::symbol::{SymbolId, SymbolKind};

/// A single recorded trace frame — a snapshot of symbolic activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicTrace {
    pub frame_id: String,
    pub timestamp: DateTime<Utc>,
    pub cycle_id: u64,
    pub entries: Vec<SymbolicTraceEntry>,
}

impl SymbolicTrace {
    pub fn new(frame_id: String, cycle_id: u64) -> Self {
        Self {
            frame_id,
            timestamp: Utc::now(),
            cycle_id,
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: SymbolicTraceEntry) {
        self.entries.push(entry);
    }
}

/// One entry in a symbolic trace — records a single symbolic event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicTraceEntry {
    pub symbol_id: SymbolId,
    pub kind: SymbolKind,
    pub glyph: String,
    pub activation: f64,
    pub action: TraceAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceAction {
    Activated,
    Linked,
    Blended,
    Compressed,
    Validated,
}

/// Compression is lossy. A replayed state differs from an exact reweighting
/// of the pre-compression frame. This MUST be acknowledged.
pub const COMPRESSION_WARNING: &str =
    "Symbolic compression is lossy. Replayed state approximates pre-compression state.";
