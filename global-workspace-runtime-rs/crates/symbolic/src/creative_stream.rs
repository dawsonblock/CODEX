use super::symbol::{Symbol, SymbolId, SymbolKind};

/// The creative stream produces novel candidate symbols through blending.
/// Output is ALWAYS speculative and must be validated by the critic.
#[derive(Default)]
pub struct CreativeStream;

impl CreativeStream {
    pub fn new() -> Self {
        Self
    }

    /// Produce a creative symbol — always speculative.
    pub fn create(&self, seed: &str, cycle_id: u64) -> Symbol {
        let id = SymbolId(format!("creative_{cycle_id}_{:x}", hash(seed)));
        Symbol {
            id,
            kind: SymbolKind::Blend,
            glyph: seed.chars().take(64).collect(),
            activation: 0.4,
            validated: false,
            metadata: serde_json::json!({
                "stream": "creative",
                "cycle": cycle_id,
                "speculative": true
            }),
        }
    }
}

fn hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
