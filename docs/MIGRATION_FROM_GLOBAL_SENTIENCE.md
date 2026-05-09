# Migration from Global_Sentience

CODEX is the authoritative base. Global_Sentience was used as a salvage source
only — useful modules and tests were recovered where they added value.

## What was salvaged from Global_Sentience

- **RuntimeEvent variants**: MemoryQueried, MemoryHitReturned, CandidateRejected,
  ArchiveCommitted, ContradictionDetected, ContradictionResolved, and symbolic
  event types were added to CODEX's event system.
- **RuntimeLoop structure**: The 8-stage pipeline concept (observation → memory →
  symbolic → candidates → critic → selection → action → archive) was adopted.
- **Memory module structure**: ArchiveBackend trait, JsonlArchiveBackend,
  MemvidBackend stub, Evidence, MemoryClaim, ClaimStatus, Contradiction,
  RetrievalPacket were implemented.
- **Python tests**: Preserved as legacy reference under `tests/`.

## What was rejected from Global_Sentience

- **Oracle scoring**: The Global_Sentience SimWorld evaluator used
  `scenario.expected_action` directly as the selected action. This was replaced
  with independent RuntimeLoop selection. Expected_action is used only for
  match-rate scoring.
- **Stale proof artifacts**: `pytest_26_passed.log` and
  `simworld_25_seed5_summary.json` claimed 26 tests and resources=0.0 —
  both were incorrect. Replaced with current artifacts.
- **Vendored Memvid**: Moved from `src/global_workspace_runtime/memory/memvid-main/`
  to `vendor/memvid-main/` so the Python package namespace contains no vendored
  Rust repo.
- **Speculative terminology**: Terms like "sentient", "consciousness", "mystical",
  "flame", "soul", "astral", "god" were removed from documentation. The SYMBOLISM
  directory with speculative PDF was quarantined to `legacy/symbolism/`.

## Why CODEX is the base

CODEX had the cleaner Rust workspace structure, working event log and replay
system, and already removed oracle behavior from the evaluator. Global_Sentience
had richer RuntimeLoop and memory module ideas that were merged in.

## Key structural changes

- `runtime/kernel/` is now legacy (marked with LEGACY.md).
- `global-workspace-runtime-rs/` is the single Rust authority.
- CI workflows point to `global-workspace-runtime-rs/`, not `memory/memvid-main/`.
- Proof artifacts are separated: `artifacts/proof/rust_authority/` and
  `artifacts/proof/python_legacy/`.
- The symbolic crate is real (14 modules), not stubs.
- Oracle guard tests verify non-oracle behavior.
- EvaluatorTrace captures per-cycle decision metadata.
