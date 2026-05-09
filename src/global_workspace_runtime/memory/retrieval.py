"""State-weighted retrieval."""
from __future__ import annotations
from typing import Any
from ..core.types import InternalState, ResonanceTag
from .episodic_store import EpisodicMemory
from .semantic_store import SemanticMemory


def retrieve_recent(memory: EpisodicMemory, n: int = 3) -> list[dict[str, Any]]:
    return [
        {
            "episode_id": e.episode_id,
            "input_summary": e.input_summary,
            "selected_candidate": e.selected_candidate,
            "prediction_error": e.prediction_error,
        }
        for e in memory.recent(n)
    ]


def retrieve_state_weighted(episodic: EpisodicMemory, semantic: SemanticMemory, text: str, state: InternalState, tags: list[ResonanceTag], limit: int = 5) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    urgent = state.threat > 0.65 or state.uncertainty > 0.65
    for e in reversed(episodic.recent(20)):
        score = 0.2
        if urgent and e.prediction_error > 0.4:
            score += 0.5
        if any(t.name in {rt.name for rt in e.resonance_tags} for t in tags):
            score += 0.3
        results.append({"type": "episode", "score": score, "input_summary": e.input_summary, "selected_candidate": e.selected_candidate})
    results.extend({"type": "semantic", "score": r["score"] / 3, "input_summary": r["key"], "selected_candidate": r["value"]} for r in semantic.query(text, limit=limit))
    results.sort(key=lambda r: r["score"], reverse=True)
    return results[:limit]
