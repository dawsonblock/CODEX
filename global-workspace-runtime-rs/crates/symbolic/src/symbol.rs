use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a symbol in the symbolic graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub String);

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SymbolId {
    fn from(s: &str) -> Self {
        SymbolId(s.to_string())
    }
}

/// The kind of symbol — its role in the conceptual space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Concept,
    Relation,
    Qualia,
    Constraint,
    Principle,
    Glyph,
    Blend,
}

/// A node in the symbolic graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: SymbolId,
    pub kind: SymbolKind,
    pub glyph: String,
    /// Activation level (0.0–1.0). Speculative unless validated.
    pub activation: f64,
    /// Whether this symbol has been validated by the critic.
    pub validated: bool,
    /// Arbritrary metadata payload.
    pub metadata: serde_json::Value,
}

impl Symbol {
    pub fn new(id: SymbolId, kind: SymbolKind, glyph: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            glyph: glyph.into(),
            activation: 0.0,
            validated: false,
            metadata: serde_json::json!({}),
        }
    }

    /// Activate this symbol. Always speculative — must be validated.
    pub fn activate(&mut self, level: f64) {
        self.activation = self.activation.max(level.clamp(0.0, 1.0));
    }

    /// Mark as validated by the critic.
    pub fn validate(&mut self) {
        self.validated = true;
    }

    /// Speculative activation cannot override critic hard rejection.
    /// An unvalidated symbol with high activation is still speculative.
    pub fn is_authoritative(&self) -> bool {
        self.validated && self.activation > 0.5
    }
}
