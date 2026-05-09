"""Internal state update with hysteresis and homeostatic forcing."""
from __future__ import annotations
from ..core.types import InternalState, GoalState, ResourceState, clamp01


def update_internal_state(state: InternalState, signals: dict[str, float], goal_state: GoalState, resource_state: ResourceState, alpha: float = 0.35) -> InternalState:
    prediction_error = signals.get("prediction_error", 0.0)
    contradiction = signals.get("contradiction", 0.0)
    risk = signals.get("risk", 0.0)
    user_feedback = signals.get("user_feedback", 0.5)
    action_outcome = signals.get("action_outcome", 0.5)
    memory_conflict = signals.get("memory_conflict", 0.0)
    ambiguity = signals.get("ambiguity", 0.0)

    resource_pressure = resource_state.aggregate_pressure
    goal_pressure = goal_state.goal_pressure

    inertia = clamp01(state.mood_inertia)
    def blend(old: float, target: float, extra_inertia: float = 0.0) -> float:
        effective_alpha = max(0.02, min(0.8, alpha * (1.0 - inertia) * (1.0 - extra_inertia)))
        return clamp01((1.0 - effective_alpha) * old + effective_alpha * target)

    state.arousal = blend(state.arousal, clamp01(prediction_error + risk + resource_pressure))
    state.threat = blend(state.threat, clamp01(risk + 0.4 * memory_conflict), extra_inertia=0.25)
    state.uncertainty = blend(state.uncertainty, clamp01(contradiction + ambiguity + memory_conflict), extra_inertia=0.15)
    state.curiosity = blend(state.curiosity, clamp01(0.4 + prediction_error + 0.3 * state.uncertainty - state.threat))
    state.resource_pressure = blend(state.resource_pressure, resource_pressure, extra_inertia=0.1)

    # Virtue homeostasis. These variables are metrics and constraints, not claims.
    state.honesty = blend(state.honesty, clamp01(1.0 - contradiction - 0.4 * memory_conflict))
    state.logical_consistency = blend(state.logical_consistency, clamp01(1.0 - contradiction))
    state.kindness = blend(state.kindness, clamp01(0.65 + 0.25 * user_feedback - 0.4 * risk))
    state.utility = blend(state.utility, clamp01(0.4 + action_outcome + 0.2 * goal_pressure - resource_pressure))
    state.intelligence = blend(state.intelligence, clamp01(0.45 + 0.3 * prediction_error + 0.2 * state.curiosity - 0.2 * contradiction))
    state.social_harmony = blend(state.social_harmony, clamp01(0.7 + 0.2 * user_feedback - 0.5 * risk - 0.2 * contradiction))

    deficit = max(0.0, 0.65 - min(state.honesty, state.logical_consistency, state.kindness, state.social_harmony))
    target_distress = clamp01(0.4 * state.threat + 0.3 * state.resource_pressure + 0.3 * deficit)
    state.distress = blend(state.distress, target_distress, extra_inertia=0.3)
    state.control = blend(state.control, clamp01(1.0 - state.distress - 0.4 * state.threat - 0.2 * state.resource_pressure))
    state.valence = blend(state.valence, clamp01(0.5 + 0.3 * state.utility + 0.2 * state.social_harmony - 0.5 * state.distress))
    state.dwell_time = state.dwell_time + 1 if state.distress > 0.35 or state.threat > 0.5 else max(0, state.dwell_time - 1)
    return state.clipped()
