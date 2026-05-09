"""JSON trace recording."""
from __future__ import annotations
import json, time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any


@dataclass
class TraceEvent:
    cycle_id: int
    phase: str
    option: str
    event_type: str
    payload: dict[str, Any]
    timestamp: float = 0.0

    def __post_init__(self) -> None:
        if not self.timestamp:
            self.timestamp = time.time()

    def to_json(self) -> str:
        return json.dumps(asdict(self), default=str, sort_keys=True)


class TraceRecorder:
    def __init__(self, path: str | Path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.events: list[TraceEvent] = []

    def emit(self, event: TraceEvent) -> None:
        self.events.append(event)
        with self.path.open("a", encoding="utf-8") as f:
            f.write(event.to_json() + "\n")

    def tail(self, n: int = 10) -> list[TraceEvent]:
        return self.events[-n:]
