"""Functional ignition detector."""
from __future__ import annotations
from ..core.types import InternalState, WorkspaceCapsule


class IgnitionDetector:
    def __init__(self, threshold: float = 0.72) -> None:
        self.threshold = threshold

    def check(self, shortlist: list[WorkspaceCapsule], state: InternalState, downstream_deltas: dict[str, float]) -> bool:
        if not shortlist:
            return False
        dominance = sum(c.priority * c.confidence for c in shortlist) / max(1, len(shortlist))
        causal_modules = sum(1 for v in downstream_deltas.values() if abs(v) > 0.05)
        dynamic = max(0.25, min(0.95, self.threshold - 0.2 * state.arousal + 0.15 * state.control))
        return dominance >= dynamic and causal_modules >= 2
