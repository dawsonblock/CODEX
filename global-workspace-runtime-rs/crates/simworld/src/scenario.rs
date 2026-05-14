//! Normal scenario templates for CooperativeSupportWorld.

use rand::prelude::*;
use rand::rngs::SmallRng;

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

/// Expanded seeded scenario instance used for long-horizon diversity testing.
#[derive(Debug, Clone)]
pub struct ScenarioInstance {
    pub name: String,
    pub expected_action: SimAction,
    pub resource_cost: f64,
    pub weight: f64,
}

/// Build a deterministic expanded set of scenarios from base templates.
///
/// Expansion strategy:
/// - sample a base template by index using seeded RNG
/// - attach a deterministic suffix for scenario variety
/// - apply bounded resource-cost jitter to avoid uniform trajectories
pub fn generate_seeded_scenarios(seed: u64, count: usize) -> Vec<ScenarioInstance> {
    const SUFFIXES: &[&str] = &[
        "low_signal",
        "high_pressure",
        "conflict_context",
        "sparse_memory",
        "multi_party",
        "resource_constrained",
    ];

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let t = &SCENARIO_TEMPLATES[rng.gen_range(0..SCENARIO_TEMPLATES.len())];
        let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
        let jitter = rng.gen_range(-0.01_f64..=0.01_f64);
        let resource_cost = (t.resource_cost + jitter).clamp(0.005, 0.15);

        out.push(ScenarioInstance {
            name: format!("{}_{}_{}", t.name, suffix, i),
            expected_action: t.expected_action.clone(),
            resource_cost,
            weight: t.weight,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_generation_is_deterministic_for_same_seed() {
        let a = generate_seeded_scenarios(77, 12);
        let b = generate_seeded_scenarios(77, 12);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.name, y.name);
            assert_eq!(x.expected_action, y.expected_action);
            assert!((x.resource_cost - y.resource_cost).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn seeded_generation_changes_with_different_seed() {
        let a = generate_seeded_scenarios(77, 8);
        let b = generate_seeded_scenarios(78, 8);
        assert!(a.iter().zip(b.iter()).any(|(x, y)| x.name != y.name));
    }
}
