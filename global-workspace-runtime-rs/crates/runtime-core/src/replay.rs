//! Replay: fold a sequence of events to produce the final RuntimeState.

use crate::event::RuntimeEvent;
use crate::event_log::EventLog;
use crate::reducer::reduce;
use crate::runtime_state::RuntimeState;

/// Replay a slice of events from an initial default state.
pub fn replay(events: &[RuntimeEvent]) -> RuntimeState {
    events.iter().fold(RuntimeState::default(), reduce)
}

/// Replay everything in an EventLog.
pub fn replay_log(log: &EventLog) -> RuntimeState {
    replay(log.events())
}

/// Replay a JSONL string directly, returning the final state.
pub fn replay_jsonl(s: &str) -> Result<RuntimeState, crate::event_log::EventLogError> {
    let log = EventLog::from_jsonl(s)?;
    Ok(replay_log(&log))
}
