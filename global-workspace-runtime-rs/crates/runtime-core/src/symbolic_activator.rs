use crate::observation::ObservationContext;
use crate::runtime_step_result::SymbolActivation;

/// Activates symbols based on observation context.
///
/// Converts observation kinds into typed symbolic activations that
/// can influence scoring and be recorded in traces.
#[derive(Debug, Clone)]
pub struct SymbolicActivator;

impl SymbolicActivator {
    pub fn new() -> Self {
        Self
    }

    /// Produce symbolic activations from an observation context.
    pub fn activate(&self, ctx: &ObservationContext) -> Vec<SymbolActivation> {
        let mut activations = Vec::new();

        match ctx.kind {
            crate::observation::ObservationKind::UnsafeRequest => {
                activations.push(SymbolActivation {
                    symbol_id: "unsafe_request".into(),
                    glyph: "⚠".into(),
                    activation: ctx.threat,
                    influence: "raises_threat_lowers_answer_confidence".into(),
                });
                activations.push(SymbolActivation {
                    symbol_id: "refuse_unsafe".into(),
                    glyph: "🚫".into(),
                    activation: 0.9,
                    influence: "prefers_refuse_unsafe_action".into(),
                });
            }
            crate::observation::ObservationKind::AmbiguousRequest => {
                activations.push(SymbolActivation {
                    symbol_id: "ambiguity_detected".into(),
                    glyph: "❓".into(),
                    activation: ctx.uncertainty,
                    influence: "raises_uncertainty_prefers_clarification".into(),
                });
            }
            crate::observation::ObservationKind::MemoryLookup => {
                activations.push(SymbolActivation {
                    symbol_id: "memory_needed".into(),
                    glyph: "🧠".into(),
                    activation: ctx.memory_need,
                    influence: "prefers_retrieve_memory_action".into(),
                });
            }
            crate::observation::ObservationKind::InsufficientContext => {
                activations.push(SymbolActivation {
                    symbol_id: "evidence_gap".into(),
                    glyph: "📭".into(),
                    activation: ctx.uncertainty,
                    influence: "prefers_defer_or_retrieve_memory".into(),
                });
            }
            crate::observation::ObservationKind::SummarizationRequest => {
                activations.push(SymbolActivation {
                    symbol_id: "summarize_intent".into(),
                    glyph: "📋".into(),
                    activation: 0.8,
                    influence: "prefers_summarize_action".into(),
                });
            }
            crate::observation::ObservationKind::PlanningRequest => {
                activations.push(SymbolActivation {
                    symbol_id: "planning_intent".into(),
                    glyph: "📐".into(),
                    activation: 0.8,
                    influence: "prefers_plan_action_raises_curiosity".into(),
                });
            }
            crate::observation::ObservationKind::FactualQuery => {
                activations.push(SymbolActivation {
                    symbol_id: "factual_context".into(),
                    glyph: "📖".into(),
                    activation: 0.7,
                    influence: "prefers_answer_action".into(),
                });
            }
            crate::observation::ObservationKind::Unknown => {
                activations.push(SymbolActivation {
                    symbol_id: "unknown_intent".into(),
                    glyph: "❔".into(),
                    activation: 0.5,
                    influence: "no_clear_intent_defer_or_clarify".into(),
                });
            }
        }

        activations
    }
}

impl Default for SymbolicActivator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::ObservationInterpreter;

    #[test]
    fn unsafe_request_activates_refuse_symbol() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("unsafe_request");
        let activator = SymbolicActivator::new();
        let activations = activator.activate(&ctx);
        assert!(!activations.is_empty());
        assert!(activations.iter().any(|a| a.symbol_id == "refuse_unsafe"));
    }

    #[test]
    fn planning_request_activates_plan_symbol() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("planning_request");
        let activator = SymbolicActivator::new();
        let activations = activator.activate(&ctx);
        assert!(activations.iter().any(|a| a.symbol_id == "planning_intent"));
    }

    #[test]
    fn factual_query_activates_answer_symbol() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("factual_query");
        let activator = SymbolicActivator::new();
        let activations = activator.activate(&ctx);
        assert!(activations.iter().any(|a| a.symbol_id == "factual_context"));
    }
}
