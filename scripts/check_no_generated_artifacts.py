#!/usr/bin/env python3
"""Fail if common generated artifacts are present in the repository tree."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _should_skip(path: Path) -> bool:
    rel = path.relative_to(ROOT)
    # Proof outputs are intentionally retained under artifacts/proof.
    return rel.parts[:2] == ("artifacts", "proof")


def _collect_matches() -> list[Path]:
    violations: list[Path] = []

    for path in ROOT.rglob("__pycache__"):
        if path.is_dir() and not _should_skip(path):
            violations.append(path)

    for path in ROOT.rglob("*.pyc"):
        if not _should_skip(path):
            violations.append(path)

    for path in ROOT.rglob("*.gwlog"):
        if not _should_skip(path):
            violations.append(path)

    return sorted(set(violations))


def main() -> int:
    violations = _collect_matches()
    if not violations:
        print("No generated artifacts detected.")
        return 0

    print("Generated artifacts detected. Clean before committing:")
    for path in violations:
        print(path.relative_to(ROOT))
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
