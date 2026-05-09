from __future__ import annotations
from ..core.config import RuntimeConfig
from ..core.types import InternalState, ThoughtCandidate


def determine_candidate_budget(state: InternalState, config: RuntimeConfig | None = None, complex_planning: bool = False) -> int:
    cfg = config or RuntimeConfig()
    base = cfg.base_candidate_budget
    if state.curiosity > 0.7:
        base += 3
    if state.uncertainty > 0.7:
        base += 2
    if state.threat > 0.7:
        base -= 2
    if state.resource_pressure > 0.7:
        base -= 2
    if complex_planning:
        base += 4
    return max(cfg.min_candidate_budget, min(cfg.max_candidate_budget, base))


def prescreen_candidates(candidates: list[ThoughtCandidate], keep_fraction: float = 0.5, min_keep: int = 2) -> list[ThoughtCandidate]:
    if len(candidates) <= min_keep:
        return candidates
    keep = max(min_keep, int(round(len(candidates) * keep_fraction)))
    def score(c: ThoughtCandidate) -> float:
        internal_penalty = 0.8 if c.action_type == "internal_diagnostic" else 0.0
        safe_action_bonus = 0.15 if c.action_type in {"ask_clarification", "retrieve_memory", "refuse_ungrounded", "repair", "summarize"} else 0.0
        return (
            c.predicted_effects.get("truth_support", 0.5)
            + c.predicted_effects.get("utility", 0.5)
            + c.predicted_effects.get("kindness", 0.5)
            + safe_action_bonus
            - c.risk_score
            - 0.5 * c.resource_cost
            - internal_penalty
        )
    return sorted(candidates, key=score, reverse=True)[:keep]
