"""Metric-grounded resonance tags."""
from __future__ import annotations
from ..core.types import InternalState, ResonanceTag


def infer_resonance_tags(state: InternalState, metrics: dict[str, float] | None = None) -> list[ResonanceTag]:
    metrics = metrics or {}
    tags: list[ResonanceTag] = []
    contradiction = metrics.get("contradiction_score", state.uncertainty)
    tension = metrics.get("hemispheric_tension", 0.0)
    novelty = metrics.get("novelty_delta", state.curiosity)
    if contradiction > 0.65:
        tags.append(ResonanceTag("Glitch", "looping", contradiction, {"contradiction_score": contradiction}))
    if state.curiosity > 0.65 and state.threat < 0.45:
        tags.append(ResonanceTag("Pull", "expansive", state.curiosity, {"curiosity": state.curiosity, "threat": state.threat}))
    if tension > 0.55:
        tags.append(ResonanceTag("Tangle", "divergent", tension, {"hemispheric_tension": tension}))
    if state.resource_pressure > 0.7 or state.distress > 0.65:
        tags.append(ResonanceTag("Fold", "deep", max(state.resource_pressure, state.distress), {"resource_pressure": state.resource_pressure, "distress": state.distress}))
    if state.logical_consistency > 0.8 and state.honesty > 0.8 and state.utility > 0.55:
        tags.append(ResonanceTag("Kick", "sustained", min(state.logical_consistency, state.honesty), {"logical_consistency": state.logical_consistency, "honesty": state.honesty, "utility": state.utility}))
    if state.social_harmony > 0.75 and state.kindness > 0.75:
        tags.append(ResonanceTag("Weld", "sustained", min(state.social_harmony, state.kindness), {"social_harmony": state.social_harmony, "kindness": state.kindness}))
    if novelty > 0.75 and state.threat < 0.4:
        tags.append(ResonanceTag("Bloom", "expansive", novelty, {"novelty_delta": novelty, "threat": state.threat}))
    if not tags:
        tags.append(ResonanceTag("Hum", "sustained", 0.2, {"baseline": 0.2}))
    return tags
