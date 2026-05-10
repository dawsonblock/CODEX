//! internal_state_update(): blended update for InternalState.
//! Ported from modulation/internal_state.py.

use runtime_core::{types::clamp01, InternalState};

/// Inputs to a single update step — all in [0, 1].
#[derive(Debug, Clone)]
pub struct UpdateInputs {
    pub prediction_error: f64,
    pub risk: f64,
    pub resource_pressure: f64,
    pub contradiction: f64,
    pub ambiguity: f64,
    pub memory_conflict: f64,
    pub utility: f64,
    pub social_harmony: f64,
    pub virtue_deficit: f64,
}

/// Compute effective alpha with inertia dampening.
fn effective_alpha(mood_inertia: f64, extra_inertia: f64) -> f64 {
    let raw = 0.35 * (1.0 - mood_inertia) * (1.0 - extra_inertia);
    raw.clamp(0.02, 0.8)
}

fn blend(old: f64, target: f64, alpha: f64) -> f64 {
    clamp01((1.0 - alpha) * old + alpha * target)
}

/// Update `state` in-place with one update step.
pub fn update_internal_state(state: &mut InternalState, inp: &UpdateInputs) {
    let alpha_normal = effective_alpha(state.mood_inertia, 0.0);
    let alpha_threat = effective_alpha(state.mood_inertia, 0.25); // extra_inertia for threat
    let alpha = alpha_normal;

    // ── arousal ──────────────────────────────────────────────────────────────
    let target_arousal = clamp01(inp.prediction_error + inp.risk + inp.resource_pressure);
    state.arousal = blend(state.arousal, target_arousal, alpha);

    // ── threat ───────────────────────────────────────────────────────────────
    let target_threat = clamp01(inp.risk + 0.4 * inp.memory_conflict);
    state.threat = blend(state.threat, target_threat, alpha_threat);

    // ── uncertainty ──────────────────────────────────────────────────────────
    let target_uncertainty = clamp01(inp.contradiction + inp.ambiguity + inp.memory_conflict);
    state.uncertainty = blend(state.uncertainty, target_uncertainty, alpha);

    // ── curiosity ────────────────────────────────────────────────────────────
    let target_curiosity =
        clamp01(0.4 + inp.prediction_error + 0.3 * state.uncertainty - state.threat);
    state.curiosity = blend(state.curiosity, target_curiosity, alpha);

    // ── honesty ──────────────────────────────────────────────────────────────
    let target_honesty = clamp01(1.0 - inp.contradiction - 0.4 * inp.memory_conflict);
    state.honesty = blend(state.honesty, target_honesty, alpha);

    // ── distress ─────────────────────────────────────────────────────────────
    let target_distress =
        clamp01(0.4 * state.threat + 0.3 * inp.resource_pressure + 0.3 * inp.virtue_deficit);
    state.distress = blend(state.distress, target_distress, alpha);

    // ── control ──────────────────────────────────────────────────────────────
    let target_control =
        clamp01(1.0 - state.distress - 0.4 * state.threat - 0.2 * inp.resource_pressure);
    state.control = blend(state.control, target_control, alpha);

    // ── valence ──────────────────────────────────────────────────────────────
    let target_valence =
        clamp01(0.5 + 0.3 * inp.utility + 0.2 * inp.social_harmony - 0.5 * state.distress);
    state.valence = blend(state.valence, target_valence, alpha);

    // ── resource_pressure ────────────────────────────────────────────────────
    state.resource_pressure = blend(state.resource_pressure, inp.resource_pressure, alpha);

    // ── dwell_time ───────────────────────────────────────────────────────────
    state.dwell_time = (state.dwell_time + 1.0).min(100.0);
}
