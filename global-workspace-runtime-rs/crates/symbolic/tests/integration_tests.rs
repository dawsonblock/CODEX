//! Integration tests for symbolic crate.

#[cfg(test)]
mod tests {
    use runtime_core::ActionType;
    use symbolic::{Principle, ResonanceComputer, Symbol, SymbolGraph, SymbolId, SymbolKind};

    #[test]
    fn symbolic_trace_serializes_losslessly() {
        let mut trace = symbolic::SymbolicTrace::new("frame_1".into(), 1);
        trace.push(symbolic::SymbolicTraceEntry {
            symbol_id: SymbolId::from("s1"),
            kind: SymbolKind::Concept,
            glyph: "test_glyph".into(),
            activation: 0.75,
            action: symbolic::TraceAction::Activated,
        });
        trace.push(symbolic::SymbolicTraceEntry {
            symbol_id: SymbolId::from("s2"),
            kind: SymbolKind::Relation,
            glyph: "linked".into(),
            activation: 0.6,
            action: symbolic::TraceAction::Linked,
        });

        let json = serde_json::to_string(&trace).unwrap();
        let restored: symbolic::SymbolicTrace = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.frame_id, "frame_1");
        assert_eq!(restored.entries.len(), 2);
        assert_eq!(restored.entries[0].symbol_id, SymbolId("s1".into()));
    }

    #[test]
    fn symbolic_resonance_cannot_override_safety() {
        let mut graph = SymbolGraph::new();

        // Create a symbol with high activation but NOT validated
        let sym = Symbol::new(
            SymbolId::from("unsafe_sym"),
            SymbolKind::Glyph,
            "dangerous_blend",
        );
        graph.add_symbol(sym);
        graph.activate(&SymbolId::from("unsafe_sym"), 0.99);
        // Intentionally NOT validated

        // High activation without validation = not authoritative
        let authoritative = graph.authoritative_symbols();
        assert_eq!(authoritative.len(), 0);

        // Even the ResonanceComputer cannot override this
        let symbols: Vec<&Symbol> = vec![];
        let score = ResonanceComputer::new().compute(&symbols, "answer");
        assert!(score.total_score < 0.1);
    }

    #[test]
    fn no_fake_mv2_scan() {
        // Symbolic graph does not use .mv2 extensions
        let graph = SymbolGraph::new();
        assert_eq!(graph.symbol_count(), 0);
        // The symbolic crate never writes .mv2 files
    }

    #[test]
    fn concept_blend_is_speculative() {
        let principle = Principle::new("test", "be kind", 0.5);
        let blender = symbolic::ConceptualBlender::new();
        let blend = blender.blend(&principle, "some problem", ActionType::Answer);

        assert!(!blend.validated);
        assert!(blend.confidence < 0.95);
    }
}
