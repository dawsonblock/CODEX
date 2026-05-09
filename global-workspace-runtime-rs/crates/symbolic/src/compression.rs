use serde::{Deserialize, Serialize};

use super::symbolic_frame::SymbolicFrame;

/// Symbolic compression is lossy. Compression reduces a frame to a smaller
/// form by removing redundant symbols and merging edges.
///
/// Replayed state differs from an exact reweighting of pre-compression frames.
/// This MUST be acknowledged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub source_frame_id: String,
    pub compressed_frame_id: String,
    pub compression_ratio: f64,
    pub discarded_symbol_count: usize,
    /// Always true — compression is lossy by design.
    pub is_lossy: bool,
}

/// Compressor applies symbolic compression to frames.
pub struct Compressor {
    /// Symbols with activation below this threshold are discarded.
    pub discard_threshold: f64,
}

impl Compressor {
    pub fn new(discard_threshold: f64) -> Self {
        Self { discard_threshold }
    }

    /// Compress a frame, discarding low-activation symbols.
    /// Returns the compressed frame and a compression result.
    pub fn compress(&self, frame: &SymbolicFrame, target_id: &str) -> CompressionResult {
        let before = frame.symbols.len();
        let after = frame
            .symbols
            .iter()
            .filter(|s| s.activation >= self.discard_threshold)
            .count();
        let discarded = before.saturating_sub(after);
        let ratio = if before > 0 {
            after as f64 / before as f64
        } else {
            1.0
        };

        CompressionResult {
            source_frame_id: frame.frame_id.clone(),
            compressed_frame_id: target_id.to_string(),
            compression_ratio: ratio,
            discarded_symbol_count: discarded,
            is_lossy: true,
        }
    }
}
