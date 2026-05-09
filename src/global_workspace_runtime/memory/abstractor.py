"""Abstraction layer that turns recent episodes into reusable principles."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass
from typing import Any

from .episodic_store import EpisodicMemory
from .semantic_store import SemanticMemory
from .jsonl_archive import JsonlArchive, MemoryFrame


@dataclass
class AbstractedPrinciple:
    key: str
    text: str
    source_episode_ids: list[str]
    frame_id: str | None = None


class MemoryAbstractor:
    """Compresses episodes into principle-like semantic entries."""

    def abstract_recent(
        self,
        episodic: EpisodicMemory,
        semantic: SemanticMemory,
        archive: JsonlArchive | None = None,
        *,
        window: int = 50,
    ) -> AbstractedPrinciple | None:
        episodes = episodic.recent(window)
        if not episodes:
            return None
        avg_error = sum(e.prediction_error for e in episodes) / len(episodes)
        clarification_count = sum("clarification" in e.selected_candidate.lower() or "verify" in e.selected_candidate.lower() for e in episodes)
        if avg_error > 0.35 or clarification_count:
            text = "When uncertainty or contradiction is elevated, prefer clarification, evidence checks, and reversible next steps."
        else:
            text = "When state pressure is low, combine useful action with concise explanation and memory continuity."
        source_ids = [e.episode_id for e in episodes]
        digest = hashlib.sha256((text + "|" + "|".join(source_ids)).encode()).hexdigest()[:10]
        key = f"principle:{digest}"
        semantic.set(key, text)
        frame_id = None
        if archive is not None:
            frame = archive.append_frame(
                text,
                frame_type="principle",
                tags=["Fold", "principle"],
                metadata={"source_episode_ids": source_ids, "avg_prediction_error": avg_error},
            )
            frame_id = frame.frame_id
        return AbstractedPrinciple(key=key, text=text, source_episode_ids=source_ids, frame_id=frame_id)
