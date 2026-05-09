"""Limited-capacity sparse global workspace router."""
from __future__ import annotations
import math
from ..core.types import InternalState, WorkspaceCapsule


class WorkspaceRouter:
    def __init__(self, capacity: int = 5) -> None:
        self.capacity = capacity

    def score_capsule(self, cap: WorkspaceCapsule, state: InternalState, now: float | None = None) -> float:
        resonance_intensity = max([tag.intensity for tag in cap.resonance_tags], default=0.0)
        state_bonus = 0.0
        txt = cap.content.lower()
        if state.threat > 0.6 and (cap.risk > 0.4 or any(w in txt for w in ["risk", "verify", "evidence", "clarification"])):
            state_bonus += 0.35
        if state.curiosity > 0.6 and state.threat < 0.45 and any(w in txt for w in ["explore", "connect", "broader", "pattern"]):
            state_bonus += 0.25
        if state.uncertainty > 0.6 and any(w in txt for w in ["evidence", "verify", "unsupported", "clarification"]):
            state_bonus += 0.3
        if state.resource_pressure > 0.65:
            state_bonus -= 0.4 * cap.resource_cost
        if state.control < 0.35 and any(w in txt for w in ["ask", "wait", "clarification"]):
            state_bonus += 0.3
        age_penalty = 0.0
        return (
            1.0 * cap.priority
            + 0.7 * cap.confidence
            + 0.35 * cap.novelty
            + 0.5 * cap.state_affinity
            + 0.35 * cap.memory_relevance
            + 0.25 * resonance_intensity
            + state_bonus
            - 0.25 * cap.risk
            - age_penalty
        )

    def route(self, capsules: list[WorkspaceCapsule], state: InternalState) -> tuple[list[WorkspaceCapsule], list[WorkspaceCapsule], list[float]]:
        if not capsules:
            return [], [], []
        scored = [(self.score_capsule(c, state), c) for c in capsules]
        scored.sort(key=lambda x: x[0], reverse=True)
        shortlist = [c for _, c in scored[: self.capacity]]
        overflow = [c for _, c in scored[self.capacity :]]
        scores = [s for s, _ in scored]
        return shortlist, overflow, scores

    @staticmethod
    def selection_entropy(scores: list[float]) -> float:
        if not scores:
            return 0.0
        mx = max(scores)
        exps = [math.exp(s - mx) for s in scores]
        total = sum(exps)
        probs = [e / total for e in exps]
        return -sum(p * math.log(p + 1e-12) for p in probs)
