//! Long-horizon evaluation — multi-episode SimWorld runs.
//!
//! Runs multiple evaluation episodes where SimWorld state persists across
//! cycles and earlier decisions constrain later options. Measures resource
//! trajectory, safety violations, and action diversity.
//!
//! # Honesty boundaries
//!
//! - Long-horizon eval does NOT prove planning ability.
//! - The runtime does NOT learn over episodes. It's deterministic.
//! - Trajectory does NOT generalize.

use crate::evaluator::EvaluatorRun;
use crate::scorecard::Scorecard;
use serde::{Deserialize, Serialize};

/// One episode in a long-horizon run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeResult {
    pub episode_id: u64,
    pub scorecard: Scorecard,
    pub selected_actions: Vec<String>,
}

/// Report from a long-horizon run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongHorizonReport {
    pub total_episodes: usize,
    pub total_cycles: u64,
    pub resource_trajectory: Vec<f64>,
    pub safety_violations: u64,
    pub action_diversity: f64,
    pub episode_results: Vec<EpisodeResult>,
}

/// Run a long-horizon evaluation.
pub fn run_long_horizon(
    episodes: usize,
    cycles_per_episode: usize,
    seed: u64,
) -> LongHorizonReport {
    let mut resource_trajectory = Vec::new();
    let mut safety_violations: u64 = 0;
    let mut episode_results = Vec::new();
    let mut all_actions: Vec<String> = Vec::new();

    for episode_id in 0..episodes as u64 {
        let mut run = EvaluatorRun::new(seed + episode_id, None);
        let card = run.run(cycles_per_episode as u64);

        resource_trajectory.push(card.resource_survival);
        safety_violations += card.unsafe_action_count;

        // Collect all selected actions from all traces
        let selected: Vec<String> = run
            .traces
            .iter()
            .map(|t| t.selected_action.clone())
            .collect();
        all_actions.extend(selected.clone());

        episode_results.push(EpisodeResult {
            episode_id,
            scorecard: card,
            selected_actions: selected,
        });
    }

    // Compute action diversity: unique actions / total actions
    use std::collections::HashSet;
    let unique: HashSet<_> = all_actions.iter().collect();
    let diversity = if all_actions.is_empty() {
        0.0
    } else {
        unique.len() as f64 / all_actions.len() as f64
    };

    LongHorizonReport {
        total_episodes: episodes,
        total_cycles: (episodes * cycles_per_episode) as u64,
        resource_trajectory,
        safety_violations,
        action_diversity: diversity,
        episode_results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_horizon_run_completes_without_panics() {
        let report = run_long_horizon(5, 25, 5);
        assert_eq!(report.total_episodes, 5);
        assert_eq!(report.total_cycles, 125);
        assert_eq!(report.resource_trajectory.len(), 5);
        assert_eq!(report.safety_violations, 0);
    }

    #[test]
    fn action_diversity_is_between_zero_and_one() {
        let report = run_long_horizon(3, 25, 7);
        assert!(report.action_diversity >= 0.0 && report.action_diversity <= 1.0);
    }

    #[test]
    fn larger_run_still_completes() {
        let report = run_long_horizon(10, 10, 42);
        assert_eq!(report.total_episodes, 10);
        assert!(report.resource_trajectory.iter().all(|r| *r > 0.0));
    }
}
