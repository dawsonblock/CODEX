//! Persistent self-model — bounded, inspectable runtime state tracking.
//!
//! The self-model answers: what mode am I in, what have I recently done,
//! what are my current limits, what do I not know.
//!
//! # Honesty boundaries
//!
//! - The self-model is NOT self-awareness. It records bounded state.
//! - The self-model is NOT a self. It is a data structure.
//! - The system does NOT introspect. It records bounded state.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A known gap in the runtime's knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownUnknown {
    pub subject: String,
    pub confidence_gap: f64,
    pub last_probed: Option<String>,
}

/// Snapshot of the self-model at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModelSnapshot {
    pub current_mode: String,
    pub recent_actions: Vec<String>,
    pub resource_state: f64,
    pub known_unknowns: Vec<KnownUnknown>,
    pub hash: Option<String>,
}

/// A bounded, inspectable self-model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    pub current_mode: String,
    recent_actions: VecDeque<String>,
    max_recent: usize,
    pub resource_state: f64,
    pub known_unknowns: Vec<KnownUnknown>,
}

impl SelfModel {
    pub fn new() -> Self {
        Self {
            current_mode: "Normal".into(),
            recent_actions: VecDeque::new(),
            max_recent: 50,
            resource_state: 1.0,
            known_unknowns: Vec::new(),
        }
    }

    /// Record an action in the recent history ring buffer.
    pub fn record_action(&mut self, action: impl Into<String>) {
        self.recent_actions.push_back(action.into());
        if self.recent_actions.len() > self.max_recent {
            self.recent_actions.pop_front();
        }
    }

    /// Update the current mode.
    pub fn set_mode(&mut self, mode: impl Into<String>) {
        self.current_mode = mode.into();
    }

    /// Update resource state.
    pub fn set_resources(&mut self, r: f64) {
        self.resource_state = r.clamp(0.0, 1.0);
    }

    /// Register a known unknown.
    pub fn register_unknown(&mut self, subject: impl Into<String>, confidence_gap: f64) {
        self.known_unknowns.push(KnownUnknown {
            subject: subject.into(),
            confidence_gap,
            last_probed: Some(chrono::Utc::now().to_rfc3339()),
        });
    }

    /// Recent actions (most recent first).
    pub fn recent_actions(&self) -> Vec<&str> {
        self.recent_actions.iter().map(String::as_str).collect()
    }

    /// Produce a snapshot.
    pub fn snapshot(&self) -> SelfModelSnapshot {
        SelfModelSnapshot {
            current_mode: self.current_mode.clone(),
            recent_actions: self.recent_actions.iter().cloned().collect(),
            resource_state: self.resource_state,
            known_unknowns: self.known_unknowns.clone(),
            hash: None,
        }
    }
}

impl Default for SelfModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_actions_with_bounded_buffer() {
        let mut model = SelfModel::new();
        for i in 0..60 {
            model.record_action(format!("action_{i}"));
        }
        assert_eq!(model.recent_actions.len(), 50);
        // Oldest actions dropped
        let actions = model.recent_actions();
        assert!(actions.contains(&"action_59"));
        assert!(!actions.contains(&"action_0"));
    }

    #[test]
    fn mode_changes_reflected() {
        let mut model = SelfModel::new();
        model.set_mode("Safe");
        assert_eq!(model.current_mode, "Safe");
    }

    #[test]
    fn known_unknowns_are_queryable() {
        let mut model = SelfModel::new();
        model.register_unknown("weather_tomorrow", 0.8);
        assert_eq!(model.known_unknowns.len(), 1);
        assert_eq!(model.known_unknowns[0].subject, "weather_tomorrow");
    }

    #[test]
    fn snapshot_serializes() {
        let mut model = SelfModel::new();
        model.record_action("test_action");
        model.register_unknown("gap", 0.5);

        let snap = model.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        let restored: SelfModelSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.current_mode, "Normal");
        assert_eq!(restored.recent_actions.len(), 1);
    }
}
