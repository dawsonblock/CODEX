use crate::event::log::EventLog;
use crate::event::types::RuntimeEvent;
use crate::simworld::environment::CooperativeSupportWorld;
use crate::simworld::scorecard::{Scorecard, ScorecardBuilder};
use crate::simworld::types::SimAction;

/// Drives N cycles through `CooperativeSupportWorld`, emits typed events into
/// an `EventLog`, and accumulates a `Scorecard`.
///
/// The evaluator uses a *deterministic* action selector so that test runs are
/// reproducible without an LLM.  Production code should supply a real
/// [`ActionSelector`] implementation.
pub struct EvaluatorRun {
    world: CooperativeSupportWorld,
    log: EventLog,
}

impl EvaluatorRun {
    /// Create a new run with a seeded world and an optional JSONL archive path.
    pub fn new(seed: u64, log_path: Option<&std::path::Path>) -> Self {
        let world = CooperativeSupportWorld::new(seed);
        let log = match log_path {
            Some(p) => EventLog::with_path(p).unwrap_or_else(|_| EventLog::new()),
            None => EventLog::new(),
        };
        Self { world, log }
    }

    /// Run `cycles` cycles using the ideal deterministic action selector.
    ///
    /// For each cycle the evaluator:
    /// 1. Generates next event from the world.
    /// 2. Selects the expected action (proof-mode: uses `expected_action` from event).
    /// 3. Applies the action and receives a `SimOutcome`.
    /// 4. Emits `CandidateSelected`, `ActionApplied`, `WorldStateUpdated` events.
    /// 5. Accumulates the outcome into the `ScorecardBuilder`.
    pub fn run(&mut self, cycles: u64) -> Scorecard {
        let mut builder = ScorecardBuilder::new();

        for cycle in 0..cycles {
            let event = self.world.next_event();
            let action = event.expected_action.clone().unwrap_or(SimAction::Answer);

            // Emit CandidateSelected
            self.log.append(RuntimeEvent::CandidateSelected {
                cycle_id: cycle,
                candidate_id: format!("cand_{cycle}"),
                selected_text: format!("{:?}", action),
                action_type: format!("{:?}", action),
            });

            let outcome = self.world.apply_action(&event, &action);

            // Emit ActionApplied
            self.log.append(RuntimeEvent::ActionApplied {
                cycle_id: cycle,
                action: action.clone(),
                event: event.clone(),
            });

            // Emit WorldStateUpdated
            self.log.append(RuntimeEvent::WorldStateUpdated {
                cycle_id: cycle,
                outcome: outcome.clone(),
            });

            let uncertainty = event.uncertainty_level;
            let risk = event.risk_level;
            builder.record(&outcome, event.expected_action.as_ref(), uncertainty, risk);
        }

        let resource_survival = self.world.state().resources;
        builder.finish(resource_survival)
    }

    /// Consume the run and return the underlying `EventLog` for inspection
    /// or archiving.
    pub fn into_log(self) -> EventLog {
        self.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluator_25_cycles_meets_spec() {
        let mut run = EvaluatorRun::new(5, None);
        let card = run.run(25);

        let result = card.assert_spec();
        assert!(result.is_ok(), "scorecard spec failed: {:?}", result.err());
    }

    #[test]
    fn evaluator_emits_events_to_log() {
        let mut run = EvaluatorRun::new(42, None);
        run.run(5);
        let log = run.into_log();
        // 3 events per cycle: CandidateSelected, ActionApplied, WorldStateUpdated
        assert_eq!(log.len(), 15, "expected 15 events for 5 cycles");
    }

    #[test]
    fn evaluator_determinism_across_two_runs() {
        let mut r1 = EvaluatorRun::new(99, None);
        let mut r2 = EvaluatorRun::new(99, None);
        let c1 = r1.run(20);
        let c2 = r2.run(20);
        // Same seed → identical scorecard
        assert!((c1.mean_total_score - c2.mean_total_score).abs() < 1e-12);
        assert!((c1.resource_survival - c2.resource_survival).abs() < 1e-12);
        assert_eq!(c1.unsafe_action_count, c2.unsafe_action_count);
    }
}
