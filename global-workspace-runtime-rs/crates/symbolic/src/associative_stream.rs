use super::symbol::{Symbol, SymbolId, SymbolKind};

/// A creative associative stream that deconstructs prior memory into
/// transferable principles.
///
/// Symbolic output is speculative unless validated.
pub struct AssociativeStream {
    pub name: String,
    pub memory_hits: Vec<String>,
}

impl AssociativeStream {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            memory_hits: Vec::new(),
        }
    }

    /// Produce a speculative symbol from a memory hit.
    /// These symbols are NOT validated — they are candidate material only.
    pub fn deconstruct(&self, memory_text: &str) -> Symbol {
        let id = SymbolId(format!("assoc_{}_{:x}", self.name, seahash(memory_text)));
        Symbol {
            id,
            kind: SymbolKind::Concept,
            glyph: memory_text.chars().take(64).collect(),
            activation: 0.3,
            validated: false,
            metadata: serde_json::json!({"stream": self.name}),
        }
    }
}

fn seahash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
