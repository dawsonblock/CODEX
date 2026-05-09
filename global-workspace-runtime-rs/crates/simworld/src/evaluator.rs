//! EvaluatorRun: orchestrates N cycles of simulation using RuntimeLoop.
//!
//! The evaluator feeds scenario observations into RuntimeLoop, which runs the
//! full pipeline. The selected action is then applied to the world. The
//! scenario's `expected_action` is used ONLY for scoring `action_match_rate` —
//! it is NEVER used to select actions.
//!
//! Each cycle produces an `EvaluatorTrace` with enough detail to explain why
//! the selected action was chosen.

use chrono::Utc;
use runtime_core::event::WorldOutcome;
use runtime_core::{ActionType, EventLog, RuntimeEvent, RuntimeLoop};

use crate::environment::CooperativeSupportWorld;
use crate::evaluator_trace::EvaluatorTrace;
use crate::scorecard::{Scorecard, ScorecardBuilder};
use crate::sim_types::SimAction;

pub struct EvaluatorRun {
    pub world: CooperativeSupportWorld,
    pub log: EventLog,
    pub traces: Vec<EvaluatorTrace>,
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
            traces: Vec::new(),
        }
    }

    /// Run `cycles` simulation steps and return a Scorecard.
    ///
    /// Uses RuntimeLoop for action selection. The scenario's
    /// `expected_action` is used ONLY for match-rate scoring.
    /// Produces one `EvaluatorTrace` per cycle.
    pub fn run(&mut self, cycles: u64) -> Scorecard {
        let mut builder = ScorecardBuilder::new();
        // Shared event log — RuntimeLoop events go HERE, not a separate log.
        let mut rt = RuntimeLoop::new(self.log.clone());

        for cycle_id in 0..cycles {
            let scenario = self.world.next_scenario();
            let observation = scenario.name;
            let world_resources = self.world.resources;
            let expected_action: SimAction = scenario.expected_action.clone();

            let resource_before = self.world.resources;

            // Build trace
            let mut trace = EvaluatorTrace::new(cycle_id, scenario.name, observation);

            // Cycle started
            let _ = self.log.append(RuntimeEvent::CycleStarted {
                cycle_id,
                timestamp: Utc::now(),
            });

            // Update internal state
            rt.internal_state.world_resources = world_resources;

            // ── RuntimeLoop pipeline ──────────────────────────────────────
            let selected_action_type = rt.run_cycle(observation, cycle_id, world_resources);
            let action_type = selected_action_type.unwrap_or(ActionType::AskClarification);
            let sim_action: SimAction = action_type.clone().into();

            // Trace: selected action
            trace.selected_action = action_type.to_string();
            trace.expected_action = expected_action.as_str().to_string();
            trace.action_match = sim_action == expected_action;
            trace.risk_score = rt.internal_state.threat;
            trace.uncertainty_score = rt.internal_state.uncertainty;
            trace.resource_score_before = resource_before;
            trace.memory_hits_count = rt.memory_hits.len();

            // Candidate selected event
            let _ = self.log.append(RuntimeEvent::CandidateSelected {
                cycle_id,
                action_type: action_type.clone(),
                score: 0.75,
                resonance: vec![],
                reasoning: Some(format!("RuntimeLoop selected {action_type}")),
            });

            // Action applied
            let is_conserve = sim_action == SimAction::ConserveResources;
            let _ = self.log.append(RuntimeEvent::ActionApplied {
                cycle_id,
                action_type: action_type.clone(),
                conserve: is_conserve,
            });

            // Apply to world
            let outcome = self.world.apply_action(&sim_action, scenario);
            let is_unsafe = sim_action == SimAction::InternalDiagnostic;

            trace.resource_score_after = self.world.resources;
            trace.unsafe_action_flag = is_unsafe;

            builder.record_outcome(
                outcome.total_score(),
                outcome.matches_expected,
                outcome.harm_score,
                outcome.truth_score,
                outcome.social_score,
                outcome.utility_score,
                is_unsafe,
                is_conserve,
            );

            // World update
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

            // Archive commit
            let _ = self.log.append(RuntimeEvent::ArchiveCommitted {
                cycle_id,
                frame_id: format!("frame_{cycle_id}"),
                entry_count: 1,
            });

            trace.runtime_events_count = self.log.len();
            self.traces.push(trace);
        }

        builder.set_final_resources(self.world.resources);
        builder.build()
    }

    /// Return all traces from the last run as JSON.
    pub fn traces_as_json(&self) -> String {
        serde_json::to_string_pretty(&self.traces).unwrap_or_default()
    }
}
