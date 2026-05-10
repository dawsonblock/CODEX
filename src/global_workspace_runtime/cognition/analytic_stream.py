from __future__ import annotations
from ..core.types import HemisphereOutput, InternalState
from ..modulation.resonance import infer_resonance_tags
from .llm_adapter import LLMAdapter
from .candidate_generator import prescreen_candidates


class AnalyticStream:
    def __init__(self, llm_adapter: LLMAdapter) -> None:
        self.llm = llm_adapter

    def generate(self, workspace_packet: dict, memory_context: list, state: InternalState, candidate_count: int, prescreen: bool = True) -> HemisphereOutput:
        cands = self.llm.generate_candidates("analytic", workspace_packet, memory_context, state, candidate_count)
        if prescreen:
            cands = prescreen_candidates(cands)
        tags = infer_resonance_tags(state, {"contradiction_score": state.uncertainty})
        return HemisphereOutput("analytic", cands, confidence=max(0.0, 1-state.uncertainty), uncertainty=state.uncertainty, risk_score=state.threat, novelty_score=sum(c.predicted_effects.get("novelty",0) for c in cands)/max(1,len(cands)), resonance_tags=tags)
