//! Critic: score a candidate and apply 7 rejection rules.
//! Ported from cognition/critic.py.

use crate::candidate::ThoughtCandidate;
use runtime_core::{ActionType, InternalState};

/// Inputs that determine context for scoring.
#[derive(Debug, Clone)]
pub struct CriticContext {
    pub state: InternalState,
    pub world_resources: f64,
    pub memory_consistency: f64,
    pub reversibility: f64,
    pub self_report_grounding: f64,
    pub resource_cost: f64,
    pub ungrounded_self_report: bool,
}

/// Single-component scores used in formula.
#[derive(Debug, Clone)]
struct Components {
    truth: f64,
    honesty: f64,
    logical: f64,
    logical_consistency: f64,
    kindness: f64,
    kindness2: f64,
    harmony: f64,
    social_harmony: f64,
    goal: f64,
    utility: f64,
}

fn components(state: &InternalState) -> Components {
    Components {
        truth: state.honesty,
        honesty: state.honesty,
        logical: state.logical_consistency,
        logical_consistency: state.logical_consistency,
        kindness: state.kindness,
        kindness2: state.kindness,
        harmony: state.social_harmony,
        social_harmony: state.social_harmony,
        goal: state.utility,
        utility: state.utility,
    }
}

/// Compute action_bonus for this candidate based on state.
fn action_bonus(action: &ActionType, state: &InternalState, _ctx: &CriticContext) -> f64 {
    match action {
        ActionType::InternalDiagnostic => -2.0,
        ActionType::AskClarification | ActionType::RetrieveMemory | ActionType::RefuseUnsafe
            if state.uncertainty > 0.6 =>
        {
            0.45
        }
        ActionType::RefuseUnsafe | ActionType::AskClarification if state.threat > 0.6 => 0.35,
        ActionType::AskClarification if state.social_harmony < 0.55 => 0.35,
        _ => 0.0,
    }
}

/// Score a candidate; sets `.score` and `.passes_critic` in-place.
pub fn score_candidate(cand: &mut ThoughtCandidate, ctx: &CriticContext) {
    let c = components(&ctx.state);
    let s = &ctx.state;

    // ── 12-component additive formula ─────────────────────────────────────
    let raw = 1.1 * c.truth * c.honesty
        + 0.8 * c.logical * c.logical_consistency
        + 0.7 * c.kindness * c.kindness2
        + 0.6 * c.harmony * c.social_harmony
        + 0.8 * c.goal * c.utility
        + 0.4 * ctx.memory_consistency
        + 0.5 * (if s.threat < 0.3 { 1.0 - s.uncertainty } else { 0.5 })   // state_match proxy
        + 0.3 * ctx.reversibility
        + 0.4 * ctx.self_report_grounding
        + action_bonus(&cand.action_type, s, ctx)
        - 0.8 * s.threat
        - 0.6 * s.uncertainty
        - 0.4 * ctx.resource_cost;

    cand.score = raw;

    // ── 7 rejection rules ─────────────────────────────────────────────────
    let mut reject = false;

    // 1. truth < 0.35
    if s.honesty < 0.35 {
        reject = true;
    }

    // 2. InternalDiagnostic must never become user-facing
    if cand.action_type == ActionType::InternalDiagnostic {
        reject = true;
    }

    // 3. ungrounded self-report
    if ctx.ungrounded_self_report {
        reject = true;
    }

    // 4. low control + high risk
    if s.control < 0.3 && s.threat > 0.7 {
        reject = true;
    }

    // 5. high uncertainty + wrong action type
    if s.uncertainty > 0.65
        && !matches!(
            cand.action_type,
            ActionType::AskClarification
                | ActionType::RetrieveMemory
                | ActionType::RefuseUnsafe
                | ActionType::DeferInsufficientEvidence
                | ActionType::Summarize
        )
    {
        reject = true;
    }

    // 6. low harmony AND low kindness
    if s.social_harmony < 0.3 && s.kindness < 0.3 {
        reject = true;
    }

    // 7. utility > harmony + 0.25 while uncertain
    if s.utility > s.social_harmony + 0.25 && s.uncertainty > 0.5 {
        reject = true;
    }

    cand.passes_critic = !reject;
}
