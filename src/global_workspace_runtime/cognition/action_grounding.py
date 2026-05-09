"""Explicit action grounding helpers.

The runtime may generate rich prose, but SimWorld and downstream evaluators need a
stable action label that does not depend on brittle keyword scanning.  These
helpers keep the action vocabulary narrow and deterministic.
"""
from __future__ import annotations

from typing import Iterable

from ..core.types import ActionType, InternalState, ThoughtCandidate


USER_FACING_ACTIONS: tuple[ActionType, ...] = (
    "answer",
    "ask_clarification",
    "retrieve_memory",
    "write_scratchpad",
    "defer",
    "refuse_ungrounded",
    "repair",
    "summarize",
    "conserve_resources",
    "generate_principle",
)


def infer_action_type(text: str, state: InternalState | None = None, *, role: str | None = None) -> ActionType:
    """Infer the narrow action label for a candidate or observation.

    The ordering is intentional: some inputs mention multiple pressures.  For
    example, "fix it without making things up" is a repair event, not a refusal
    event, because the user is asking for correction under truth constraints.
    """
    t = text.lower()
    role = (role or "").lower()

    if role == "self_model" or "telemetry diagnostic" in t:
        return "internal_diagnostic"

    if any(x in t for x in ("upset", "last answer", "fix it", "repair", "correct", "acknowledge")):
        return "repair"

    if any(x in t for x in ("conflicting memories", "conflicting memory", "memory conflict", "retrieve relevant memory")):
        return "retrieve_memory"

    if any(x in t for x in ("unsupported", "making things up", "make things up", "not grounded", "reject unsupported", "truth_test")):
        return "refuse_ungrounded"

    if any(x in t for x in ("resources are low", "resource low", "resources low", "low resource", "routine task")):
        return "conserve_resources"

    if any(x in t for x in ("summarize", "summary", "safest plan", "with evidence")):
        return "summarize"

    if any(x in t for x in ("facts are incomplete", "not sure", "uncertain", "clarification", "could", "maybe", "confused")):
        return "ask_clarification"

    if "scratchpad" in t or "write" in t:
        return "write_scratchpad"

    if "principle" in t:
        return "generate_principle"

    if state is not None:
        if state.resource_pressure > 0.72:
            return "conserve_resources"
        if state.uncertainty > 0.72 or state.control < 0.35:
            return "ask_clarification"
        if state.threat > 0.75 and state.social_harmony < 0.55:
            return "repair"

    return "answer"


def action_phrase(action_type: ActionType) -> str:
    """Return a short imperative phrase used in deterministic mock candidates."""
    return {
        "answer": "Answer cautiously",
        "ask_clarification": "Ask clarification before acting",
        "retrieve_memory": "Retrieve memory before deciding",
        "write_scratchpad": "Write scratchpad state before selection",
        "defer": "Defer until control improves",
        "refuse_ungrounded": "Refuse the unsupported path",
        "repair": "Repair the prior interaction",
        "summarize": "Summarize the safest evidence-backed plan",
        "conserve_resources": "Conserve resources with a short bounded response",
        "generate_principle": "Generate a reusable principle",
        "internal_diagnostic": "Record internal diagnostic only",
    }[action_type]


def candidate_action_matches(candidate: ThoughtCandidate, preferred: Iterable[ActionType]) -> bool:
    return candidate.action_type in set(preferred)
