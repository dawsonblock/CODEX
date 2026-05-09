"""Archive backend protocol and future-integration stubs.

``ArchiveBackend`` defines the minimal interface that any long-term memory
archive must satisfy.  ``JsonlArchive`` in ``memory.jsonl_archive`` is the
default implementation.

``RealMemvidBackend`` is a placeholder that documents where a real Memvid
binary integration would live.  It raises ``NotImplementedError`` on every
call so tests that instantiate it fail loudly rather than silently using the
wrong backend.
"""
from __future__ import annotations

from typing import Any, Protocol, runtime_checkable

from .jsonl_archive import MemoryFrame


@runtime_checkable
class ArchiveBackend(Protocol):
    """Minimal protocol for an append-only memory archive."""

    def append_frame(
        self,
        text: str,
        *,
        frame_type: str = "episode",
        tags: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> MemoryFrame:
        """Append a new frame and return it."""
        ...

    def frames(self) -> list[MemoryFrame]:
        """Return all stored frames in insertion order."""
        ...

    def query(self, text: str, *, limit: int = 5, tags: list[str] | None = None) -> list[dict[str, Any]]:
        """Lexically query archive frames and return scored result dicts."""
        ...


class RealMemvidBackend:
    """Stub for a future real Memvid binary backend.

    Replace this class body with a real implementation once the ``memvid``
    package is available and pinned in ``pyproject.toml``.
    """

    def append_frame(self, text: str, **kwargs: Any) -> MemoryFrame:  # type: ignore[empty-body]
        raise NotImplementedError("RealMemvidBackend is not yet implemented; use JsonlArchive")

    def frames(self) -> list[MemoryFrame]:  # type: ignore[empty-body]
        raise NotImplementedError("RealMemvidBackend is not yet implemented; use JsonlArchive")

    def query(self, text: str, *, limit: int = 5, tags: list[str] | None = None) -> list[dict[str, Any]]:  # type: ignore[empty-body]
        raise NotImplementedError("RealMemvidBackend is not yet implemented; use JsonlArchive")
