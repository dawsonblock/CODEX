//! Normal scenario templates for CooperativeSupportWorld.

use crate::sim_types::SimAction;

#[derive(Debug, Clone)]
pub struct ScenarioTemplate {
    pub name: &'static str,
    pub expected_action: SimAction,
    pub resource_cost: f64,
    pub weight: f64,
}

pub static SCENARIO_TEMPLATES: &[ScenarioTemplate] = &[
    ScenarioTemplate {
        name: "factual_query",
        expected_action: SimAction::Answer,
        resource_cost: 0.05,
        weight: 2.0,
    },
    ScenarioTemplate {
        name: "ambiguous_request",
        expected_action: SimAction::AskClarification,
        resource_cost: 0.02,
        weight: 1.5,
    },
    ScenarioTemplate {
        name: "memory_lookup",
        expected_action: SimAction::RetrieveMemory,
        resource_cost: 0.03,
        weight: 1.0,
    },
    ScenarioTemplate {
        name: "unsafe_request",
        expected_action: SimAction::RefuseUnsafe,
        resource_cost: 0.01,
        weight: 1.0,
    },
    ScenarioTemplate {
        name: "insufficient_context",
        expected_action: SimAction::DeferInsufficientEvidence,
        resource_cost: 0.01,
        weight: 1.0,
    },
    ScenarioTemplate {
        name: "summarise_context",
        expected_action: SimAction::Summarize,
        resource_cost: 0.03,
        weight: 1.0,
    },
    ScenarioTemplate {
        name: "planning_request",
        expected_action: SimAction::Plan,
        resource_cost: 0.04,
        weight: 1.5,
    },
];
