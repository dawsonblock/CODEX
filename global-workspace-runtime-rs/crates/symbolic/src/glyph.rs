use serde::{Deserialize, Serialize};

/// A glyph is a human-readable, compact representation of a symbol's content.
///
/// ⚠ Glyphs are internal symbolic state only. They do NOT represent
/// consciousness, qualia, sentience, or subjective experience. Any language
/// in glyph content that implies otherwise is an artifact of the encoding
/// and must be ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub content: String,
    /// Whether this glyph was validated by the critic.
    pub validated: bool,
}

impl Glyph {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            validated: false,
        }
    }

    pub fn validate(&mut self) {
        self.validated = true;
    }
}
