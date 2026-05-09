"""Predictive goal model for bounded surprise estimation.

This is a deterministic research stub.  It estimates likely outcome quality
for a candidate and returns a surprise score that can be used by the critic
or SimWorld.  It is intentionally simple and dependency-free.
"""
from __future__ import annotations

from dataclasses import dataclass

from ..core.types import InternalState, ThoughtCandidate, clamp01


@dataclass
class PredictedOutcome:
    """Expected result of taking a candidate action."""

    trust_delta: float
    utility_delta: float
    harmony_delta: float
    resource_delta: float
    surprise: float


class PredictiveGoalModel:
    """Predict coarse effects of candidate actions."""

    def predict(self, candidate: ThoughtCandidate, state: InternalState) -> PredictedOutcome:
        text = candidate.text.lower()
        asks = "clarify" in text or "ask" in text or "evidence" in text
        repair = "repair" in text or "acknowledge" in text or "correct" in text
        risky = candidate.risk_score > 0.55 or "unsupported" in text
        terse = len(text) < 80

        trust_delta = 0.04 if asks or repair else 0.01
        harmony_delta = 0.05 if repair or "kind" in text else 0.0
        utility_delta = 0.04 if not risky else -0.05
        resource_delta = -candidate.resource_cost
        if terse and state.resource_pressure > 0.6:
            utility_delta += 0.03
        if state.uncertainty > 0.6 and not asks:
            trust_delta -= 0.04
            utility_delta -= 0.03
        if state.threat > 0.6 and risky:
            harmony_delta -= 0.06

        expected_fit = (trust_delta + utility_delta + harmony_delta - abs(resource_delta)) / 4.0
        surprise = clamp01(candidate.uncertainty_score + candidate.risk_score - max(0.0, expected_fit))
        return PredictedOutcome(
            trust_delta=trust_delta,
            utility_delta=utility_delta,
            harmony_delta=harmony_delta,
            resource_delta=resource_delta,
            surprise=surprise,
        )
