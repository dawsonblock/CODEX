"""Interhemispheric bridge over analytic and associative streams."""
from __future__ import annotations
from ..core.types import BridgeOutput, HemisphereOutput, ThoughtCandidate, ResonanceTag


class InterhemisphericBridge:
    def compare(self, analytic: HemisphereOutput, associative: HemisphereOutput) -> BridgeOutput:
        a_text = " ".join(c.text.lower() for c in analytic.candidates)
        b_text = " ".join(c.text.lower() for c in associative.candidates)
        a_words = set(a_text.split())
        b_words = set(b_text.split())
        overlap = len(a_words & b_words) / max(1, len(a_words | b_words))
        agreement = min(1.0, overlap + 0.2 * min(analytic.confidence, associative.confidence))
        contradiction = max(0.0, abs(analytic.risk_score - associative.risk_score) + abs(analytic.uncertainty - associative.uncertainty) - 0.2)
        novelty_delta = abs(analytic.novelty_score - associative.novelty_score)
        evidence_gap = max(0.0, associative.novelty_score - analytic.confidence)
        tension = min(1.0, 0.4 * contradiction + 0.3 * novelty_delta + 0.3 * evidence_gap)
        conflicts: list[str] = []
        scratchpad: list[str] = []
        if tension > 0.45:
            note = f"Stream disagreement: tension={tension:.2f}, contradiction={contradiction:.2f}, evidence_gap={evidence_gap:.2f}"
            conflicts.append(note)
            scratchpad.append(note)
        merged: list[ThoughtCandidate] = []
        # Keep both sets if conflict is productive; otherwise interleave best candidates.
        if tension > 0.65:
            merged = analytic.candidates[:2] + associative.candidates[:2]
        else:
            merged = (analytic.candidates + associative.candidates)[: max(2, len(analytic.candidates + associative.candidates))]
        tags: list[ResonanceTag] = []
        if agreement > 0.65:
            tags.append(ResonanceTag("Weld", "sustained", agreement, {"agreement_score": agreement}))
        if tension > 0.55:
            tags.append(ResonanceTag("Tangle", "divergent", tension, {"hemispheric_tension": tension}))
        elif novelty_delta > 0.35:
            tags.append(ResonanceTag("Bloom", "expansive", novelty_delta, {"novelty_delta": novelty_delta}))
        return BridgeOutput(merged, conflicts, agreement, contradiction, novelty_delta, evidence_gap, tension, tags, scratchpad)
