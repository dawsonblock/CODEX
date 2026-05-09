"""Small performance helpers for latency instrumentation."""
from __future__ import annotations

import time
from dataclasses import dataclass, field


@dataclass
class CycleTimer:
    """Collect named timing spans for one runtime cycle."""

    spans: dict[str, float] = field(default_factory=dict)
    _starts: dict[str, float] = field(default_factory=dict)

    def start(self, name: str) -> None:
        self._starts[name] = time.perf_counter()

    def stop(self, name: str) -> float:
        start = self._starts.pop(name, None)
        if start is None:
            raise KeyError(f"timer span not started: {name}")
        elapsed = time.perf_counter() - start
        self.spans[name] = self.spans.get(name, 0.0) + elapsed
        return elapsed

    def summary(self) -> dict[str, float]:
        return dict(self.spans)
