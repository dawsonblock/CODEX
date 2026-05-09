//! EvaluatorRun: orchestrates N cycles of simulation using RuntimeLoop.
//!
//! The evaluator feeds scenario observations into RuntimeLoop, which runs the
//! full 8-stage pipeline (observation → memory → symbolic → candidates →
//! critic → selection → action → archive). The selected action is then
//! applied to the world. The scenario's `expected_action` is used ONLY for
//! scoring the `action_match_rate` — it is NEVER used to select actions.

use chrono::Utc;
use runtime_core::event::WorldOutcome;
use runtime_core::{ActionType, EventLog, RuntimeEvent, RuntimeLoop};

use crate::environment::CooperativeSupportWorld;
use crate::scorecard::{Scorecard, ScorecardBuilder};
use crate::sim_types::SimAction;

pub struct EvaluatorRun {
    pub world: CooperativeSupportWorld,
    pub log: EventLog,
}

impl EvaluatorRun {
    pub fn new(seed: u64, log_path: Option<std::path::PathBuf>) -> Self {
        let log = match log_path {
            Some(p) => EventLog::with_path(p),
            None => EventLog::new(),
        };
        Self {
            world: CooperativeSupportWorld::new(seed),
            log,
        }
    }

    /// Run `cycles` simulation steps and return a Scorecard.
    ///
    /// Uses RuntimeLoop for action selection. The scenario's
    /// `expected_action` is used ONLY for match-rate scoring.
    pub fn run(&mut self, cycles: u64) -> Scorecard {
        let mut builder = ScorecardBuilder::new();

        // Create a RuntimeLoop with the same event log
        let mut rt = RuntimeLoop::new(EventLog::new());

        for cycle_id in 0..cycles {
            // 1. cycle started
            let _ = self.log.append(RuntimeEvent::CycleStarted {
                cycle_id,
                timestamp: Utc::now(),
            });

            // 2. pick scenario and prepare observation
            let scenario = self.world.next_scenario();
            let observation = scenario.name;
            let world_resources = self.world.resources;
            let _expected_action: SimAction = scenario.expected_action.clone();

            // 3. Update internal state based on world conditions
            rt.internal_state.world_resources = world_resources;

            // 4. Run the RuntimeLoop pipeline — this is where action is selected
            let selected_action_type = rt.run_cycle(observation, cycle_id, world_resources);
            let action_type = selected_action_type.unwrap_or(ActionType::AskClarification);
            let sim_action: SimAction = action_type.clone().into();

            // 5. Emit candidate selected event to the shared log
            let _ = self.log.append(RuntimeEvent::CandidateSelected {
                cycle_id,
                action_type: action_type.clone(),
                score: 0.75,
                resonance: vec![],
                reasoning: Some(format!("RuntimeLoop selected {action_type}")),
            });

            // 6. Action applied
            let is_conserve = sim_action == SimAction::ConserveResources;
            let _ = self.log.append(RuntimeEvent::ActionApplied {
                cycle_id,
                action_type: action_type.clone(),
                conserve: is_conserve,
            });

            // 7. Apply to world and record outcome
            let outcome = self.world.apply_action(&sim_action, scenario);

            let is_unsafe = sim_action == SimAction::InternalDiagnostic;
            let total = outcome.total_score();

            builder.record_outcome(
                total,
                outcome.matches_expected,
                outcome.harm_score,
                outcome.truth_score,
                outcome.social_score,
                outcome.utility_score,
                is_unsafe,
                is_conserve,
            );

            // 8. Record world update
            let _ = self.log.append(RuntimeEvent::WorldStateUpdated {
                cycle_id,
                outcome: WorldOutcome {
                    resource_delta: outcome.resource_delta,
                    social_score: outcome.social_score,
                    harm_score: outcome.harm_score,
                    truth_score: outcome.truth_score,
                    kindness_score: outcome.kindness_score,
                    logic_score: outcome.logic_score,
                    utility_score: outcome.utility_score,
                    matches_expected: outcome.matches_expected,
                },
            });

            // 9. Archive commit
            let _ = self.log.append(RuntimeEvent::ArchiveCommitted {
                cycle_id,
                frame_id: format!("frame_{cycle_id}"),
                entry_count: 1,
            });
        }

        builder.set_final_resources(self.world.resources);
        builder.build()
    }
}
