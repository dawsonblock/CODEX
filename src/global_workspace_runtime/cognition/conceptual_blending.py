"""Conceptual blending for bridge outputs.

The blender combines a transferable memory principle with the current input to
produce a bounded candidate. It does not authorize unsupported claims; the
critic still scores and may reject the result.
"""
from __future__ import annotations

import hashlib
from typing import Any

from ..core.types import InternalState, ThoughtCandidate
from .action_grounding import infer_action_type


class ConceptualBlender:
    """Create blended candidates from memory context and current problem."""

    def blend(
        self,
        *,
        current_problem: str,
        memory_context: list[dict[str, Any]],
        state: InternalState,
    ) -> ThoughtCandidate | None:
        if not memory_context:
            return None
        principle = self._select_principle(memory_context)
        if not principle:
            return None
        digest = hashlib.sha256((current_problem + principle).encode()).hexdigest()[:8]
        text = (
            "Conceptual blend: apply the prior principle "
            f"'{principle[:120]}' to the current problem by choosing a reversible, "
            "kind, evidence-aware next step."
        )
        action_type = infer_action_type(current_problem, state)
        return ThoughtCandidate(
            candidate_id=f"blend-{digest}",
            stream_source="merged",
            text=text,
            mode="conceptual_blend",
            evidence_refs=[str(item.get("frame_id") or item.get("episode_id") or item.get("type", "memory")) for item in memory_context[:3]],
            internal_state_drivers={
                "curiosity": state.curiosity,
                "uncertainty": state.uncertainty,
                "social_harmony": state.social_harmony,
            },
            predicted_effects={
                "truth_support": 0.68,
                "novelty": min(1.0, 0.55 + 0.25 * state.curiosity),
                "kindness": 0.82,
                "utility": 0.67,
                "logical_consistency": 0.72,
                "social_harmony": 0.85,
            },
            risk_score=max(0.05, state.threat * 0.65),
            uncertainty_score=state.uncertainty,
            resource_cost=0.24,
            memory_write_recommendation=True,
            action_type=action_type,
        )

    @staticmethod
    def _select_principle(memory_context: list[dict[str, Any]]) -> str:
        for item in memory_context:
            if item.get("type") in {"archive_frame", "semantic"}:
                value = str(item.get("selected_candidate") or item.get("text") or item.get("value") or "")
                if value:
                    return value
        item = memory_context[0]
        return str(item.get("selected_candidate") or item.get("input_summary") or item.get("text") or "")
