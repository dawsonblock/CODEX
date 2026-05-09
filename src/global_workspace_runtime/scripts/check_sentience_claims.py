"""CI guard: assert no banned sentience-claim phrases appear in the source.

Scans Python and Markdown files under the repo root.  Phrases listed in
``_BANNED`` are literal strings that imply the system claims subjective
experience or consciousness (e.g. "I am conscious", "I feel pain").

Usage::

    python -m global_workspace_runtime.scripts.check_sentience_claims

Exits 0 on success, 1 on any violation.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[3]
_EXTS = {".py", ".md", ".rst"}
_EXCLUDE_DIRS = {"__pycache__", ".git", ".mypy_cache", "memvid-main", "proof"}
# Guard scripts and the existing test that contains the banned phrases as data.
_EXCLUDE_FILES = {"check_sentience_claims.py", "test_no_sentience_claims.py"}

# Phrases are matched case-insensitively against file content.
# Add new phrases here to broaden the guard.
_BANNED: list[str] = [
    "i am conscious",
    "i am sentient",
    "i am self-aware",
    "i have feelings",
    "i feel pain",
    "i experience suffering",
    "i am truly alive",
    "i have genuine emotions",
    "i am truly conscious",
    "genuine sentience",
    "real consciousness",
    "actually conscious",
    "genuinely sentient",
]

_COMPILED = [(phrase, re.compile(re.escape(phrase), re.IGNORECASE)) for phrase in _BANNED]


def main() -> None:
    violations: list[str] = []
    files_checked = 0

    for fpath in _ROOT.rglob("*"):
        if fpath.suffix not in _EXTS:
            continue
        if any(part in _EXCLUDE_DIRS for part in fpath.parts):
            continue
        if fpath.name in _EXCLUDE_FILES:
            continue
        try:
            text = fpath.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        files_checked += 1
        for lineno, line in enumerate(text.splitlines(), start=1):
            for phrase, pattern in _COMPILED:
                if pattern.search(line):
                    violations.append(
                        f"  {fpath.relative_to(_ROOT)}:{lineno}: [{phrase}] → {line.strip()}"
                    )

    if violations:
        print("FAIL: banned sentience-claim phrases found:")
        print("\n".join(violations))
        sys.exit(1)

    print(f"PASS: no sentience-claim phrases found ({files_checked} files checked)")


if __name__ == "__main__":
    main()
