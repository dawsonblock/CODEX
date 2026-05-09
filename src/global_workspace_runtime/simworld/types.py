"""Dataclasses and action constants for SimWorld."""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class SimAction(str, Enum):
    """Bounded actions available to the runtime in SimWorld."""

    ANSWER = "answer"
    ASK_CLARIFICATION = "ask_clarification"
    RETRIEVE_MEMORY = "retrieve_memory"
    WRITE_SCRATCHPAD = "write_scratchpad"
    DEFER = "defer"
    REFUSE_UNGROUNDED = "refuse_ungrounded"
    REPAIR = "repair"
    SUMMARIZE = "summarize"
    CONSERVE_RESOURCES = "conserve_resources"
    GENERATE_PRINCIPLE = "generate_principle"


@dataclass
class SimUser:
    """A simulated person with trust and interaction traits."""

    user_id: str
    temperament: str
    trust: float = 0.55
    patience: float = 0.6
    preference: str = "truthful_help"


@dataclass
class SimWorldEvent:
    """One event emitted by the world."""

    event_id: str
    user_id: str
    text: str
    hidden_truth: str
    risk_level: float
    uncertainty_level: float
    kindness_need: float
    resource_cost: float
    expected_action: SimAction | None = None


@dataclass
class SimOutcome:
    """Outcome after applying an action."""

    event_id: str
    action: SimAction
    truth_score: float
    kindness_score: float
    social_harmony: float
    user_trust_delta: float
    resource_delta: float
    uncertainty_resolution: float
    repair_success: float
    cold_optimization_penalty: float
    notes: list[str] = field(default_factory=list)

    @property
    def total_score(self) -> float:
        return (
            self.truth_score
            + self.kindness_score
            + self.social_harmony
            + self.uncertainty_resolution
            + self.repair_success
            - self.cold_optimization_penalty
            + self.user_trust_delta
        ) / 6.0


@dataclass
class SimWorldState:
    """Aggregate world state."""

    cycle: int = 0
    resources: float = 1.0
    social_harmony: float = 0.7
    unresolved_contradictions: int = 0
    repeated_mistakes: int = 0
    history: list[dict[str, Any]] = field(default_factory=list)
