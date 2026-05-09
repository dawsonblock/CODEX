"""Append-only event log for runtime event sourcing.

Each call to ``EventLog.append`` records a ``RuntimeEvent`` in memory and
flushes a JSON line to disk.  The log can be queried by ``cycle_id`` or
``event_type`` for replay and audit purposes.

Design principles
-----------------
* **Append-only** – no update or delete operations.
* **Disk-backed** – every event is durably written before the call returns.
* **Lightweight** – zero external dependencies; uses only stdlib json / pathlib.
"""
from __future__ import annotations

import json
import time
import uuid
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

_DEFAULT_PATH = "artifacts/events/runtime.evlog"


@dataclass
class RuntimeEvent:
    """A single structured event emitted by the runtime."""

    event_id: str
    cycle_id: int
    event_type: str
    timestamp: float
    payload: dict[str, Any] = field(default_factory=dict)

    def to_json(self) -> str:
        return json.dumps(asdict(self), default=str, sort_keys=True)

    @classmethod
    def from_json(cls, line: str) -> "RuntimeEvent":
        d = json.loads(line)
        return cls(**d)


class EventLog:
    """Append-only event log backed by a JSONL file.

    Parameters
    ----------
    path:
        File path for the JSONL log.  Parent directories are created
        automatically.  Pass ``None`` to keep events in-memory only (useful
        for unit tests).
    """

    def __init__(self, path: str | Path | None = _DEFAULT_PATH) -> None:
        self._events: list[RuntimeEvent] = []
        self._path: Path | None = None
        if path is not None:
            self._path = Path(path)
            self._path.parent.mkdir(parents=True, exist_ok=True)

    # ------------------------------------------------------------------
    # Write
    # ------------------------------------------------------------------

    def append(
        self,
        cycle_id: int,
        event_type: str,
        payload: dict[str, Any] | None = None,
    ) -> RuntimeEvent:
        """Record an event and flush it to disk.

        Parameters
        ----------
        cycle_id:
            The runtime cycle that produced this event.
        event_type:
            A short snake_case label such as ``"cycle_start"`` or
            ``"candidate_selected"``.
        payload:
            Arbitrary JSON-serialisable metadata for the event.

        Returns
        -------
        RuntimeEvent
            The newly created event (already stored).
        """
        evt = RuntimeEvent(
            event_id=f"ev-{uuid.uuid4().hex[:12]}",
            cycle_id=cycle_id,
            event_type=event_type,
            timestamp=time.time(),
            payload=payload or {},
        )
        self._events.append(evt)
        if self._path is not None:
            with self._path.open("a", encoding="utf-8") as fh:
                fh.write(evt.to_json() + "\n")
        return evt

    # ------------------------------------------------------------------
    # Read
    # ------------------------------------------------------------------

    def events(
        self,
        *,
        cycle_id: int | None = None,
        event_type: str | None = None,
    ) -> list[RuntimeEvent]:
        """Return a filtered snapshot of the in-memory log.

        Parameters
        ----------
        cycle_id:
            If provided, return only events for this cycle.
        event_type:
            If provided, return only events of this type.
        """
        result = self._events
        if cycle_id is not None:
            result = [e for e in result if e.cycle_id == cycle_id]
        if event_type is not None:
            result = [e for e in result if e.event_type == event_type]
        return list(result)

    def by_cycle(self, cycle_id: int) -> list[RuntimeEvent]:
        """Convenience wrapper: all events for *cycle_id*, in order."""
        return self.events(cycle_id=cycle_id)

    def __len__(self) -> int:
        return len(self._events)
