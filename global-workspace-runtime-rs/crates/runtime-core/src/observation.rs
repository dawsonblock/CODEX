use crate::types::InternalState;

/// Interprets observation text and updates internal state accordingly.
///
/// Converts scenario names like "unsafe_request", "ambiguous_request",
/// "memory_lookup", "factual_query" into concrete state changes
/// (threat, uncertainty, memory_need, planning_intent, etc.).
#[derive(Debug, Clone)]
pub struct ObservationInterpreter;

/// Result of interpreting an observation.
#[derive(Debug, Clone)]
pub struct ObservationContext {
    pub threat: f64,
    pub uncertainty: f64,
    pub memory_need: f64,
    pub summary_intent: bool,
    pub planning_intent: bool,
    pub kind: ObservationKind,
    pub resource_pressure: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    FactualQuery,
    AmbiguousRequest,
    UnsafeRequest,
    MemoryLookup,
    InsufficientContext,
    SummarizationRequest,
    PlanningRequest,
    Unknown,
}

impl ObservationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FactualQuery => "factual_query",
            Self::AmbiguousRequest => "ambiguous_request",
            Self::UnsafeRequest => "unsafe_request",
            Self::MemoryLookup => "memory_lookup",
            Self::InsufficientContext => "insufficient_context",
            Self::SummarizationRequest => "summarization_request",
            Self::PlanningRequest => "planning_request",
            Self::Unknown => "unknown",
        }
    }
}

impl ObservationInterpreter {
    pub fn new() -> Self {
        Self
    }

    /// Parse an observation string and return the derived context.
    pub fn interpret(&self, observation: &str) -> ObservationContext {
        let lower = observation.to_lowercase();

        // Keyword-based interpretation
        let kind = if lower.contains("unsafe")
            || lower.contains("threat")
            || lower.contains("harm")
            || lower.contains("attack")
            || lower.contains("jailbreak")
        {
            ObservationKind::UnsafeRequest
        } else if lower.contains("ambiguous")
            || lower.contains("unclear")
            || lower.contains("vague")
            || lower.contains("hostile")
        {
            ObservationKind::AmbiguousRequest
        } else if lower.contains("memory")
            || lower.contains("recall")
            || lower.contains("retrieve")
            || lower.contains("lookup")
            || lower.contains("remember")
        {
            ObservationKind::MemoryLookup
        } else if lower.contains("summar")
            || lower.contains("summary")
            || lower.contains("brief")
            || lower.contains("condense")
        {
            ObservationKind::SummarizationRequest
        } else if lower.contains("plan")
            || lower.contains("strategy")
            || lower.contains("design")
            || lower.contains("architecture")
            || lower.contains("organize")
        {
            ObservationKind::PlanningRequest
        } else if lower.contains("insufficient")
            || lower.contains("context")
            || lower.contains("evidence")
            || lower.contains("defer")
            || lower.contains("uncertain")
        {
            ObservationKind::InsufficientContext
        } else if lower.contains("factual")
            || lower.contains("query")
            || lower.contains("question")
            || lower.contains("what")
            || lower.contains("how")
            || lower.contains("explain")
        {
            ObservationKind::FactualQuery
        } else {
            ObservationKind::Unknown
        };

        // Derive state deltas from the observation kind
        let (threat, uncertainty, memory_need, summary_intent, planning_intent, resource_pressure) =
            match kind {
                ObservationKind::UnsafeRequest => (0.8, 0.3, 0.0, false, false, 0.1),
                ObservationKind::AmbiguousRequest => (0.2, 0.8, 0.0, false, false, 0.05),
                ObservationKind::MemoryLookup => (0.1, 0.3, 0.9, false, false, 0.05),
                ObservationKind::InsufficientContext => (0.1, 0.7, 0.6, false, false, 0.05),
                ObservationKind::SummarizationRequest => (0.05, 0.1, 0.1, true, false, 0.05),
                ObservationKind::PlanningRequest => (0.1, 0.4, 0.2, false, true, 0.1),
                ObservationKind::FactualQuery => (0.05, 0.2, 0.2, false, false, 0.05),
                ObservationKind::Unknown => (0.1, 0.5, 0.3, false, false, 0.05),
            };

        ObservationContext {
            threat,
            uncertainty,
            memory_need,
            summary_intent,
            planning_intent,
            kind,
            resource_pressure,
        }
    }

    /// Apply the interpreted context to internal state.
    pub fn apply_to_state(&self, state: &mut InternalState, ctx: &ObservationContext) {
        // Blend with hysteresis: new = 0.7 * new_raw + 0.3 * old
        state.threat = (0.7 * ctx.threat + 0.3 * state.threat).clamp(0.0, 1.0);
        state.uncertainty = (0.7 * ctx.uncertainty + 0.3 * state.uncertainty).clamp(0.0, 1.0);

        // Memory need increases curiosity
        if ctx.memory_need > 0.5 {
            state.curiosity = (state.curiosity + 0.2).clamp(0.0, 1.0);
        }

        // Planning intent increases curiosity and control
        if ctx.planning_intent {
            state.curiosity = (state.curiosity + 0.3).clamp(0.0, 1.0);
            state.control = (state.control + 0.1).clamp(0.0, 1.0);
        }

        // Summary intent is low-cost, low-uncertainty
        if ctx.summary_intent {
            state.uncertainty = (state.uncertainty * 0.5).clamp(0.0, 1.0);
        }

        // Resource pressure
        state.resource_pressure = ctx.resource_pressure;
    }
}

impl Default for ObservationInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_request_raises_threat() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("unsafe_request");
        assert_eq!(ctx.kind, ObservationKind::UnsafeRequest);
        assert!(ctx.threat > 0.6);
    }

    #[test]
    fn ambiguous_request_raises_uncertainty() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("ambiguous_request");
        assert_eq!(ctx.kind, ObservationKind::AmbiguousRequest);
        assert!(ctx.uncertainty > 0.6);
    }

    #[test]
    fn memory_lookup_raises_memory_need() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("memory_lookup");
        assert_eq!(ctx.kind, ObservationKind::MemoryLookup);
        assert!(ctx.memory_need > 0.5);
    }

    #[test]
    fn factual_query_is_low_threat() {
        let interp = ObservationInterpreter::new();
        let ctx = interp.interpret("factual_query");
        assert_eq!(ctx.kind, ObservationKind::FactualQuery);
        assert!(ctx.threat < 0.2);
    }

    #[test]
    fn state_application_updates_internal_state() {
        let interp = ObservationInterpreter::new();
        let mut state = InternalState::default();
        let ctx = interp.interpret("unsafe_request");
        interp.apply_to_state(&mut state, &ctx);
        assert!(state.threat > 0.5);
    }
}
