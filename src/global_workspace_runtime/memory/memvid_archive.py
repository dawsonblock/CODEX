"""Deprecated shim — use :mod:`global_workspace_runtime.memory.jsonl_archive` instead.

This module is retained for backwards compatibility only. All functionality
has moved to ``jsonl_archive.JsonlArchive``.  Import the new name directly::

    from global_workspace_runtime.memory import JsonlArchive
"""
import warnings as _warnings

_warnings.warn(
    "memvid_archive is deprecated; import JsonlArchive from jsonl_archive instead.",
    DeprecationWarning,
    stacklevel=2,
)

from .jsonl_archive import JsonlArchive as MemvidArchive, MemoryFrame  # noqa: F401, E402

__all__ = ["MemvidArchive", "MemoryFrame"]

