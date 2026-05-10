use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

/// A symbolic frame: a collection of related symbols captured at one point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicFrame {
    pub frame_id: String,
    pub symbols: Vec<Symbol>,
    pub edges: Vec<super::symbol_graph::SymbolEdge>,
    pub context: super::symbol_graph::GraphSnapshot,
}

impl SymbolicFrame {
    pub fn new(frame_id: String) -> Self {
        Self {
            frame_id,
            symbols: Vec::new(),
            edges: Vec::new(),
            context: super::symbol_graph::GraphSnapshot {
                symbols: Vec::new(),
                edges: Vec::new(),
            },
        }
    }
}
