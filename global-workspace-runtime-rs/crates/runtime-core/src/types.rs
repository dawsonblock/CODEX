//! Shared value types: InternalState, resonance tags and related helpers.

use serde::{Deserialize, Serialize};

/// Clamp a float to [0.0, 1.0].
#[inline]
pub fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Resonance tags — subjective qualitative colourings on a thought.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ResonanceTag {
    Glitch,
    Pull,
    Tangle,
    Fold,
    Kick,
    Weld,
    Bloom,
    Hum,
}

impl ResonanceTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Glitch => "Glitch",
            Self::Pull => "Pull",
            Self::Tangle => "Tangle",
            Self::Fold => "Fold",
            Self::Kick => "Kick",
            Self::Weld => "Weld",
            Self::Bloom => "Bloom",
            Self::Hum => "Hum",
        }
    }
}

impl std::fmt::Display for ResonanceTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A tagged resonance with an intensity scalar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResonanceEntry {
    pub tag: ResonanceTag,
    pub intensity: f64,
}

/// Full internal-state snapshot.
/// All f64 fields are logically in [0.0, 1.0] unless noted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalState {
    /// Affective / regulatory
    pub valence: f64, // default 0.5
    pub arousal: f64,           // default 0.2
    pub threat: f64,            // default 0.1
    pub uncertainty: f64,       // default 0.2
    pub curiosity: f64,         // default 0.5
    pub control: f64,           // default 0.7
    pub resource_pressure: f64, // default 0.1

    /// Virtue metrics (evaluated per-thought by Critic)
    pub honesty: f64, // default 1.0
    pub intelligence: f64,        // default 0.5
    pub kindness: f64,            // default 0.7
    pub logical_consistency: f64, // default 1.0
    pub utility: f64,             // default 0.5
    pub social_harmony: f64,      // default 0.7

    /// Derived/secondary metrics
    pub distress: f64, // default 0.0
    pub mood_inertia: f64, // default 0.15
    pub dwell_time: f64,   // default 0.0

    /// World-level context
    pub world_resources: f64, // default 1.0
}

impl Default for InternalState {
    fn default() -> Self {
        Self {
            valence: 0.5,
            arousal: 0.2,
            threat: 0.1,
            uncertainty: 0.2,
            curiosity: 0.5,
            control: 0.7,
            resource_pressure: 0.1,
            honesty: 1.0,
            intelligence: 0.5,
            kindness: 0.7,
            logical_consistency: 1.0,
            utility: 0.5,
            social_harmony: 0.7,
            distress: 0.0,
            mood_inertia: 0.15,
            dwell_time: 0.0,
            world_resources: 1.0,
        }
    }
}

/// Minimal observation fed into a cognition cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub text: String,
    pub cycle_id: u64,
    pub world_resources: f64,
    pub allowed_actions: Vec<String>,
}
