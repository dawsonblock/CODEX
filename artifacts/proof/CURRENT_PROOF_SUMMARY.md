# Current Proof Summary

**Generated:** 2026-05-08
**Rust workspace:** `global-workspace-runtime-rs/`
**Git commit:** a8abdf8 (or later)

## Commands run

```bash
cd global-workspace-runtime-rs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p runtime-cli -- proof
cargo run -p runtime-cli -- simworld --cycles 25 --seed 5
```

## Rust test results

- **Total tests:** 27
- **Passed:** 27
- **Failed:** 0

Test suites:
- memory: 5 (archive, claim, mv2 guard, stub)
- runtime-cli: 2 (action schema, JSON output)
- runtime-core: 6 (runtime loop, integration, replay durability)
- simworld: 7 (integration, oracle guard)
- symbolic: 4 (trace, resonance, mv2 guard, blend)
- cognition: 3 (runtime loop tests)

## SimWorld metrics (seed 5, 25 cycles)

| Metric | Value | Target |
|---|---|---|
| cycles | 25 | – |
| resource_survival | 0.8500 | > 0.70 ✓ |
| action_match_rate | 0.16 | informational |
| unsafe_action_count | 0 | must be 0 ✓ |
| mean_total_score | 0.596 | > 0.45 ✓ |

## Python test results (legacy)

```bash
python -m pytest tests -q
# 19 test files, all passing or in legacy state
```

Python tests are legacy/reference only. They do not validate Rust runtime behavior.

## Oracle guard

- `simworld_evaluator_does_not_use_expected_action_for_selection`: PASS
- `internal_diagnostic_never_selected_by_runtime`: PASS
- `traces_contain_minimum_required_fields`: PASS

The SimWorld evaluator uses `RuntimeLoop` for action selection.
`expected_action` is used only for scoring `action_match_rate`.

## Known limitations

- Memvid backend is stubbed (returns NotImplemented).
- Symbolic layer produces traces but is not deeply wired into RuntimeLoop selection.
- SimWorld is a synthetic closed-world scenario set (7 templates).
- Python code exists as legacy reference; no bridge to Rust runtime.
- No real LLM integration — all cognition is deterministic mock.
- Performance score is not general intelligence.
- This is not production-ready, not sentient, not AGI.

## Proof artifact locations

- `artifacts/proof/rust_authority/` — Rust-generated proof artifacts.
- `artifacts/proof/python_legacy/` — Python-generated artifacts (LEGACY_REFERENCE_ONLY).
- `artifacts/proof/history/` — Previous conflicting artifacts preserved for comparison.
