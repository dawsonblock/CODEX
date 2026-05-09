"""Reflexive self-model stream over measured telemetry."""
from __future__ import annotations
from ..core.types import HemisphereOutput, InternalState
from ..modulation.resonance import infer_resonance_tags
from .llm_adapter import LLMAdapter


class SelfModelStream:
    def __init__(self, llm_adapter: LLMAdapter) -> None:
        self.llm = llm_adapter

    def generate(self, workspace_packet: dict, memory_context: list, state: InternalState, candidate_count: int = 1) -> HemisphereOutput:
        cands = self.llm.generate_candidates("self_model", workspace_packet, memory_context, state, candidate_count)
        tags = infer_resonance_tags(state, {"contradiction_score": state.uncertainty})
        return HemisphereOutput("self_model", cands, confidence=0.9, uncertainty=state.uncertainty, risk_score=state.threat, novelty_score=0.1, resonance_tags=tags)
