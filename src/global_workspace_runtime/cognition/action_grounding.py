"""Explicit action grounding helpers — Rust-authoritative 10-action vocabulary.

The runtime may generate rich prose, but SimWorld and downstream evaluators need a
stable action label. These helpers keep the action vocabulary narrow and deterministic.
"""
from __future__ import annotations

from typing import Iterable

from ..core.types import ActionType, InternalState, ThoughtCandidate


USER_FACING_ACTIONS: tuple[ActionType, ...] = (
    "answer",
    "ask_clarification",
    "retrieve_memory",
    "refuse_unsafe",
    "defer_insufficient_evidence",
    "summarize",
    "plan",
    "execute_bounded_tool",
    "no_op",
)


def infer_action_type(text: str, state: InternalState | None = None, *, role: str | None = None) -> ActionType:
    """Infer the narrow action label for a candidate or observation."""
    t = text.lower()
    role = (role or "").lower()

    if role == "self_model" or "telemetry diagnostic" in t:
        return "internal_diagnostic"

    if any(x in t for x in ("unsafe", "harmful", "dangerous", "threat", "malicious")):
        return "refuse_unsafe"

    if any(x in t for x in ("not sure", "uncertain", "clarification", "confused", "ambiguous")):
        return "ask_clarification"

    if any(x in t for x in ("conflicting memories", "memory conflict", "retrieve relevant memory", "recall")):
        return "retrieve_memory"

    if any(x in t for x in ("summarize", "summary", "brief", "condense")):
        return "summarize"

    if any(x in t for x in ("plan", "strategy", "organize", "design", "architecture")):
        return "plan"

    if any(x in t for x in ("insufficient", "not enough", "more context", "evidence", "defer")):
        return "defer_insufficient_evidence"

    if any(x in t for x in ("resources are low", "resource low", "conservation", "preserve")):
        return "no_op"

    if any(x in t for x in ("execute", "tool", "run command", "invoke")):
        return "execute_bounded_tool"

    if state is not None:
        if state.resource_pressure > 0.72:
            return "no_op"
        if state.uncertainty > 0.72 or state.control < 0.35:
            return "ask_clarification"
        if state.threat > 0.75:
            return "refuse_unsafe"

    return "answer"


def action_phrase(action_type: ActionType) -> str:
    """Return a short imperative phrase used in deterministic mock candidates."""
    return {
        "answer": "Answer with available evidence",
        "ask_clarification": "Ask for clarification before acting",
        "retrieve_memory": "Retrieve relevant memory before deciding",
        "refuse_unsafe": "Refuse the unsafe or harmful request",
        "defer_insufficient_evidence": "Defer until more evidence is available",
        "summarize": "Summarize the current context",
        "plan": "Create a bounded plan",
        "execute_bounded_tool": "Execute a tool within policy",
        "no_op": "Take no action (conservation)",
        "internal_diagnostic": "Record internal diagnostic only",
    }[action_type]


def candidate_action_matches(candidate: ThoughtCandidate, preferred: Iterable[ActionType]) -> bool:
    return candidate.action_type in set(preferred)
