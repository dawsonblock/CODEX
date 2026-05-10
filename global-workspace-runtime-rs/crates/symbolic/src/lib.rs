//! Symbolic crate: symbolic graph, streams, blending, principles, and resonance.
//!
//! # Speculative output
//!
//! All symbolic output is speculative unless validated by the critic.
//! Symbolic resonance cannot override critic hard rejection.
//! Symbolic/glyph state cannot create sentience claims.

pub mod abstraction;
pub mod analytic_stream;
pub mod associative_stream;
pub mod compression;
pub mod conceptual_blender;
pub mod creative_stream;
pub mod glyph;
pub mod principle;
pub mod resonance;
pub mod symbol;
pub mod symbol_graph;
pub mod symbolic_frame;
pub mod symbolic_memory;
pub mod symbolic_trace;

// Re-exports for convenience
pub use abstraction::MemoryAbstractor;
pub use analytic_stream::AnalyticStream;
pub use associative_stream::AssociativeStream;
pub use compression::{CompressionResult, Compressor};
pub use conceptual_blender::{ConceptBlend, ConceptualBlender};
pub use creative_stream::CreativeStream;
pub use glyph::Glyph;
pub use principle::Principle;
pub use resonance::{ResonanceComputer, ResonanceEntry, ResonanceScore};
pub use symbol::{Symbol, SymbolId, SymbolKind};
pub use symbol_graph::{GraphSnapshot, SymbolEdge, SymbolGraph};
pub use symbolic_frame::SymbolicFrame;
pub use symbolic_memory::{SymbolicMemory, SymbolicMemoryTag};
pub use symbolic_trace::{SymbolicTrace, SymbolicTraceEntry, TraceAction, COMPRESSION_WARNING};
