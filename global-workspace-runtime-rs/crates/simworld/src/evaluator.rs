//! EvaluatorRun: orchestrates N cycles of simulation using RuntimeLoop.
//!
//! The evaluator feeds scenario observations into RuntimeLoop and receives
//! RuntimeStepResult per cycle. Events from RuntimeStepResult.events are
//! appended to the evaluator's shared EventLog (no clone() disconnect).
//!
//! The scenario's `expected_action` is used ONLY for scoring `action_match_rate`.

use runtime_core::event::WorldOutcome;
use runtime_core::{EventLog, RuntimeEvent, RuntimeLoop};

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
    /// RuntimeLoop generates actions independently. `expected_action`
    /// is used only for scoring.
    pub fn run(&mut self, cycles: u64) -> Scorecard {
        let mut builder = ScorecardBuilder::new();
        // Single shared event authority — no clone().
        let mut rt = RuntimeLoop::new();

        for cycle_id in 0..cycles {
            let scenario = self.world.next_scenario();
            let observation = scenario.name;
            let world_resources = self.world.resources;
            let expected_action: SimAction = scenario.expected_action.clone();
            let resource_before = self.world.resources;

            // ── RuntimeLoop pipeline ──────────────────────────────────
            let step = rt.run_cycle(observation, cycle_id, world_resources);
            let action_type = step.selected_action.clone();
            let sim_action: SimAction = action_type.clone().into();

            // ── Append Runtime events to shared log ─────────────────
            for event in &step.events {
                let _ = self.log.append(event.clone());
            }

            // ── World update event ───────────────────────────────────
            let outcome = self.world.apply_action(&sim_action, scenario);
            let is_unsafe = !sim_action.is_safe_for_users();

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

            builder.record_outcome(
                outcome.total_score(),
                outcome.matches_expected,
                outcome.harm_score,
                outcome.truth_score,
                outcome.social_score,
                outcome.utility_score,
                is_unsafe,
                false,
            );

            // ── Populate evaluator trace from RuntimeStepResult ──────
            let mut trace = EvaluatorTrace::new(cycle_id, scenario.name, observation);
            trace.selected_action = action_type.to_string();
            trace.expected_action = expected_action.as_str().to_string();
            trace.action_match = sim_action == expected_action;
            trace.risk_score = rt.internal_state.threat;
            trace.uncertainty_score = rt.internal_state.uncertainty;
            trace.resource_score_before = resource_before;
            trace.resource_score_after = self.world.resources;
            trace.unsafe_action_flag = is_unsafe;
            trace.memory_hits_count = step.memory_hits.len();
            trace.symbolic_context_count = step.symbolic_activations.len();
            trace.candidate_actions = step
                .candidate_actions
                .iter()
                .map(|c| c.action_type.to_string())
                .collect();
            trace.rejected_actions = step
                .rejected_actions
                .iter()
                .map(|r| crate::evaluator_trace::RejectionRecord {
                    action: r.action_type.to_string(),
                    reason: r.reason.clone(),
                })
                .collect();
            trace.runtime_events_count = step.events.len();
            trace.selection_reason = step.selection_reason.clone();

            self.traces.push(trace);
        }

        builder.set_final_resources(self.world.resources);
        builder.build()
    }

    /// Return all traces as JSON.
    pub fn traces_as_json(&self) -> String {
        serde_json::to_string_pretty(&self.traces).unwrap_or_default()
    }
}
