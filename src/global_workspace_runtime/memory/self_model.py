"""Self-model memory for runtime telemetry."""
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any


@dataclass
class SelfModel:
    cycles_run: int = 0
    episodes_recorded: int = 0
    last_state_snapshot: dict[str, Any] = field(default_factory=dict)
    telemetry_history: list[dict[str, Any]] = field(default_factory=list)

    def observe_telemetry(self, snapshot: dict[str, Any]) -> None:
        self.cycles_run += 1
        self.last_state_snapshot = snapshot
        self.telemetry_history.append(snapshot)
        self.telemetry_history = self.telemetry_history[-50:]

    def record_episode(self) -> None:
        self.episodes_recorded += 1
