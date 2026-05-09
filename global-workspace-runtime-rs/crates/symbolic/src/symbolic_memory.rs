use serde::{Deserialize, Serialize};

use super::symbol::{Symbol, SymbolId, SymbolKind};
use super::symbol_graph::SymbolGraph;
use super::principle::Principle;

/// Tags for classifying symbolic memory entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolicMemoryTag {
    Episodic,
    Semantic,
    Principle,
    Blend,
    Trace,
    Glyph,
}

/// A symbolic memory store backed by an in-memory graph.
///
/// Does NOT make sentience claims. The symbolic system is internal
/// abstraction machinery — symbols represent concepts, not experiences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolicMemory {
    pub graph: SymbolGraph,
    pub principles: Vec<Principle>,
}

impl SymbolicMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_episodic_symbol(&mut self, glyph: &str, tag: SymbolicMemoryTag) -> SymbolId {
        let id = SymbolId(format!("sym_{:x}", hash(glyph)));
        let kind = match tag {
            SymbolicMemoryTag::Episodic => SymbolKind::Concept,
            SymbolicMemoryTag::Semantic => SymbolKind::Constraint,
            SymbolicMemoryTag::Principle => SymbolKind::Principle,
            SymbolicMemoryTag::Blend => SymbolKind::Blend,
            SymbolicMemoryTag::Trace => SymbolKind::Relation,
            SymbolicMemoryTag::Glyph => SymbolKind::Glyph,
        };
        let symbol = Symbol {
            id: id.clone(),
            kind,
            glyph: glyph.to_string(),
            activation: 0.5,
            validated: false,
            metadata: serde_json::json!({"tag": tag}),
        };
        self.graph.add_symbol(symbol);
        id
    }

    pub fn push_principle(&mut self, principle: Principle) {
        self.principles.push(principle);
    }

    pub fn principles(&self) -> &[Principle] {
        &self.principles
    }
}

fn hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
