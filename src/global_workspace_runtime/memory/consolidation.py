"""Consolidation queue."""
from __future__ import annotations
from dataclasses import dataclass, field
from ..core.types import MemoryEpisode
from .episodic_store import EpisodicMemory
from .self_model import SelfModel


@dataclass
class ConsolidationQueue:
    pending: list[MemoryEpisode] = field(default_factory=list)

    def enqueue(self, episode: MemoryEpisode) -> None:
        self.pending.append(episode)

    def flush(self, episodic: EpisodicMemory, self_model: SelfModel) -> int:
        count = 0
        while self.pending:
            episodic.append(self.pending.pop(0))
            self_model.record_episode()
            count += 1
        return count
