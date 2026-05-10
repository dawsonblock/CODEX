use serde::{Deserialize, Serialize};

/// A principle extracted from memory episodes.
/// Principles are speculative until validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub key: String,
    pub statement: String,
    pub confidence: f64,
    pub source_frame_ids: Vec<String>,
    pub validated: bool,
}

impl Principle {
    pub fn new(key: impl Into<String>, statement: impl Into<String>, confidence: f64) -> Self {
        Self {
            key: key.into(),
            statement: statement.into(),
            confidence,
            source_frame_ids: Vec::new(),
            validated: false,
        }
    }

    /// Mark as validated.
    pub fn validate(&mut self) {
        self.validated = true;
    }

    /// Principles are speculative unless validated by the critic.
    pub fn is_authoritative(&self) -> bool {
        self.validated && self.confidence > 0.5
    }
}
