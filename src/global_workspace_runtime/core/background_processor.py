"""Dormant background processor.

This processor performs bounded idle work: it inspects unresolved scratchpad
questions, queries the long-term archive, and writes non-binding notes or
principles.  It does not send messages, change external state, or bypass the
critic/planner.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .runtime_state import RuntimeState


@dataclass
class BackgroundWorkResult:
    """Result of one idle processing tick."""

    ran: bool
    actions: list[str] = field(default_factory=list)
    archive_hits: int = 0
    unresolved_questions: int = 0
    principle_written: str | None = None


class BackgroundProcessor:
    """Runs bounded self-maintenance while waiting for new input."""

    def tick(self, state: RuntimeState, *, query_hint: str | None = None, max_hits: int = 3) -> BackgroundWorkResult:
        """Run one background tick.

        Args:
            state: runtime state container.
            query_hint: optional query; otherwise derived from scratchpad.
            max_hits: maximum archive hits.

        Returns:
            BackgroundWorkResult describing work performed.
        """
        unresolved = list(getattr(state.scratchpad, "unresolved_questions", []))
        query = query_hint or (unresolved[-1] if unresolved else "")
        result = BackgroundWorkResult(ran=False, unresolved_questions=len(unresolved))
        if not query and state.internal_state.uncertainty < 0.55:
            return result

        result.ran = True
        if query:
            hits = state.long_term_archive.query(query, limit=max_hits)
            result.archive_hits = len(hits)
            if hits:
                summary = f"Background recall for '{query[:60]}': {len(hits)} archive hit(s)."
                state.scratchpad.write_summary(summary)
                result.actions.append("archive_recall")
        if unresolved:
            note = f"Background note: unresolved question retained for later workspace review: {unresolved[-1][:120]}"
            state.scratchpad.write_conflict(note)
            result.actions.append("scratchpad_review")
        if state.internal_state.uncertainty > 0.7 or state.internal_state.distress > 0.5:
            principle = state.memory_abstractor.abstract_recent(
                state.episodic_memory,
                state.semantic_memory,
                state.long_term_archive,
                window=10,
            )
            if principle:
                result.principle_written = principle.key
                result.actions.append("principle_abstraction")
        return result
