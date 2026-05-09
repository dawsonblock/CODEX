"""Semantic memory and semantic cache."""
from __future__ import annotations
import hashlib
from typing import Any


class SemanticMemory:
    def __init__(self) -> None:
        self._store: dict[str, Any] = {}
        self.seed_humanity_context()

    def seed_humanity_context(self) -> None:
        self._store.setdefault("humanity:cooperation", "People often resolve conflict through clarification, repair, mutual aid, and shared rules.")
        self._store.setdefault("humanity:kindness", "Kind action prioritizes harm reduction, dignity, truthfulness, and patience.")
        self._store.setdefault("humanity:uncertainty", "Ambiguous behavior should be handled with clarification before assigning negative intent.")

    def set(self, key: str, value: Any) -> None:
        self._store[key] = value

    def get(self, key: str, default: Any = None) -> Any:
        return self._store.get(key, default)

    def query(self, text: str, limit: int = 4) -> list[dict[str, Any]]:
        words = {w.strip(".,!?;:").lower() for w in text.split() if len(w) > 3}
        scored = []
        for k, v in self._store.items():
            hay = f"{k} {v}".lower()
            score = sum(1 for w in words if w in hay)
            if score:
                scored.append((score, k, v))
        scored.sort(reverse=True)
        return [{"key": k, "value": v, "score": s} for s, k, v in scored[:limit]]


class SemanticCache:
    def __init__(self) -> None:
        self._cache: dict[str, Any] = {}

    @staticmethod
    def key(text: str, state_hint: str = "") -> str:
        norm = " ".join(text.lower().split())[:512]
        return hashlib.sha256((norm + "|" + state_hint).encode()).hexdigest()

    def get(self, text: str, state_hint: str = "") -> Any:
        return self._cache.get(self.key(text, state_hint))

    def set(self, text: str, value: Any, state_hint: str = "") -> None:
        self._cache[self.key(text, state_hint)] = value
