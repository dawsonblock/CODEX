"""Fast-path reactive policy for routine inputs."""
from __future__ import annotations
from dataclasses import dataclass


@dataclass
class ReactiveResult:
    handled: bool
    text: str
    reason: str


class ReactiveLayer:
    def try_handle(self, text: str) -> ReactiveResult:
        t = text.strip().lower()
        if t in {"hi", "hello", "hey"}:
            return ReactiveResult(True, "Ready. Send the task or question.", "routine greeting")
        if len(t) < 12 and any(w in t for w in ["thanks", "thank you"]):
            return ReactiveResult(True, "Noted.", "routine acknowledgement")
        return ReactiveResult(False, "", "requires workspace cycle")
