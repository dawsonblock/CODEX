---
name: feature-implementation-with-tests-and-docs
description: Workflow command scaffold for feature-implementation-with-tests-and-docs in CODEX.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /feature-implementation-with-tests-and-docs

Use this workflow when working on **feature-implementation-with-tests-and-docs** in `CODEX`.

## Goal

Implements a new feature, adds corresponding tests, and updates documentation.

## Common Files

- `global-workspace-runtime-rs/crates/*/src/*.rs`
- `global-workspace-runtime-rs/crates/*/tests/*.rs`
- `docs/ARCHITECTURE.md`
- `docs/LIMITATIONS.md`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Implement feature logic in relevant src files.
- Add or update unit/integration tests in the corresponding tests/ directory.
- Update documentation files (e.g., ARCHITECTURE.md, LIMITATIONS.md) to reflect the new feature or changes.

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.