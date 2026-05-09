use super::symbol::{Symbol, SymbolId, SymbolKind};

/// The analytic stream produces grounded, logical candidate symbols.
pub struct AnalyticStream;

impl AnalyticStream {
    pub fn new() -> Self {
        Self
    }

    /// Produce an analytic symbol from observation text.
    /// Speculative until validated.
    pub fn analyze(&self, observation: &str, cycle_id: u64) -> Symbol {
        let id = SymbolId(format!("analytic_{cycle_id}"));
        Symbol {
            id,
            kind: SymbolKind::Constraint,
            glyph: observation.chars().take(64).collect(),
            activation: 0.5,
            validated: false,
            metadata: serde_json::json!({"stream": "analytic", "cycle": cycle_id}),
        }
    }
}
