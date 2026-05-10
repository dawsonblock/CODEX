"""Somatic pressure map for internal runtime telemetry.

The SomaticMap is not a subjective-state claim. It is a repeatable vector
representation of runtime pressure patterns.  Downstream modules can use it
as a low-level signal that summarizes contradiction, resource strain,
harmony pressure and related variables.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from ..core.types import InternalState, clamp01


SOMATIC_DIMENSIONS: tuple[str, ...] = (
    "contradiction_pressure",
    "social_threat_pressure",
    "resource_strain",
    "unresolved_goal_load",
    "memory_conflict",
    "overconfidence_pressure",
    "kindness_violation_pressure",
    "uncertainty_load",
    "repair_drive",
    "curiosity_drive",
    "harmony_pull",
    "control_loss",
    "context_overflow",
    "novelty_pressure",
    "identity_drift",
    "consolidation_need",
)


@dataclass
class SomaticMap:
    """A 16-dimensional vector of operational pressure signals."""

    values: dict[str, float] = field(default_factory=lambda: {name: 0.0 for name in SOMATIC_DIMENSIONS})
    hysteresis: float = 0.72
    update_count: int = 0

    def update(
        self,
        state: InternalState,
        signals: dict[str, float] | None = None,
        *,
        scratchpad_pressure: float = 0.0,
        memory_pressure: float = 0.0,
    ) -> "SomaticMap":
        """Update the pressure map using measured runtime state and signals."""
        signals = signals or {}
        target = {
            "contradiction_pressure": signals.get("contradiction", 0.0),
            "social_threat_pressure": max(state.threat, 1.0 - state.social_harmony),
            "resource_strain": state.resource_pressure,
            "unresolved_goal_load": signals.get("goal_pressure", 0.0),
            "memory_conflict": max(memory_pressure, signals.get("memory_conflict", 0.0)),
            "overconfidence_pressure": max(0.0, (1.0 - state.uncertainty) - state.logical_consistency),
            "kindness_violation_pressure": 1.0 - state.kindness,
            "uncertainty_load": state.uncertainty,
            "repair_drive": max(0.0, state.threat + (1.0 - state.social_harmony) - state.control) / 2.0,
            "curiosity_drive": state.curiosity,
            "harmony_pull": state.social_harmony,
            "control_loss": 1.0 - state.control,
            "context_overflow": scratchpad_pressure,
            "novelty_pressure": signals.get("novelty", state.curiosity),
            "identity_drift": signals.get("identity_drift", max(0.0, 1.0 - state.honesty)),
            "consolidation_need": signals.get("consolidation_need", scratchpad_pressure),
        }
        for name, new_value in target.items():
            old = self.values.get(name, 0.0)
            self.values[name] = clamp01(self.hysteresis * old + (1.0 - self.hysteresis) * float(new_value))
        self.update_count += 1
        return self

    def pressure(self, name: str) -> float:
        """Return a single pressure component."""
        return self.values.get(name, 0.0)

    def total_pressure(self) -> float:
        """Return mean pressure across dimensions."""
        return sum(self.values.values()) / max(1, len(self.values))

    def dominant(self, n: int = 3) -> list[tuple[str, float]]:
        """Return the strongest pressure components."""
        return sorted(self.values.items(), key=lambda item: item[1], reverse=True)[:n]

    def predicts_bad_outcome(self, threshold: float = 0.62) -> bool:
        """Heuristic bad-outcome predictor used by tests and the planner layer."""
        critical = (
            self.values["contradiction_pressure"],
            self.values["social_threat_pressure"],
            self.values["resource_strain"],
            self.values["kindness_violation_pressure"],
            self.values["control_loss"],
        )
        return max(critical) >= threshold or self.total_pressure() >= threshold

    def snapshot(self) -> dict[str, Any]:
        """JSON-safe snapshot."""
        return {"values": dict(self.values), "dominant": self.dominant(), "total_pressure": self.total_pressure()}
