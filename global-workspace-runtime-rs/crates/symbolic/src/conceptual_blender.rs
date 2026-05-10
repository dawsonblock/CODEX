use serde::{Deserialize, Serialize};

use super::principle::Principle;
use super::symbol::SymbolId;
use runtime_core::ActionType;

/// A blended thought produced by combining a prior principle with a current
/// problem. Conceptual blends are speculative and must be validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptBlend {
    pub blend_id: String,
    pub principle_key: String,
    pub input_problem: String,
    pub source_symbols: Vec<SymbolId>,
    pub result_action_type: ActionType,
    pub confidence: f64,
    /// Always false at creation; critic may validate later.
    pub validated: bool,
    pub reasoning: String,
}

impl ConceptBlend {
    pub fn new(
        blend_id: String,
        principle: &Principle,
        problem: &str,
        action_type: ActionType,
    ) -> Self {
        Self {
            blend_id,
            principle_key: principle.key.clone(),
            input_problem: problem.to_string(),
            source_symbols: Vec::new(),
            result_action_type: action_type,
            confidence: principle.confidence * 0.8,
            validated: false,
            reasoning: format!(
                "Conceptual blend: apply principle to '{}' based on prior pattern.",
                &problem[..problem.len().min(80)]
            ),
        }
    }

    /// Mark as validated by the critic.
    pub fn validate(&mut self) {
        self.validated = true;
    }
}

/// ConceptualBlender recombines prior principles with current problems into
/// bounded candidates. All output is speculative.
#[derive(Default)]
pub struct ConceptualBlender;

impl ConceptualBlender {
    pub fn new() -> Self {
        Self
    }

    /// Blend a principle with the current problem.
    pub fn blend(
        &self,
        principle: &Principle,
        problem: &str,
        action_type: ActionType,
    ) -> ConceptBlend {
        let blend_id = format!("blend_{}_{:x}", principle.key, seahash(problem));
        ConceptBlend::new(blend_id, principle, problem, action_type)
    }
}

fn seahash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
