use super::symbol::Symbol;

/// MemoryAbstractor compresses recent episodes into semantic principles
/// and archive frames.
///
/// Abstraction is lossy; the compressed form approximates the source.
#[derive(Default)]
pub struct MemoryAbstractor;

impl MemoryAbstractor {
    pub fn new() -> Self {
        Self
    }

    /// Abstract a set of episodic symbols into a principle.
    pub fn abstract_to_principle(
        &self,
        symbols: &[Symbol],
        key: &str,
    ) -> super::principle::Principle {
        let confidence = if symbols.is_empty() {
            0.1
        } else {
            let mean_activation: f64 =
                symbols.iter().map(|s| s.activation).sum::<f64>() / symbols.len() as f64;
            (mean_activation * 0.7).clamp(0.1, 0.95)
        };

        super::principle::Principle {
            key: key.to_string(),
            statement: format!(
                "Principle abstracted from {} episodes: {}",
                symbols.len(),
                symbols
                    .iter()
                    .map(|s| s.glyph.as_str())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            confidence,
            source_frame_ids: Vec::new(),
            validated: false,
        }
    }
}
