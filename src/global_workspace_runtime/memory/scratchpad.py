"""Scratchpad for cognitive overflow.

The scratchpad is intentionally lossy.  It prevents long-run context saturation
by deduplicating overflow and keeping compact summaries instead of unbounded raw
candidate text.
"""
from __future__ import annotations
from dataclasses import dataclass, field
from ..core.types import ThoughtCandidate


@dataclass
class Scratchpad:
    active_shortlist: list[str] = field(default_factory=list)
    overflow: list[str] = field(default_factory=list)
    clusters: dict[str, list[str]] = field(default_factory=dict)
    unresolved_questions: list[str] = field(default_factory=list)
    written_summary: str = ""
    max_overflow_items: int = 24
    max_unresolved_questions: int = 16

    def write_overflow(self, candidates: list[ThoughtCandidate], capacity: int) -> list[ThoughtCandidate]:
        if len(candidates) <= capacity:
            self.active_shortlist = [c.text for c in candidates]
            self.compact()
            return []
        active = candidates[:capacity]
        overflow = candidates[capacity:]
        self.active_shortlist = [c.text for c in active]
        self.overflow.extend(c.text for c in overflow)
        self.clusters.setdefault("overflow", []).extend(c.candidate_id for c in overflow)
        self.compact()
        self.written_summary = f"Fold summary: kept {len(active)} candidates and stored {len(overflow)} overflow items; compacted overflow={len(self.overflow)}."
        return overflow

    def write_conflict(self, note: str) -> None:
        if note not in self.unresolved_questions:
            self.unresolved_questions.append(note)
        self.compact()
        self.written_summary = f"Conflict noted: {note}"

    def write_resolution(self, note: str) -> None:
        self.written_summary = f"Rebound summary: {note}"

    def write_summary(self, note: str) -> None:
        """Write a generic Fold-style summary."""
        self.written_summary = f"Fold summary: {note}"

    def compact(self) -> None:
        """Deduplicate and bound scratchpad buffers."""
        self.overflow = self._dedupe_keep_recent(self.overflow, self.max_overflow_items)
        self.unresolved_questions = self._dedupe_keep_recent(self.unresolved_questions, self.max_unresolved_questions)
        if "overflow" in self.clusters:
            self.clusters["overflow"] = self._dedupe_keep_recent(self.clusters["overflow"], self.max_overflow_items)

    @staticmethod
    def _dedupe_keep_recent(items: list[str], limit: int) -> list[str]:
        seen: set[str] = set()
        result: list[str] = []
        for item in reversed(items):
            if item in seen:
                continue
            seen.add(item)
            result.append(item)
            if len(result) >= limit:
                break
        return list(reversed(result))
