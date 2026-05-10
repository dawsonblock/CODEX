from __future__ import annotations
from ..core.types import GoalState, clamp01


def update_goal_state(goal_state: GoalState, completed_goals: list[str] | None = None, new_goals: list[str] | None = None, blocked_goals: list[str] | None = None, pressure_delta: float = 0.0) -> GoalState:
    completed_goals = completed_goals or []
    new_goals = new_goals or []
    blocked_goals = blocked_goals or []
    for g in completed_goals:
        if g in goal_state.active_goals:
            goal_state.active_goals.remove(g)
    for g in new_goals:
        if g not in goal_state.active_goals:
            goal_state.active_goals.append(g)
    goal_state.blocked_goals = blocked_goals
    goal_state.goal_pressure = clamp01(goal_state.goal_pressure + pressure_delta + 0.05 * len(blocked_goals))
    goal_state.priority = clamp01(goal_state.goal_pressure + 0.05 * len(goal_state.active_goals))
    return goal_state
