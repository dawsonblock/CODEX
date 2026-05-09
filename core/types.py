"""Core dataclasses for GlobalWorkspaceRuntime."""
from __future__ import annotations

import time
from dataclasses import dataclass, field, asdict
from enum import Enum
from typing import Any, Literal

Texture = Literal["scattered", "looping", "deep", "expansive", "sustained", "distorted", "divergent"]


class ActionType(str, Enum):
    """Bounded vocabulary of first-class runtime actions.

    ``str`` mixin ensures equality with plain string literals so existing code
    that compares ``action_type == "answer"`` continues to work without changes.
    """
    ANSWER               = "answer"
    ASK_CLARIFICATION    = "ask_clarification"
    RETRIEVE_MEMORY      = "retrieve_memory"
    WRITE_SCRATCHPAD     = "write_scratchpad"
    DEFER                = "defer"
    REFUSE_UNGROUNDED    = "refuse_ungrounded"
    REPAIR               = "repair"
    SUMMARIZE            = "summarize"
    CONSERVE_RESOURCES   = "conserve_resources"
    GENERATE_PRINCIPLE   = "generate_principle"
    INTERNAL_DIAGNOSTIC  = "internal_diagnostic"


def now_ts() -> float:
    return time.time()


def clamp01(x: float) -> float:
    return max(0.0, min(1.0, float(x)))


@dataclass
class ObservationPacket:
    observation_id: str
    timestamp: float
    source: str
    text: str
    metadata: dict[str, Any] = field(default_factory=dict)
    provenance: dict[str, Any] = field(default_factory=dict)


@dataclass
class InternalState:
    valence: float = 0.5
    arousal: float = 0.2
    threat: float = 0.1
    uncertainty: float = 0.2
    curiosity: float = 0.5
    control: float = 0.7
    resource_pressure: float = 0.1

    # Upgraded regulating variables. These are not output claims; they are runtime metrics.
    honesty: float = 1.0
    intelligence: float = 0.5
    kindness: float = 0.7
    logical_consistency: float = 1.0
    utility: float = 0.5
    social_harmony: float = 0.7
    distress: float = 0.0
    mood_inertia: float = 0.15
    dwell_time: int = 0
    world_resources: float = 1.0

    def validate(self) -> None:
        for k, v in asdict(self).items():
            if k == "dwell_time":
                continue
            if not 0.0 <= float(v) <= 1.0:
                raise ValueError(f"InternalState.{k} must be within [0, 1], got {v}")

    def clipped(self) -> "InternalState":
        for k in asdict(self):
            if k == "dwell_time":
                continue
            setattr(self, k, clamp01(getattr(self, k)))
        return self

    def snapshot(self) -> dict[str, float | int]:
        return asdict(self)


@dataclass
class GoalState:
    active_goals: list[str] = field(default_factory=lambda: ["answer accurately", "minimize harm", "preserve clarity"])
    goal_pressure: float = 0.4
    blocked_goals: list[str] = field(default_factory=list)
    priority: float = 0.5


@dataclass
class ResourceState:
    compute_budget: float = 1.0
    time_budget: float = 1.0
    memory_pressure: float = 0.1
    context_pressure: float = 0.1

    @property
    def aggregate_pressure(self) -> float:
        return clamp01((1.0 - self.compute_budget + 1.0 - self.time_budget + self.memory_pressure + self.context_pressure) / 4.0)


@dataclass
class ResonanceTag:
    name: str
    texture: Texture
    intensity: float
    source_metrics: dict[str, float] = field(default_factory=dict)
    evidence_refs: list[str] = field(default_factory=list)

    def __post_init__(self) -> None:
        allowed = {"Hum", "Glitch", "Pull", "Kick", "Weld", "Bloom", "Echo", "Tangle", "Verge", "Static", "Drift", "Fold", "Stretch", "Thread", "Weight", "Lift", "Rebound"}
        textures = {"scattered", "looping", "deep", "expansive", "sustained", "distorted", "divergent"}
        if self.name not in allowed:
            raise ValueError(f"unknown resonance tag: {self.name}")
        if self.texture not in textures:
            raise ValueError(f"unknown resonance texture: {self.texture}")
        self.intensity = clamp01(self.intensity)
        if self.intensity > 0 and not self.source_metrics:
            raise ValueError("ResonanceTag requires source_metrics for nonzero intensity")


@dataclass
class ThoughtCandidate:
    candidate_id: str
    stream_source: str
    text: str
    mode: str
    evidence_refs: list[str] = field(default_factory=list)
    internal_state_drivers: dict[str, float] = field(default_factory=dict)
    predicted_effects: dict[str, float] = field(default_factory=dict)
    risk_score: float = 0.0
    uncertainty_score: float = 0.0
    resource_cost: float = 0.1
    self_report_claims: list[str] = field(default_factory=list)
    memory_write_recommendation: bool = False
    action_type: ActionType = "answer"


@dataclass
class HemisphereOutput:
    stream_name: str
    candidates: list[ThoughtCandidate]
    confidence: float
    uncertainty: float
    risk_score: float
    novelty_score: float
    resonance_tags: list[ResonanceTag]


@dataclass
class BridgeOutput:
    merged_candidates: list[ThoughtCandidate]
    conflicts: list[str]
    agreement_score: float
    contradiction_score: float
    novelty_delta: float
    evidence_gap: float
    hemispheric_tension: float
    resonance_tags: list[ResonanceTag]
    scratchpad_writes: list[str]


@dataclass
class WorkspaceCapsule:
    capsule_id: str
    source: str
    content: str
    priority: float
    confidence: float
    novelty: float
    risk: float
    state_affinity: float
    evidence_refs: list[str] = field(default_factory=list)
    resonance_tags: list[ResonanceTag] = field(default_factory=list)
    timestamp: float = field(default_factory=now_ts)
    candidate: ThoughtCandidate | None = None
    resource_cost: float = 0.1
    memory_relevance: float = 0.0


@dataclass
class WorkspaceState:
    cycle_id: int
    shortlist: list[WorkspaceCapsule]
    broadcast_packet: dict[str, Any]
    ignition: bool
    state_snapshot: dict[str, Any]
    trace: list[dict[str, Any]]
    overflow: list[WorkspaceCapsule] = field(default_factory=list)


@dataclass
class CandidateScore:
    truth_support: float
    goal_alignment: float
    memory_consistency: float
    risk_penalty: float
    uncertainty_penalty: float
    state_match: float
    resource_cost: float
    self_report_grounding: float
    reversibility: float
    kindness: float = 0.7
    logical_consistency: float = 1.0
    social_harmony: float = 0.7
    total_score: float = 0.0
    rejected: bool = False
    rejection_reasons: list[str] = field(default_factory=list)


@dataclass
class MemoryEpisode:
    episode_id: str
    timestamp: float
    input_summary: str
    workspace_winners: list[str]
    analytic_candidates: list[str]
    associative_candidates: list[str]
    selected_candidate: str
    rejected_candidates: list[str]
    internal_state_snapshot: dict[str, Any]
    resonance_tags: list[ResonanceTag]
    prediction_error: float
    outcome_score: float
    provenance: dict[str, Any] = field(default_factory=dict)
