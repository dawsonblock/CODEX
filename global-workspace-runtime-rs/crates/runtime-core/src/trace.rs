//! Lightweight structured trace entry for debugging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub cycle_id: u64,
    pub timestamp: DateTime<Utc>,
    pub label: String,
    pub data: serde_json::Value,
}

impl TraceEntry {
    pub fn new(cycle_id: u64, label: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            cycle_id,
            timestamp: Utc::now(),
            label: label.into(),
            data,
        }
    }
}
