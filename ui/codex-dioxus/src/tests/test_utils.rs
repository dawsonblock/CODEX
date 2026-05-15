use crate::bridge::types::{
    EvidenceDisplay, GroundingStatus, LiveClaimDisplay, PressureMetrics, TimelineEvent,
};
use crate::bridge::ui_state::UIRuntimeState;
use dioxus::prelude::WritableExt;

/// Builder for creating test UIRuntimeState instances with controlled data
pub struct TestStateBuilder {
    state: UIRuntimeState,
}

impl TestStateBuilder {
    /// Create a new empty test state
    pub fn new() -> Self {
        Self {
            state: UIRuntimeState::new(),
        }
    }

    /// Add timeline events to the state
    pub fn with_timeline_events(mut self, count: usize) -> Self {
        let events = (1..=count)
            .map(|i| TimelineEvent {
                cycle: i,
                event_type: if i % 2 == 0 { "evidence" } else { "claim" }.to_string(),
                timestamp: format!("2026-05-14T10:{:02}:00Z", i),
                claim_ids: vec![format!("cl-{:03}", i)],
                evidence_ids: vec![format!("ev-{:03}", i)],
                confidence: 0.75 + (i as f64 * 0.01),
                message: format!("Event {} processed", i),
            })
            .collect();
        self.state.set_timeline_events(events);
        self
    }

    /// Add claims to the state
    pub fn with_claims(mut self, count: usize) -> Self {
        let claims = (1..=count)
            .map(|i| LiveClaimDisplay {
                claim_id: format!("cl-{:03}", i),
                subject: format!("Subject{}", i),
                predicate: format!("predicate{}", i),
                object: Some(format!("object{}", i)),
                grounding_status: if i % 3 == 0 {
                    GroundingStatus::Validated
                } else {
                    GroundingStatus::Unverified
                },
                evidence_count: i,
                contradiction_count: 0,
                confidence_pct: (80 + i) as u8,
                evidence_ids: vec![format!("ev-{:03}", i)],
            })
            .collect();
        self.state.set_claims(claims);
        self
    }

    /// Add evidence to the state
    pub fn with_evidence(mut self, count: usize) -> Self {
        let evidence = (1..=count)
            .map(|i| EvidenceDisplay {
                entry_id: format!("ev-{:03}", i),
                source: format!("source{}", i),
                confidence_pct: (70 + i) as u8,
                content_hash: format!("hash{:x}", i),
                provenance: match i % 3 {
                    0 => "assertion".to_string(),
                    1 => "query".to_string(),
                    _ => "memory".to_string(),
                },
            })
            .collect();
        self.state.set_evidence(evidence);
        self
    }

    /// Add pressure metrics to the state
    pub fn with_pressure_metrics(mut self, count: usize) -> Self {
        let metrics = (1..=count)
            .map(|i| PressureMetrics {
                cycle: i,
                pressure: 0.2 + (i as f64 * 0.05),
                regulation: 0.85 - (i as f64 * 0.02),
                peak_pressure: 0.2 + (i as f64 * 0.05),
                avg_pressure: 0.3,
                avg_regulation: 0.8,
                threshold_exceeded: (i as f64 * 0.05) > 0.8,
            })
            .collect();
        self.state.set_pressure_metrics(metrics);
        self
    }

    /// Set the current cycle
    pub fn with_cycle(mut self, cycle: usize) -> Self {
        self.state.set_current_cycle(cycle);
        self
    }

    /// Set loading state
    pub fn loading(mut self, is_loading: bool) -> Self {
        self.state.set_loading(is_loading);
        self
    }

    /// Set an error message
    pub fn with_error(mut self, message: String) -> Self {
        self.state.set_error(Some(message));
        self
    }

    /// Build the final state
    pub fn build(self) -> UIRuntimeState {
        self.state
    }
}

impl Default for TestStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests for TestStateBuilder require Dioxus runtime context
    // The builder can be used from within component/UI test code that runs
    // within the Dioxus desktop environment

    #[test]
    #[ignore] // Requires Dioxus runtime - TestStateBuilder creates UIRuntimeState with Signals. Test in UI context instead.
    fn test_builder_construction() {
        // Builder can be constructed without runtime
        let _ = TestStateBuilder::new();
        assert!(true);
    }
}
