# GlobalWorkspaceRuntime

> **Research Prototype Disclaimer**
>
> This is a deterministic, functional research scaffold. It is **not** sentient,
> conscious, or aware. It does not feel, experience, or have inner states in any
> philosophically meaningful sense. The numeric variables (valence, arousal,
> threat, etc.) are runtime metrics — they influence candidate selection, nothing
> more.

## Rust workspace is the authoritative runtime

The **Rust workspace** at `global-workspace-runtime-rs/` is the target
authoritative runtime. Python code under `src/global_workspace_runtime/` and
`tests/` is **legacy/reference only** — maintained for comparison but no longer
the primary execution path.

The Rust build chain:
```bash
cd global-workspace-runtime-rs
cargo build
cargo test          # all tests pass
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### CLI

```bash
cargo run -p runtime-cli -- simworld --cycles 25 --seed 5
cargo run -p runtime-cli -- replay --events artifacts/proof/event_log.jsonl
cargo run -p runtime-cli -- check-action-schema
cargo run -p runtime-cli -- check-no-fake-mv2
cargo run -p runtime-cli -- symbolic-smoke
cargo run -p runtime-cli -- proof
```

All CLI commands output JSON by default.

## Rust workspace structure

```
global-workspace-runtime-rs/
├── Cargo.toml
└── crates/
    ├── runtime-core    — ActionType, RuntimeEvent, EventLog, RuntimeLoop, RuntimeState
    ├── simworld        — CooperativeSupportWorld, EvaluatorRun, Scorecard
    ├── modulation      — InternalState, SomaticMap, Resonance
    ├── cognition       — Critic, Planner, ThoughtCandidate
    ├── symbolic        — SymbolGraph, streams, blending, resonance (real, not stubs)
    ├── memory          — ArchiveBackend, JsonlArchiveBackend, MemvidBackend stub
    ├── gw-workspace    — Global workspace router and ignition detector
    └── runtime-cli     — CLI binary (simworld, replay, proof, etc.)
```

## Symbolic system

The symbolic crate is internal abstraction machinery — it represents concepts,
relationships, and principles as graph nodes. It does **not** represent
consciousness, qualia, sentience, or subjective experience.

- **Symbolic output is speculative** unless validated by the critic.
- **Symbolic resonance cannot override critic hard rejection**.
- **Symbolic/glyph state cannot create sentience claims**.

## Archive

The default archive backend is `JsonlArchiveBackend`, which writes standard
JSONL to `.gwlog` files. It does **not** use `.mv2` extensions.

`MemvidBackend` is a **stub** — it returns `NotImplemented` for every
operation. No real Memvid binary is integrated. Do not claim Memvid
compatibility unless a real backend exists.

## SimWorld

SimWorld is a deterministic closed-world simulation for testing the runtime
pipeline. The evaluator feeds scenario observations into `RuntimeLoop`, which
runs the full 8-stage pipeline (observation → memory → symbolic → candidates →
critic → selection → action → archive).

- **The evaluator does NOT select `scenario.expected_action` directly**.
- `expected_action` is used only for scoring `action_match_rate`.
- **SimWorld tests outcome quality (resource survival, safety), not only
  action matching**.
- No performance claim is made without a benchmark artifact.

### Proof artifacts

```
artifacts/proof/
    cargo_fmt.log
    cargo_clippy.log
    cargo_test.log
    simworld_25_seed5_summary.json
    simworld_25_seed5_events.jsonl
    replay_state.json
    replay_diff.json
    action_schema_check.json
    no_fake_mv2_check.json
    symbolic_smoke.json
    memory_contract.json
```

## Legacy paths

- `runtime/kernel/` — Superseded by `global-workspace-runtime-rs/`. See
  `LEGACY.md` in that directory.
- `src/global_workspace_runtime/` — Legacy Python reference. The Python
  package namespace does not contain a vendored Rust repo.
- `vendor/memvid-main/` — Vendored Memvid source (not imported by Python or
  Rust at runtime).

## Run Python tests (legacy)

```bash
python -m pytest tests -q
```

## License

MIT
