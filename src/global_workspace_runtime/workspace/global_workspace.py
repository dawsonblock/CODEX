"""Global workspace as limited-capacity broadcast."""
from __future__ import annotations
from ..core.types import InternalState, WorkspaceCapsule, WorkspaceState
from .router import WorkspaceRouter
from .ignition import IgnitionDetector


class GlobalWorkspace:
    def __init__(self, capacity: int = 5) -> None:
        self.router = WorkspaceRouter(capacity)
        self.ignition = IgnitionDetector()
        self.previous_shortlist_ids: set[str] = set()

    def update(self, cycle_id: int, capsules: list[WorkspaceCapsule], state: InternalState, downstream_deltas: dict[str, float] | None = None) -> WorkspaceState:
        downstream_deltas = downstream_deltas or {}
        shortlist, overflow, scores = self.router.route(capsules, state)
        current_ids = {c.capsule_id for c in shortlist}
        shortlist_delta = 1.0 - (len(current_ids & self.previous_shortlist_ids) / max(1, len(current_ids | self.previous_shortlist_ids))) if self.previous_shortlist_ids else 1.0
        self.previous_shortlist_ids = current_ids
        downstream_deltas = dict(downstream_deltas)
        downstream_deltas.setdefault("workspace_shortlist", shortlist_delta)
        ignition = self.ignition.check(shortlist, state, downstream_deltas)
        broadcast = {
            "shortlist_ids": [c.capsule_id for c in shortlist],
            "contents": [c.content for c in shortlist],
            "sources": [c.source for c in shortlist],
            "selection_entropy": self.router.selection_entropy(scores),
            "overflow_count": len(overflow),
        }
        return WorkspaceState(cycle_id, shortlist, broadcast, ignition, state.snapshot(), [], overflow)
