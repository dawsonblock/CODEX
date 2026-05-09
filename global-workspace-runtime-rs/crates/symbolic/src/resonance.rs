use serde::{Deserialize, Serialize};

use super::symbol::{Symbol, SymbolId};

/// A resonance score represents how strongly a candidate resonates with
/// the existing symbolic graph.
///
/// Resonance is speculative and advisory. It CANNOT override critic hard
/// rejection. High resonance on an unsafe candidate is still unsafe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceScore {
    pub action_type: String,
    pub entries: Vec<ResonanceEntry>,
    pub total_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceEntry {
    pub symbol_id: SymbolId,
    pub glyph: String,
    pub intensity: f64,
}

/// Compute resonance scores for candidates against the symbolic graph.
pub struct ResonanceComputer;

impl ResonanceComputer {
    pub fn new() -> Self {
        Self
    }

    /// Compute a resonance score for a set of activated symbols.
    /// Scores are ADVISORY ONLY and do not override critic decisions.
    pub fn compute(
        &self,
        activated_symbols: &[&Symbol],
        action_type: &str,
    ) -> ResonanceScore {
        let entries: Vec<ResonanceEntry> = activated_symbols
            .iter()
            .map(|s| ResonanceEntry {
                symbol_id: s.id.clone(),
                glyph: s.glyph.clone(),
                intensity: s.activation,
            })
            .collect();

        let total = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.intensity).sum::<f64>() / entries.len() as f64
        };

        ResonanceScore {
            action_type: action_type.to_string(),
            entries,
            total_score: total,
        }
    }
}
