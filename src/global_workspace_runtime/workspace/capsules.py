"""Capsule construction helpers."""
from __future__ import annotations
import uuid
from ..core.types import InternalState, ResonanceTag, ThoughtCandidate, WorkspaceCapsule


def state_affinity(candidate: ThoughtCandidate, state: InternalState) -> float:
    drivers = candidate.internal_state_drivers
    diff = 0.0
    keys = ["threat", "uncertainty", "curiosity", "control", "social_harmony"]
    for k in keys:
        if k in drivers:
            diff += abs(float(drivers[k]) - float(getattr(state, k)))
    return max(0.0, min(1.0, 1.0 - diff / max(1, len(keys))))


def candidate_to_capsule(candidate: ThoughtCandidate, state: InternalState, resonance_tags: list[ResonanceTag], memory_relevance: float = 0.0) -> WorkspaceCapsule:
    truth = candidate.predicted_effects.get("truth_support", 0.5)
    utility = candidate.predicted_effects.get("utility", 0.5)
    novelty = candidate.predicted_effects.get("novelty", 0.0)
    priority = max(0.0, min(1.0, 0.4 * truth + 0.3 * utility + 0.2 * state_affinity(candidate, state) + 0.1 * memory_relevance))
    return WorkspaceCapsule(
        capsule_id=f"cap-{uuid.uuid4().hex[:10]}",
        source=candidate.stream_source,
        content=candidate.text,
        priority=priority,
        confidence=truth,
        novelty=novelty,
        risk=candidate.risk_score,
        state_affinity=state_affinity(candidate, state),
        evidence_refs=candidate.evidence_refs,
        resonance_tags=resonance_tags,
        candidate=candidate,
        resource_cost=candidate.resource_cost,
        memory_relevance=memory_relevance,
    )
