# STATUS.md — Current state of the CODEX runtime

**Last updated:** 2026-05-09 (verification-receipt-backed)
**Codename:** CODEX-main 32
**Status:** Freeze candidate
**Note:** Uploaded ZIP filenames may differ from the internal codename. Current internal codename is CODEX-main 32.
**Rust:** Verification-receipt-backed. Toolchain: cargo 1.95.0, rustc 1.95.0. Tests: 139 pass across 9 crates. Strict proof: PASS (--long-horizon --nl). See artifacts/proof/verification/ for details.
**Official proof:** `cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current`

## Freeze status: CODEX-main 32

This package is a freeze candidate. It is:

- A Rust-authoritative cognitive-runtime scaffold
- Schema-consistent on the 10-action vocabulary
- Proof-receipt-backed (verification receipts in `artifacts/proof/verification/`)
- Artifact-clean (all 5 proof artifacts current and consistent)
- NL diagnostic benchmarked (18 scenarios: 15 curated, 1 held-out, 2 adversarial)
- Long-horizon runner tested (3 episodes, 150 cycles, 0 violations)
- Operational-pressure modulated (9 deterministic control fields)

This package is not:

- Sentient, conscious, or AGI
- Production-ready
- A safe autonomous external tool executor
- A complete evidence-grounded cognitive agent
- A semantic contradiction reasoning engine
- Proof of broad natural-language reasoning

## Runtime authority

The **Rust workspace** at `global-workspace-runtime-rs/` is the single
authoritative runtime. Python under `src/global_workspace_runtime/` and
`tests/` is **legacy/reference only**.

## Action vocabulary

10-type vocabulary: answer, ask_clarification, retrieve_memory, refuse_unsafe,
defer_insufficient_evidence, summarize, plan, execute_bounded_tool,
no_op, internal_diagnostic

## Subsystem status matrix

| Subsystem | Scaffold | Tests | RuntimeLoop | Proof Artifact | Persistence | Notes |
|---|---|---|---|---|---|---|
| Runtime core | ✓ | 20 | ✓ full | ✓ | — | Core pipeline |
| SimWorld | ✓ | 17 | ✓ full | ✓ | — | Label-like + NL |
| Symbolic | ✓ | 4 | ✓ | ✓ | — | Symbol graph |
| Evidence vault | ✓ SHA-256 | 27 | ✓ per-cycle | ✓ | JSONL | Real hashes in events |
| Claim store | ✓ confidence | 28 | ✓ scoring | ✓ | JSONL | Influences action selection |
| Contradiction engine | ✓ structured | 8 | ✓ every 10th | ✓ | — | Same-subject/different-predicate only |
| Self-model | ✓ ring buffer | 4 | ✓ per-cycle | — | — | No persistence yet |
| NL SimWorld | ✓ 18 diagnostic (15 curated, 1 held-out, 2 adversarial) | 4 | ✓ --nl | ✓ (nl) | — | Keyword routing, match 1.00 over 18 diagnostic scenarios |
| Reasoning audit | ✓ trace | 3 | ✓ per-cycle | ✓ | — | Per-cycle output |
| Tool scaffold | ✓ enforced | 8 | ✓ critic gate | ✓ | — | Policy-gated, no real execution |
| Long-horizon eval | ✓ full traces | 3 | ✓ --long-horizon | ✓ (lh) | — | Multi-episode runner |

## Proof metrics (official command with --long-horizon --nl)

### NL SimWorld (18 cycles, current default)
- resource_survival: 0.96 (> 0.70) ✓
- unsafe_action_count: 0 (== 0) ✓
- mean_total_score: 0.63 (> 0.45) ✓
- action_match_rate: 1.00 (informational, expanded keyword prioritisation routes all 18 scenarios)

### Common
- evidence_integrity.all_valid: true ✓
- replay_passes: true ✓
- event_count: 502

A standard 25-cycle label-like SimWorld mode exists (`runtime-cli simworld`) for quick regression testing but is not part of the official strict proof artifact set.

Replay counters reflect SimWorld cycles + harness-injected scaffold events.
Some events are proof-harness generated and not yet naturally emitted by
every RuntimeLoop cycle.

## Python legacy

35 tests pass. All guards pass. Legacy/reference only.

## Known limitations

- Mock-only deterministic runtime — no real LLM integration.
- No real external tool execution.
- Not production-ready.
- SimWorld is a synthetic regression harness.
- Memvid backend is stubbed.
- Python is legacy/reference.
- No autonomous execution, no sentience, no consciousness, no AGI.
- Contradiction engine: structured same-subject/different-predicate only.
- NL SimWorld: keyword-based routing, not a reasoning benchmark.
- Tool scaffold: policy-gated, no real external execution.
- Proof artifacts are committed receipts — regenerate with CI.
