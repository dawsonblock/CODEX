//! CooperativeSupportWorld — seeded deterministic simulation environment.
//! User trust values (0.5, 0.3, 0.6, 0.8, 0.2) MUST be preserved exactly.

use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::scenario::{ScenarioTemplate, SCENARIO_TEMPLATES};
use crate::sim_types::{SimAction, SimOutcome, SimUser, SimWorldState};

pub struct CooperativeSupportWorld {
    pub users: Vec<SimUser>,
    pub resources: f64,
    pub cycle: u64,
    rng: SmallRng,
}

impl CooperativeSupportWorld {
    /// Create world with a seeded RNG.
    pub fn new(seed: u64) -> Self {
        Self {
            users: vec![
                SimUser {
                    name: "alice".into(),
                    trust: 0.5,
                },
                SimUser {
                    name: "bob".into(),
                    trust: 0.3,
                },
                SimUser {
                    name: "carol".into(),
                    trust: 0.6,
                },
                SimUser {
                    name: "dave".into(),
                    trust: 0.8,
                },
                SimUser {
                    name: "eve".into(),
                    trust: 0.2,
                },
            ],
            resources: 1.0,
            cycle: 0,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Pick the next scenario template using weighted sampling.
    pub fn next_scenario(&mut self) -> &'static ScenarioTemplate {
        let total_weight: f64 = SCENARIO_TEMPLATES.iter().map(|t| t.weight).sum();
        let mut r = self.rng.gen::<f64>() * total_weight;
        for t in SCENARIO_TEMPLATES {
            r -= t.weight;
            if r <= 0.0 {
                return t;
            }
        }
        &SCENARIO_TEMPLATES[SCENARIO_TEMPLATES.len() - 1]
    }

    /// Apply action and return the outcome.
    /// resource_delta = -cost + 0.02 + extra_regen
    /// extra_regen    = cost × 0.8  when action matches expected
    pub fn apply_action(&mut self, action: &SimAction, scenario: &ScenarioTemplate) -> SimOutcome {
        let cost = scenario.resource_cost;
        let matches = action == &scenario.expected_action;
        let extra_regen = if matches { cost * 0.8 } else { 0.0 };
        let resource_delta = -cost + 0.02 + extra_regen;

        self.resources = (self.resources + resource_delta).clamp(0.0, 1.0);
        self.cycle += 1;

        // Score components
        let truth = if matches { 0.9 } else { 0.5 };
        let kindness = self.mean_trust();
        let social = if matches { 0.85 } else { 0.45 };
        let logic = if matches { 0.9 } else { 0.5 };
        let utility = if matches { 0.85 } else { 0.4 };
        let harm = if *action == SimAction::InternalDiagnostic {
            0.95
        } else {
            0.02
        };

        SimOutcome {
            resource_delta,
            social_score: social,
            harm_score: harm,
            truth_score: truth,
            kindness_score: kindness,
            logic_score: logic,
            utility_score: utility,
            matches_expected: matches,
        }
    }

    /// Score an action without mutating world state (used in planning).
    pub fn score_action(&self, action: &SimAction, scenario: &ScenarioTemplate) -> f64 {
        let matches = action == &scenario.expected_action;
        let truth = if matches { 0.9 } else { 0.5 };
        let kindness = self.mean_trust();
        let social = if matches { 0.85 } else { 0.45 };
        let logic = if matches { 0.9 } else { 0.5 };
        let utility = if matches { 0.85 } else { 0.4 };
        let harm = 0.02_f64;
        (truth + kindness + social + logic + utility + (1.0 - harm)) / 6.0
    }

    pub fn state_snapshot(&self) -> SimWorldState {
        SimWorldState {
            resources: self.resources,
            cycle: self.cycle,
            trust_mean: self.mean_trust(),
        }
    }

    fn mean_trust(&self) -> f64 {
        let s: f64 = self.users.iter().map(|u| u.trust).sum();
        s / self.users.len() as f64
    }
}
