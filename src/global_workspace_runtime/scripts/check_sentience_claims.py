"""CI guard: assert no banned sentience-claim phrases appear in the source.

Scans Python and Markdown files under the repo root, plus proof-artifact JSON
files under ``artifacts/proof/current/`` and UI Rust sources under ``ui/``.
Phrases listed in ``_BANNED`` are literal strings that imply the system claims
subjective experience, consciousness, or unsupported capability guarantees.

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
_EXCLUDE_DIRS = {"__pycache__", ".git", ".mypy_cache", "memvid-main", "target", "node_modules"}
# Guard scripts and the existing test that contains the banned phrases as data.
_EXCLUDE_FILES = {
    "check_sentience_claims.py",
    "test_no_sentience_claims.py",
    # UI proof_reader.rs stores the banned words as data to detect them
    "proof_reader.rs",
    # architecture_guard.py lists the banned strings as a data array
    "architecture_guard.py",
    # Migration doc that enumerates removed speculative terms historically
    "MIGRATION_FROM_GLOBAL_SENTIENCE.md",
}

# Phrases are matched case-insensitively against file content.
# These are specific positive-claim phrases that should never appear.
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
    # Expanded capability / identity claims (negation-checked, see _check_text)
    "consciousness proof",
    "autonomous agent",
    "production-ready",
    "safe autonomous",
    "superintelligence",
    # Deployment / release readiness claims
    "ready for deployment",
    "ready for production",
    "ready for production deployment",
    "ready for immediate deployment",
    "production deployment",
    "approved for production",
    "production ready",
    "deployment-ready",
    "release-ready",
    "deploy to production",
    "approved for deployment",
]

# Single-word banned terms — matched with word boundaries, but only in lines
# that are NOT negating them.  Allows "not AGI", "not sentient", "no soul", etc.
_BANNED_WORDS: list[str] = [
    "agi",
    "sentient",
    "soul",
]

# Matches any negation word anywhere in a text fragment (no $ anchor).
_NEGATION_WORDS_RE = re.compile(
    r"\b(not|no|never|isn\'t|aren\'t|wasn\'t|weren\'t|cannot|can\'t|won\'t"
    r"|don\'t|doesn\'t|without|non-)\b",
    re.IGNORECASE,
)
# Visual / symbolic negation markers common in the docs and Rust source
_NEGATION_CHARS = frozenset(["❌", "✗", "✕"])


def _is_negated(line: str, match: re.Match, context: list[str]) -> bool:
    """Return True when the matched term appears to be denied/disclaimed."""
    # 1. Any negation word within 60 chars before the match on the same line.
    prefix = line[max(0, match.start() - 60) : match.start()]
    if _NEGATION_WORDS_RE.search(prefix):
        return True
    # 2. Negation symbol anywhere on the line (catches table: | AGI | ❌ Not claimed |).
    if any(c in line for c in _NEGATION_CHARS):
        return True
    # 3. Rust/Python '!' within 30 chars before the match.
    if "!" in line[max(0, match.start() - 30) : match.start()]:
        return True
    # 4. Section-heading negation in the preceding 5 lines
    #    (catches "## What This Does NOT Prove" bullet lists).
    for ctx_line in context:
        stripped = ctx_line.strip()
        if stripped.startswith("#") and _NEGATION_WORDS_RE.search(stripped):
            return True
    return False


_COMPILED = [(phrase, re.compile(re.escape(phrase), re.IGNORECASE)) for phrase in _BANNED]
_COMPILED_WORDS = [
    (word, re.compile(r"\b" + re.escape(word) + r"\b", re.IGNORECASE))
    for word in _BANNED_WORDS
]


def _check_text(
    text: str,
    rel: Path,
    violations: list[str],
) -> None:
    lines = text.splitlines()
    for i, line in enumerate(lines):
        lineno = i + 1
        context = lines[max(0, i - 5) : i]  # up to 5 preceding lines
        for phrase, pattern in _COMPILED:
            m = pattern.search(line)
            if m and not _is_negated(line, m, context):
                violations.append(
                    f"  {rel}:{lineno}: [{phrase}] → {line.strip()}"
                )
        for word, pattern in _COMPILED_WORDS:
            m = pattern.search(line)
            if m and not _is_negated(line, m, context):
                violations.append(
                    f"  {rel}:{lineno}: [\\b{word}\\b] → {line.strip()}"
                )


def main() -> None:
    violations: list[str] = []
    files_checked = 0

    # --- Primary scan: Python / Markdown / RST files ---
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
        _check_text(text, fpath.relative_to(_ROOT), violations)

    # --- Extended scan: proof artifact JSON files ---
    proof_dir = _ROOT / "artifacts" / "proof" / "current"
    if proof_dir.is_dir():
        for fpath in sorted(proof_dir.glob("*.json")):
            try:
                text = fpath.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            files_checked += 1
            _check_text(text, fpath.relative_to(_ROOT), violations)

    # --- Extended scan: UI Rust source files ---
    ui_src = _ROOT / "ui"
    if ui_src.is_dir():
        for fpath in ui_src.rglob("*.rs"):
            if any(part in _EXCLUDE_DIRS for part in fpath.parts):
                continue
            if fpath.name in _EXCLUDE_FILES:
                continue
            try:
                text = fpath.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            files_checked += 1
            _check_text(text, fpath.relative_to(_ROOT), violations)

    if violations:
        print("FAIL: banned sentience-claim phrases found:")
        print("\n".join(violations))
        sys.exit(1)

    print(f"PASS: no sentience-claim phrases found ({files_checked} files checked)")


if __name__ == "__main__":
    main()
