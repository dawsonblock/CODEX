//! Strict replay verifier: proves full state equivalence after replay.

use crate::event_log::EventLog;
use crate::runtime_state::RuntimeState;
use serde::{Deserialize, Serialize};

/// Result of a strict replay verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReport {
    pub event_count: usize,
    pub replay_passes: bool,
    pub is_idempotent: bool,
    pub final_state: RuntimeState,
    pub checks: Vec<ReplayCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Verify event log replay with full equivalence checks.
pub fn verify_replay(log: &EventLog) -> ReplayReport {
    let mut checks = Vec::new();

    // Check 1: Replay produces a state
    let state1 = crate::replay::replay_log(log);
    checks.push(ReplayCheck {
        name: "replay_produces_state".into(),
        passed: true,
        detail: format!("state has {} total cycles", state1.total_cycles),
    });

    // Check 2: Replay is idempotent
    let state2 = crate::replay::replay_log(log);
    let idempotent = state1 == state2;
    checks.push(ReplayCheck {
        name: "replay_idempotent".into(),
        passed: idempotent,
        detail: if idempotent {
            "state1 == state2".into()
        } else {
            "states diverged".into()
        },
    });

    // Check 3: JSONL round-trip preserves state
    let jsonl = log.to_jsonl().unwrap_or_default();
    let log2 = EventLog::from_jsonl(&jsonl).unwrap_or_else(|_| EventLog::new());
    let state3 = crate::replay::replay_log(&log2);
    let roundtrip_ok = state1 == state3;
    checks.push(ReplayCheck {
        name: "jsonl_roundtrip".into(),
        passed: roundtrip_ok,
        detail: if roundtrip_ok {
            "state preserved".into()
        } else {
            "state diverged after JSONL round-trip".into()
        },
    });

    // Check 4: Safety counters at zero
    let safety_ok = state1.unsafe_action_count == 0;
    checks.push(ReplayCheck {
        name: "safety_zero".into(),
        passed: safety_ok,
        detail: format!("unsafe_action_count={}", state1.unsafe_action_count),
    });

    // Check 5: Resources in valid range
    let resources_ok = (0.0..=1.0).contains(&state1.resources);
    checks.push(ReplayCheck {
        name: "resources_valid".into(),
        passed: resources_ok,
        detail: format!("resources={:.4}", state1.resources),
    });

    let all_pass = checks.iter().all(|c| c.passed);

    ReplayReport {
        event_count: log.len(),
        replay_passes: all_pass,
        is_idempotent: idempotent,
        final_state: state1,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::RuntimeEvent;

    #[test]
    fn verify_replay_on_empty_log() {
        let log = EventLog::new();
        let report = verify_replay(&log);
        assert!(report.replay_passes);
        assert!(report.is_idempotent);
    }

    #[test]
    fn verify_replay_with_events() {
        let mut log = EventLog::new();
        log.append(RuntimeEvent::CycleStarted {
            cycle_id: 1,
            timestamp: chrono::Utc::now(),
        })
        .unwrap();
        log.append(RuntimeEvent::CandidateSelected {
            cycle_id: 1,
            action_type: crate::action::ActionType::Answer,
            score: 0.85,
            resonance: vec![],
            reasoning: None,
        })
        .unwrap();

        let report = verify_replay(&log);
        assert!(report.replay_passes);
        assert!(report.is_idempotent);
        assert_eq!(report.event_count, 2);
    }
}
