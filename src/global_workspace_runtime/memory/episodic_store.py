"""Episodic memory store."""
from __future__ import annotations
from collections import deque
from typing import Iterator
from ..core.types import MemoryEpisode


class EpisodicMemory:
    def __init__(self, maxlen: int = 1000) -> None:
        self._episodes: deque[MemoryEpisode] = deque(maxlen=maxlen)

    def append(self, episode: MemoryEpisode) -> None:
        self._episodes.append(episode)

    def recent(self, n: int = 5) -> list[MemoryEpisode]:
        return list(self._episodes)[-n:]

    def __iter__(self) -> Iterator[MemoryEpisode]:
        return iter(self._episodes)

    def __len__(self) -> int:
        return len(self._episodes)

    def clear(self) -> None:
        self._episodes.clear()
