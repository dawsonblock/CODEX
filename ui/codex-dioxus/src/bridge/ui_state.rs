use crate::bridge::metrics::global_metrics;
use crate::bridge::types::{EvidenceDisplay, LiveClaimDisplay, PressureMetrics, TimelineEvent};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

/// Global state for bridging runtime events to Dioxus UI components.
/// Uses Signal to enable reactive updates across the entire UI.
pub struct UIRuntimeState {
    /// Timeline events from the runtime.
    pub timeline_events: Signal<Vec<TimelineEvent>>,

    /// All claims with grounding status.
    pub claims: Signal<Vec<LiveClaimDisplay>>,
    pub claims_by_id: Signal<HashMap<String, LiveClaimDisplay>>,

    /// All evidence with provenance.
    pub evidence: Signal<Vec<EvidenceDisplay>>,
    pub evidence_by_id: Signal<HashMap<String, EvidenceDisplay>>,

    /// Pressure metrics history.
    pub pressure_metrics: Signal<Vec<PressureMetrics>>,
    pub current_pressure: Signal<Option<PressureMetrics>>,

    /// Current cycle and metadata.
    pub current_cycle: Signal<usize>,
    pub last_update: Signal<String>,
    pub is_loading: Signal<bool>,
    pub error_message: Signal<Option<String>>,
}

impl UIRuntimeState {
    /// Create a new UI runtime state with empty initial values.
    pub fn new() -> Self {
        Self {
            timeline_events: Signal::new(Vec::new()),
            claims: Signal::new(Vec::new()),
            claims_by_id: Signal::new(HashMap::new()),
            evidence: Signal::new(Vec::new()),
            evidence_by_id: Signal::new(HashMap::new()),
            pressure_metrics: Signal::new(Vec::new()),
            current_pressure: Signal::new(None),
            current_cycle: Signal::new(0),
            last_update: Signal::new("Never".to_string()),
            is_loading: Signal::new(false),
            error_message: Signal::new(None),
        }
    }

    /// Update timeline events.
    pub fn set_timeline_events(&mut self, events: Vec<TimelineEvent>) {
        let start = Instant::now();
        self.timeline_events.set(events);
        self.update_timestamp();
        global_metrics().record_signal_write("timeline_events", start.elapsed().as_micros() as u64);
    }

    /// Add a single timeline event.
    pub fn add_timeline_event(&mut self, event: TimelineEvent) {
        let start = Instant::now();
        let mut events = self.timeline_events.read().clone();
        events.push(event);
        self.timeline_events.set(events);
        self.update_timestamp();
        global_metrics().record_signal_write("timeline_events_add", start.elapsed().as_micros() as u64);
    }

    /// Update claims and index by ID.
    pub fn set_claims(&mut self, claims: Vec<LiveClaimDisplay>) {
        let start = Instant::now();
        let mut index = HashMap::new();
        for claim in &claims {
            index.insert(claim.claim_id.clone(), claim.clone());
        }
        self.claims.set(claims);
        self.claims_by_id.set(index);
        self.update_timestamp();
        global_metrics().record_signal_write("claims", start.elapsed().as_micros() as u64);
    }

    /// Get a specific claim by ID.
    pub fn get_claim(&self, claim_id: &str) -> Option<LiveClaimDisplay> {
        let start = Instant::now();
        let result = self.claims_by_id.read().get(claim_id).cloned();
        global_metrics().record_signal_read("claims_by_id", start.elapsed().as_micros() as u64);
        result
    }

    /// Update evidence and index by ID.
    pub fn set_evidence(&mut self, evidence: Vec<EvidenceDisplay>) {
        let start = Instant::now();
        let mut index = HashMap::new();
        for ev in &evidence {
            index.insert(ev.entry_id.clone(), ev.clone());
        }
        self.evidence.set(evidence);
        self.evidence_by_id.set(index);
        self.update_timestamp();
        global_metrics().record_signal_write("evidence", start.elapsed().as_micros() as u64);
    }

    /// Get a specific evidence by ID.
    pub fn get_evidence(&self, entry_id: &str) -> Option<EvidenceDisplay> {
        let start = Instant::now();
        let result = self.evidence_by_id.read().get(entry_id).cloned();
        global_metrics().record_signal_read("evidence_by_id", start.elapsed().as_micros() as u64);
        result
    }

    /// Update pressure metrics.
    pub fn set_pressure_metrics(&mut self, metrics: Vec<PressureMetrics>) {
        let start = Instant::now();
        let current = metrics.last().cloned();
        self.pressure_metrics.set(metrics);
        self.current_pressure.set(current);
        self.update_timestamp();
        global_metrics().record_signal_write("pressure_metrics", start.elapsed().as_micros() as u64);
    }

    /// Add a single pressure metric.
    pub fn add_pressure_metric(&mut self, metric: PressureMetrics) {
        let start = Instant::now();
        let mut metrics = self.pressure_metrics.read().clone();
        metrics.push(metric.clone());
        self.pressure_metrics.set(metrics);
        self.current_pressure.set(Some(metric));
        self.update_timestamp();
        global_metrics().record_signal_write("pressure_metrics_add", start.elapsed().as_micros() as u64);
    }

    /// Set current cycle.
    pub fn set_current_cycle(&mut self, cycle: usize) {
        let start = Instant::now();
        self.current_cycle.set(cycle);
        self.update_timestamp();
        global_metrics().record_signal_write("current_cycle", start.elapsed().as_micros() as u64);
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        let start = Instant::now();
        self.is_loading.set(loading);
        global_metrics().record_signal_write("is_loading", start.elapsed().as_micros() as u64);
    }

    /// Set error message.
    pub fn set_error(&mut self, error: Option<String>) {
        let start = Instant::now();
        self.error_message.set(error);
        global_metrics().record_signal_write("error_message", start.elapsed().as_micros() as u64);
    }

    /// Update timestamp to current time.
    fn update_timestamp(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        self.last_update
            .set(format!("{:.0}s", now.as_secs_f64() % 86400.0));
    }

    /// Clear all data.
    pub fn reset(&mut self) {
        let start = Instant::now();
        self.timeline_events.set(Vec::new());
        self.claims.set(Vec::new());
        self.claims_by_id.set(HashMap::new());
        self.evidence.set(Vec::new());
        self.evidence_by_id.set(HashMap::new());
        self.pressure_metrics.set(Vec::new());
        self.current_pressure.set(None);
        self.current_cycle.set(0);
        self.error_message.set(None);
        self.last_update.set("Reset".to_string());
        global_metrics().record_signal_write("reset_all", start.elapsed().as_micros() as u64);
    }

    /// Get summary statistics.
    pub fn get_summary(&self) -> StateSummary {
        let start = Instant::now();
        let summary = StateSummary {
            timeline_event_count: self.timeline_events.read().len(),
            claim_count: self.claims.read().len(),
            evidence_count: self.evidence.read().len(),
            pressure_reading_count: self.pressure_metrics.read().len(),
            current_cycle: *self.current_cycle.read(),
            is_loading: *self.is_loading.read(),
            has_error: self.error_message.read().is_some(),
        };
        global_metrics().record_signal_read("get_summary", start.elapsed().as_micros() as u64);
        summary
    }
}

#[derive(Debug, Clone)]
pub struct StateSummary {
    pub timeline_event_count: usize,
    pub claim_count: usize,
    pub evidence_count: usize,
    pub pressure_reading_count: usize,
    pub current_cycle: usize,
    pub is_loading: bool,
    pub has_error: bool,
}

/// Error boundary component for graceful error handling.
#[component]
pub fn ErrorBoundary(children: Element) -> Element {
    rsx! {
        div { class: "error-boundary",
            {children}
        }
    }
}

/// Loading placeholder component.
#[component]
pub fn LoadingPlaceholder(message: String) -> Element {
    rsx! {
        div { class: "loading-placeholder",
            div { class: "loading-spinner" }
            p { class: "loading-text", "{message}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Dioxus runtime - Signals cannot be created in unit tests. Test in UI context instead.
    fn test_state_initialization() {
        let state = UIRuntimeState::new();
        let summary = state.get_summary();

        assert_eq!(summary.timeline_event_count, 0);
        assert_eq!(summary.claim_count, 0);
        assert_eq!(summary.evidence_count, 0);
        assert_eq!(summary.pressure_reading_count, 0);
        assert_eq!(summary.current_cycle, 0);
        assert!(!summary.is_loading);
        assert!(!summary.has_error);
    }

    #[test]
    #[ignore] // Requires Dioxus runtime - Signals cannot be created in unit tests. Test in UI context instead.
    fn test_claim_indexing() {
        let mut state = UIRuntimeState::new();
        let claims = vec![LiveClaimDisplay {
            claim_id: "cl-001".to_string(),
            subject: "Test".to_string(),
            predicate: "property".to_string(),
            object: None,
            grounding_status: crate::bridge::types::GroundingStatus::Unverified,
            evidence_count: 0,
            contradiction_count: 0,
            confidence_pct: 50,
            evidence_ids: Vec::new(),
        }];

        state.set_claims(claims);
        let retrieved = state.get_claim("cl-001");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().claim_id, "cl-001");
    }

    #[test]
    #[ignore] // Requires Dioxus runtime - Signals cannot be created in unit tests. Test in UI context instead.
    fn test_error_handling() {
        let mut state = UIRuntimeState::new();
        assert!(state.error_message.read().is_none());

        state.set_error(Some("Test error".to_string()));
        assert!(state.error_message.read().is_some());
        assert_eq!(state.error_message.read().as_ref().unwrap(), "Test error");
    }

    #[test]
    #[ignore] // Requires Dioxus runtime - Signals cannot be created in unit tests. Test in UI context instead.
    fn test_reset() {
        let mut state = UIRuntimeState::new();
        state.set_current_cycle(5);
        state.set_loading(true);
        state.set_error(Some("Error".to_string()));

        state.reset();

        let summary = state.get_summary();
        assert_eq!(summary.current_cycle, 0);
        assert!(!summary.is_loading);
        assert!(!summary.has_error);
    }
}
