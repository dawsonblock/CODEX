#!/usr/bin/env python3
"""Remove generated artifacts that must not be committed or left after CI.

Safe paths (never removed):
  - artifacts/proof/current/
  - artifacts/proof/verification/

Removed paths:
  - __pycache__/ directories
  - *.pyc files
  - .pytest_cache/
  - .mypy_cache/
  - .ruff_cache/
  - artifacts/memory/*.gwlog
  - any *.gwlog outside artifacts/proof/
  - artifacts/test_traces/
  - artifacts/traces/

Usage:
  python3 scripts/clean_generated_artifacts.py          # clean mode
  python3 scripts/clean_generated_artifacts.py --check  # report mode (non-destructive)
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Subtrees that must never be touched.
SAFE_PREFIXES = (
    ROOT / "artifacts" / "proof",
)

# Whole directories to remove if they exist directly.
WHOLE_DIRS = [
    ROOT / "artifacts" / "memory",
    ROOT / "artifacts" / "test_traces",
    ROOT / "artifacts" / "traces",
]


def _is_safe(path: Path) -> bool:
    for prefix in SAFE_PREFIXES:
        try:
            path.relative_to(prefix)
            return True
        except ValueError:
            pass
    return False


def _collect() -> tuple[list[Path], list[Path]]:
    """Return (dirs_to_remove, files_to_remove)."""
    dirs: list[Path] = []
    files: list[Path] = []

    # __pycache__ dirs
    for p in ROOT.rglob("__pycache__"):
        if p.is_dir() and not _is_safe(p):
            dirs.append(p)

    # .pytest_cache / .mypy_cache / .ruff_cache at any level
    for name in (".pytest_cache", ".mypy_cache", ".ruff_cache"):
        for p in ROOT.rglob(name):
            if p.is_dir() and not _is_safe(p):
                dirs.append(p)

    # Whole artifact dirs (if present)
    for d in WHOLE_DIRS:
        if d.exists() and d.is_dir() and not _is_safe(d):
            dirs.append(d)

    # *.pyc files
    for p in ROOT.rglob("*.pyc"):
        if not _is_safe(p):
            files.append(p)

    # *.gwlog files outside artifacts/proof/
    for p in ROOT.rglob("*.gwlog"):
        if not _is_safe(p):
            files.append(p)

    return sorted(set(dirs)), sorted(set(files))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Report what would be removed without removing anything.",
    )
    args = parser.parse_args(argv)

    dirs, files = _collect()

    if args.check:
        if not dirs and not files:
            print("clean_generated_artifacts: nothing to remove.")
            return 0
        print("clean_generated_artifacts --check: would remove:")
        for d in dirs:
            print(f"  dir  {d.relative_to(ROOT)}")
        for f in files:
            print(f"  file {f.relative_to(ROOT)}")
        return 1

    removed_dirs = 0
    removed_files = 0

    for d in dirs:
        shutil.rmtree(d, ignore_errors=True)
        removed_dirs += 1

    for f in files:
        try:
            f.unlink()
            removed_files += 1
        except OSError:
            pass

    print(
        f"clean_generated_artifacts: removed {removed_dirs} dirs, {removed_files} files."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
