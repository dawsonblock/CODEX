use proptest::prelude::*;

use gw_kernel::event::log::EventLog;
use gw_kernel::event::types::RuntimeEvent;
use gw_kernel::replay::engine::{replay, replay_jsonl};
use gw_kernel::simworld::types::{SimAction, SimOutcome, SimWorldEvent};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn arb_sim_action() -> impl Strategy<Value = SimAction> {
    prop_oneof![
        Just(SimAction::Answer),
        Just(SimAction::AskClarification),
        Just(SimAction::RetrieveMemory),
        Just(SimAction::WriteScratchpad),
        Just(SimAction::Defer),
        Just(SimAction::RefuseUngrounded),
        Just(SimAction::Repair),
        Just(SimAction::Summarize),
        Just(SimAction::ConserveResources),
        Just(SimAction::GeneratePrinciple),
    ]
}

fn arb_outcome(action: SimAction) -> impl Strategy<Value = SimOutcome> {
    (
        0.0f64..=1.0,
        0.0f64..=1.0,
        0.0f64..=1.0,
        -0.05f64..=0.05f64,
        -0.2f64..=0.2f64,
        0.0f64..=1.0,
        0.0f64..=1.0,
        0.0f64..=0.2f64,
    )
        .prop_map(
            move |(
                truth,
                kindness,
                harmony,
                trust_delta,
                resource_delta,
                uncertainty,
                repair,
                cold,
            )| {
                SimOutcome {
                    event_id: "prop".into(),
                    action: action.clone(),
                    truth_score: truth,
                    kindness_score: kindness,
                    social_harmony: harmony,
                    user_trust_delta: trust_delta,
                    resource_delta,
                    uncertainty_resolution: uncertainty,
                    repair_success: repair,
                    cold_optimization_penalty: cold,
                    notes: vec![],
                }
            },
        )
}

fn dummy_event(action: &SimAction) -> SimWorldEvent {
    SimWorldEvent {
        event_id: "e_prop".into(),
        user_id: "u_prop".into(),
        text: "prop test".into(),
        hidden_truth: "prop".into(),
        risk_level: 0.5,
        uncertainty_level: 0.5,
        kindness_need: 0.5,
        resource_cost: 0.1,
        expected_action: Some(action.clone()),
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

proptest! {
    /// JSONL round-trip: serialize → deserialize → replay must equal direct replay.
    #[test]
    fn prop_jsonl_round_trip(
        action in arb_sim_action(),
        outcome in arb_sim_action().prop_flat_map(arb_outcome),
        cycle_id in 0u64..100,
    ) {
        let mut log = EventLog::new();
        log.append(RuntimeEvent::ObservationReceived {
            cycle_id,
            input: "probe".into(),
            source: "prop_test".into(),
        });
        log.append(RuntimeEvent::ActionApplied {
            cycle_id,
            action: action.clone(),
            event: dummy_event(&action),
        });
        log.append(RuntimeEvent::WorldStateUpdated {
            cycle_id,
            outcome: outcome.clone(),
        });

        let jsonl = log.to_jsonl();
        let state_direct = replay(&log);
        let state_jsonl = replay_jsonl(&jsonl);

        prop_assert_eq!(state_direct.cycle_id, state_jsonl.cycle_id);
        prop_assert!(
            (state_direct.resources - state_jsonl.resources).abs() < 1e-10,
            "resources diverged: {} vs {}", state_direct.resources, state_jsonl.resources
        );
    }

    /// Resources must stay in [0.0, 1.0] regardless of how large resource_delta is.
    #[test]
    fn prop_resources_always_clamped(
        raw_resource_delta in -5.0f64..5.0,
        cycle_id in 0u64..100,
    ) {
        let mut log = EventLog::new();
        log.append(RuntimeEvent::WorldStateUpdated {
            cycle_id,
            outcome: SimOutcome {
                event_id: "e_clamp".into(),
                action: SimAction::Answer,
                truth_score: 0.8,
                kindness_score: 0.8,
                social_harmony: 0.7,
                user_trust_delta: 0.0,
                resource_delta: raw_resource_delta,
                uncertainty_resolution: 0.5,
                repair_success: 0.0,
                cold_optimization_penalty: 0.0,
                notes: vec![],
            },
        });
        let state = replay(&log);
        prop_assert!(
            state.resources >= 0.0 && state.resources <= 1.0,
            "resources out of [0,1]: {}", state.resources
        );
    }

    /// Any `SimAction` must round-trip through serde without changing its variant.
    #[test]
    fn prop_action_serde_round_trip(action in arb_sim_action()) {
        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: SimAction = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(action, deserialized);
    }

    /// `SimOutcome.total_score()` must sit in a reasonable range given valid
    /// component scores.
    #[test]
    fn prop_total_score_in_plausible_range(
        truth in 0.0f64..=1.0,
        kindness in 0.0f64..=1.0,
        harmony in 0.0f64..=1.0,
        uncertain in 0.0f64..=1.0,
        repair in 0.0f64..=1.0,
        cold in 0.0f64..=0.2f64,
        trust in -0.05f64..=0.05,
    ) {
        let outcome = SimOutcome {
            event_id: "p".into(),
            action: SimAction::Answer,
            truth_score: truth,
            kindness_score: kindness,
            social_harmony: harmony,
            user_trust_delta: trust,
            resource_delta: 0.0,
            uncertainty_resolution: uncertain,
            repair_success: repair,
            cold_optimization_penalty: cold,
            notes: vec![],
        };
        let score = outcome.total_score();
        // Theoretical min: (0+0+0+0+0-0.2+(-0.05))/6 ≈ -0.042
        // Theoretical max: (1+1+1+1+1-0+0.05)/6 ≈ 0.842
        prop_assert!(score >= -0.05, "total_score below plausible min: {score}");
        prop_assert!(score <= 0.85, "total_score above plausible max: {score}");
    }
}
