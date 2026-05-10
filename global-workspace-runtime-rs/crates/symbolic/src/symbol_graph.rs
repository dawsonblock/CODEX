use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::symbol::{Symbol, SymbolId};

/// Edge between two symbols in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub source_id: SymbolId,
    pub target_id: SymbolId,
    pub kind: String,
    pub weight: f64,
}

/// A directed graph of symbols and their relationships.
///
/// Symbolic output is speculative unless validated.
/// Symbolic resonance cannot override critic hard rejection.
/// Symbolic/glyph state cannot create sentience claims.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolGraph {
    symbols: HashMap<SymbolId, Symbol>,
    edges: Vec<SymbolEdge>,
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.id.clone(), symbol);
    }

    pub fn get_symbol(&self, id: &SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn get_symbol_mut(&mut self, id: &SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id)
    }

    pub fn add_edge(&mut self, edge: SymbolEdge) {
        self.edges.push(edge);
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn edges(&self) -> &[SymbolEdge] {
        &self.edges
    }

    /// Activate a symbol by id. Returns false if not found.
    pub fn activate(&mut self, id: &SymbolId, level: f64) -> bool {
        if let Some(sym) = self.symbols.get_mut(id) {
            sym.activate(level);
            true
        } else {
            false
        }
    }

    /// Validate a symbol by id. Returns false if not found.
    pub fn validate(&mut self, id: &SymbolId) -> bool {
        if let Some(sym) = self.symbols.get_mut(id) {
            sym.validate();
            true
        } else {
            false
        }
    }

    /// All validated symbols with activation above threshold.
    pub fn authoritative_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.is_authoritative())
            .collect()
    }

    /// Create a snapshot of the graph for serialization.
    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            symbols: self.symbols.values().cloned().collect(),
            edges: self.edges.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub symbols: Vec<Symbol>,
    pub edges: Vec<SymbolEdge>,
}
