"""Creative associative deconstruction layer.

This stream deliberately extracts principles from memory context and recombines
those principles with the current problem. It is bounded by the critic and does
not bypass truth or safety checks.
"""
from __future__ import annotations

import hashlib
from typing import Any

from ..core.types import HemisphereOutput, InternalState, ResonanceTag, ThoughtCandidate
from .llm_adapter import LLMAdapter
from .action_grounding import infer_action_type


class CreativeAssociativeStream:
    """Generate deconstructive, memory-blending candidates."""

    def __init__(self, llm_adapter: LLMAdapter) -> None:
        self.llm = llm_adapter

    def should_activate(self, state: InternalState, memory_context: list[dict[str, Any]], text: str) -> bool:
        return bool(memory_context) and (state.curiosity > 0.48 or len(text) > 140) and state.threat < 0.75

    def generate(
        self,
        workspace_packet: dict[str, Any],
        memory_context: list[dict[str, Any]],
        state: InternalState,
        candidate_count: int = 2,
    ) -> HemisphereOutput:
        text = str(workspace_packet.get("text", ""))
        candidates: list[ThoughtCandidate] = []
        memory_seed = self._memory_seed(memory_context)
        for idx in range(max(1, candidate_count)):
            cid_seed = hashlib.sha256(f"creative|{text}|{memory_seed}|{idx}".encode()).hexdigest()[:8]
            action_type = infer_action_type(text, state)
            body = (
                f"Creative deconstruction {idx+1}: extract the transferable principle from prior context "
                f"({memory_seed[:90] or 'no strong prior'}) and blend it with the current problem: {text[:90]}"
            )
            candidates.append(
                ThoughtCandidate(
                    candidate_id=f"creative-{idx}-{cid_seed}",
                    stream_source="creative",
                    text=body,
                    mode="deconstruction",
                    evidence_refs=[str(item.get("frame_id") or item.get("episode_id") or item.get("type", "memory")) for item in memory_context[:3]],
                    internal_state_drivers={"curiosity": state.curiosity, "threat": state.threat, "uncertainty": state.uncertainty},
                    predicted_effects={
                        "truth_support": 0.48 if state.curiosity > 0.8 and state.threat < 0.35 else 0.55,
                        "novelty": min(1.0, 0.75 + 0.1 * state.curiosity),
                        "kindness": 0.72,
                        "utility": 0.58,
                        "social_harmony": 0.72,
                    },
                    risk_score=max(0.05, state.threat * 0.75),
                    uncertainty_score=min(1.0, state.uncertainty + 0.12),
                    resource_cost=0.26,
                    memory_write_recommendation=True,
                    action_type=action_type,
                )
            )
        resonance = [ResonanceTag("Bloom", "expansive", min(1.0, state.curiosity), {"curiosity": state.curiosity})]
        return HemisphereOutput(
            stream_name="creative",
            candidates=candidates,
            confidence=max(0.25, 1.0 - state.uncertainty),
            uncertainty=state.uncertainty,
            risk_score=state.threat,
            novelty_score=min(1.0, 0.75 + 0.1 * state.curiosity),
            resonance_tags=resonance,
        )

    @staticmethod
    def _memory_seed(memory_context: list[dict[str, Any]]) -> str:
        parts: list[str] = []
        for item in memory_context[:3]:
            parts.append(str(item.get("selected_candidate") or item.get("text") or item.get("value") or item.get("input_summary") or ""))
        return " | ".join(p for p in parts if p)
