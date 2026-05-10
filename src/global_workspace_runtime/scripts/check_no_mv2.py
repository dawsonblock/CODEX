"""CI guard: assert no .mv2 file references in Python, docs, config, or CI files.

Scope: Python, Markdown, reST, TOML, YAML — excludes vendor/memvid-main/
(which is vendored upstream source and expected to contain .mv2 references).

Usage::

    python -m global_workspace_runtime.scripts.check_no_mv2

Exits 0 on success, 1 on any violation.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[3]
_EXTS = {".py", ".md", ".rst", ".toml", ".yaml", ".yml"}
_PATTERN = re.compile(r"\.mv2\b", re.IGNORECASE)
_EXCLUDE_DIRS = {"__pycache__", ".git", ".mypy_cache", ".pytest_cache", "node_modules", "memvid-main"}
_EXCLUDE_FILES = {"check_no_mv2.py"}  # guard script may reference the pattern it checks


def _iter_files(root: Path):
    for path in root.rglob("*"):
        if path.suffix not in _EXTS:
            continue
        if any(part in _EXCLUDE_DIRS for part in path.parts):
            continue
        if path.name in _EXCLUDE_FILES:
            continue
        yield path


def main() -> None:
    violations: list[str] = []
    for fpath in _iter_files(_ROOT):
        try:
            text = fpath.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for lineno, line in enumerate(text.splitlines(), start=1):
            if _PATTERN.search(line):
                violations.append(f"  {fpath.relative_to(_ROOT)}:{lineno}: {line.strip()}")

    if violations:
        print("FAIL: .mv2 references found:")
        print("\n".join(violations))
        sys.exit(1)

    print(f"PASS: no .mv2 references in Python/docs/config ({sum(1 for _ in _iter_files(_ROOT))} files scanned; vendor/memvid-main excluded)")


if __name__ == "__main__":
    main()
