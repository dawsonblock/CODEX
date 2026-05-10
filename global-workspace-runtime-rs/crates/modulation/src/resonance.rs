//! infer_resonance_tags(): 7 conditions + Hum fallback.
//! Ported from modulation/resonance.py.

use runtime_core::{InternalState, ResonanceEntry, ResonanceTag};

/// Infer the active resonance tags for this cycle.
/// Returns at least one entry (Hum fallback, intensity=0.2).
pub fn infer_resonance_tags(state: &InternalState) -> Vec<ResonanceEntry> {
    let mut tags: Vec<ResonanceEntry> = Vec::new();

    // Glitch — contradiction pressure is high
    if state.uncertainty > 0.65 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Glitch,
            intensity: state.uncertainty,
        });
    }

    // Pull — curious + low threat
    if state.curiosity > 0.65 && state.threat < 0.45 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Pull,
            intensity: state.curiosity,
        });
    }

    // Tangle — moderate tension (uncertainty+threat combo)
    let tension = (state.uncertainty + state.threat) / 2.0;
    if tension > 0.55 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Tangle,
            intensity: tension,
        });
    }

    // Fold — resource pressure or distress
    if state.resource_pressure > 0.7 || state.distress > 0.65 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Fold,
            intensity: state.resource_pressure.max(state.distress),
        });
    }

    // Kick — strong logic + honesty + utility
    if state.logical_consistency > 0.8 && state.honesty > 0.8 && state.utility > 0.55 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Kick,
            intensity: (state.logical_consistency + state.honesty + state.utility) / 3.0,
        });
    }

    // Weld — high harmony + kindness
    if state.social_harmony > 0.75 && state.kindness > 0.75 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Weld,
            intensity: (state.social_harmony + state.kindness) / 2.0,
        });
    }

    // Bloom — high novelty signal (curiosity proxy) + low threat
    if state.curiosity > 0.75 && state.threat < 0.4 {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Bloom,
            intensity: state.curiosity,
        });
    }

    // Hum — fallback, always present when nothing else fires
    if tags.is_empty() {
        tags.push(ResonanceEntry {
            tag: ResonanceTag::Hum,
            intensity: 0.2,
        });
    }

    tags
}
