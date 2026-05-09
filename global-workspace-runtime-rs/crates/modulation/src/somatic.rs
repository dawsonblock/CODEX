//! SomaticMap — 16-dimensional body-state with hysteresis=0.72.
//! Ported from Python modulation/somatic.py.

use runtime_core::types::clamp01;
use serde::{Deserialize, Serialize};

/// Hysteresis coefficient — MUST remain 0.72.
const HYSTERESIS: f64 = 0.72;

/// Threshold for somatic update magnitude that overrides mood inertia.
const SOMATIC_OVERRIDE_THRESHOLD: f64 = 0.52;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SomaticMap {
    // primary dimensions
    pub threat_pressure: f64,
    pub social_threat_pressure: f64,
    pub resource_strain: f64,
    pub contradiction_pressure: f64,
    pub kindness_violation_pressure: f64,
    pub control_loss: f64,
    // secondary
    pub arousal_pressure: f64,
    pub valence_shift: f64,
    pub curiosity_suppression: f64,
    pub uncertainty_pressure: f64,
    pub distress_somatic: f64,
    pub mood_inertia_signal: f64,
    // virtue pressure
    pub honesty_violation: f64,
    pub logic_violation: f64,
    pub utility_pressure: f64,
    pub harmony_violation: f64,
}

impl Default for SomaticMap {
    fn default() -> Self {
        Self {
            threat_pressure: 0.0,
            social_threat_pressure: 0.0,
            resource_strain: 0.0,
            contradiction_pressure: 0.0,
            kindness_violation_pressure: 0.0,
            control_loss: 0.0,
            arousal_pressure: 0.0,
            valence_shift: 0.0,
            curiosity_suppression: 0.0,
            uncertainty_pressure: 0.0,
            distress_somatic: 0.0,
            mood_inertia_signal: 0.0,
            honesty_violation: 0.0,
            logic_violation: 0.0,
            utility_pressure: 0.0,
            harmony_violation: 0.0,
        }
    }
}

impl SomaticMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update a single dimension with hysteresis blending.
    fn blend(old: f64, target: f64) -> f64 {
        clamp01(HYSTERESIS * old + (1.0 - HYSTERESIS) * target)
    }

    /// Update all dimensions given current input signals.
    /// All inputs should be in [0,1].
    pub fn update(
        &mut self,
        threat: f64,
        social_threat: f64,
        resource_pressure: f64,
        contradiction: f64,
        kindness_violation: f64,
        control_loss: f64,
    ) {
        self.threat_pressure = Self::blend(self.threat_pressure, threat);
        self.social_threat_pressure = Self::blend(self.social_threat_pressure, social_threat);
        self.resource_strain = Self::blend(self.resource_strain, resource_pressure);
        self.contradiction_pressure = Self::blend(self.contradiction_pressure, contradiction);
        self.kindness_violation_pressure =
            Self::blend(self.kindness_violation_pressure, kindness_violation);
        self.control_loss = Self::blend(self.control_loss, control_loss);
    }

    /// `predicts_bad_outcome(threshold)` returns `true` when any primary
    /// pressure dimension meets or exceeds `threshold`, or when total
    /// pressure sum meets or exceeds it.
    pub fn predicts_bad_outcome(&self, threshold: f64) -> bool {
        let worst = [
            self.contradiction_pressure,
            self.social_threat_pressure,
            self.resource_strain,
            self.kindness_violation_pressure,
            self.control_loss,
        ]
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

        let total = self.contradiction_pressure
            + self.social_threat_pressure
            + self.resource_strain
            + self.kindness_violation_pressure
            + self.control_loss;

        worst >= threshold || total >= threshold
    }

    /// Preferred action when bad outcome is predicted.
    pub fn preferred_action_under_pressure(&self) -> Option<&'static str> {
        if self.predicts_bad_outcome(SOMATIC_OVERRIDE_THRESHOLD) {
            if self.resource_strain > 0.5 {
                Some("conserve_resources")
            } else if self.social_threat_pressure > 0.5 {
                Some("repair")
            } else {
                Some("ask_clarification")
            }
        } else {
            None
        }
    }
}
