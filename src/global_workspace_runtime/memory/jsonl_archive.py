"""Portable append-only JSONL memory archive.

This module provides a small local adapter for long-term archival: portable,
append-only, timestamped frames that can be queried and rewound for inspection.
It does not require any external package, so tests remain offline and
deterministic.

The archive writes standard JSONL to `.gwlog` files.  If a real Memvid backend
is needed, implement the ``ArchiveBackend`` protocol defined in
``memory.archive_backend`` and swap it in at the ``JsonlArchive`` construction
site.
"""
from __future__ import annotations

import hashlib
import json
import time
import uuid
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

from ..core.types import MemoryEpisode


@dataclass(frozen=True)
class MemoryFrame:
    """Immutable archival frame."""

    frame_id: str
    timestamp: float
    frame_type: str
    text: str
    tags: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)
    parent_hash: str = ""
    sha256: str = ""


def _hash_payload(payload: dict[str, Any]) -> str:
    encoded = json.dumps(payload, sort_keys=True, ensure_ascii=False).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


class JsonlArchive:
    """Append-only JSONL archive with lexical retrieval.

    Frames are written as newline-delimited JSON to a ``.gwlog`` file.
    Each frame carries a SHA-256 hash chained from the previous frame so the
    sequence can be integrity-checked without a database.
    """

    DEFAULT_PATH = "artifacts/memory/runtime.gwlog"

    def __init__(self, path: str | Path = DEFAULT_PATH) -> None:
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._last_hash = self._load_last_hash()

    def _load_last_hash(self) -> str:
        if not self.path.exists():
            return ""
        last = ""
        for line in self.path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                last = json.loads(line).get("sha256", last)
            except json.JSONDecodeError:
                continue
        return last

    def append_frame(
        self,
        text: str,
        *,
        frame_type: str = "episode",
        tags: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> MemoryFrame:
        """Append and return an immutable frame."""
        payload = {
            "frame_id": f"frame-{uuid.uuid4().hex[:12]}",
            "timestamp": time.time(),
            "frame_type": frame_type,
            "text": text,
            "tags": tags or [],
            "metadata": metadata or {},
            "parent_hash": self._last_hash,
        }
        payload["sha256"] = _hash_payload(payload)
        frame = MemoryFrame(**payload)
        with self.path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(frame), ensure_ascii=False) + "\n")
        self._last_hash = frame.sha256
        return frame

    def append_episode(self, episode: MemoryEpisode) -> MemoryFrame:
        """Append an episode summary frame."""
        text = (
            f"Input: {episode.input_summary}\n"
            f"Selected: {episode.selected_candidate}\n"
            f"Prediction error: {episode.prediction_error:.3f}"
        )
        tags = [tag.name for tag in episode.resonance_tags]
        metadata = {
            "episode_id": episode.episode_id,
            "outcome_score": episode.outcome_score,
            "workspace_winner_count": len(episode.workspace_winners),
        }
        return self.append_frame(text, frame_type="episode", tags=tags, metadata=metadata)

    def frames(self) -> list[MemoryFrame]:
        """Return all frames in order."""
        if not self.path.exists():
            return []
        out: list[MemoryFrame] = []
        for line in self.path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            out.append(MemoryFrame(**json.loads(line)))
        return out

    def query(self, text: str, *, limit: int = 5, tags: list[str] | None = None) -> list[dict[str, Any]]:
        """Lexically query archive frames and return scored dicts."""
        words = {w.strip(".,!?;:()[]{}\"").lower() for w in text.split() if len(w) > 3}
        required_tags = set(tags or [])
        scored: list[tuple[float, MemoryFrame]] = []
        for frame in self.frames():
            if required_tags and not required_tags.intersection(frame.tags):
                continue
            hay = f"{frame.frame_type} {' '.join(frame.tags)} {frame.text}".lower()
            lexical = sum(1 for word in words if word in hay)
            tag_bonus = 0.25 * len(required_tags.intersection(frame.tags))
            if lexical or tag_bonus:
                scored.append((lexical + tag_bonus, frame))
        scored.sort(key=lambda item: (item[0], item[1].timestamp), reverse=True)
        return [
            {
                "type": "archive_frame",
                "score": score,
                "frame_id": frame.frame_id,
                "frame_type": frame.frame_type,
                "text": frame.text,
                "tags": frame.tags,
                "timestamp": frame.timestamp,
                "sha256": frame.sha256,
            }
            for score, frame in scored[:limit]
        ]

    def rewind(self, *, frame_id: str | None = None, timestamp: float | None = None) -> list[MemoryFrame]:
        """Return frames up to a target frame or timestamp for inspection."""
        frames = self.frames()
        if frame_id is not None:
            out: list[MemoryFrame] = []
            for frame in frames:
                out.append(frame)
                if frame.frame_id == frame_id:
                    break
            return out
        if timestamp is not None:
            return [frame for frame in frames if frame.timestamp <= timestamp]
        return frames


# ---------------------------------------------------------------------------
# Backwards-compatibility alias.  New code should use JsonlArchive directly.
# ---------------------------------------------------------------------------
MemvidArchive = JsonlArchive  # deprecated — use JsonlArchive
