# Architecture

## One runtime authority

`global-workspace-runtime-rs/crates/runtime-core/` is the single authoritative
runtime. No other crate contains a competing runtime loop. `gw-workspace` wraps
`runtime-core` but does not own the selection pipeline.

## Event flow

```
ObservationInput
  → memory retrieval (semantic store)
  → symbolic/context activation (symbolic crate)
  → candidate generation (action scoring)
  → critic/policy evaluation (rejection rules)
  → planner selection (policy-driven)
  → action execution (SimWorld or real environment)
  → state update (RuntimeState via reducer)
  → event log append (EventLog → .gwlog)
  → proof/replay output
```

## Crate responsibilities

- **runtime-core**: ActionType, RuntimeEvent, EventLog, RuntimeLoop, RuntimeState,
  reducer, replay.
- **simworld**: CooperativeSupportWorld, EvaluatorRun, Scorecard, EvaluatorTrace.
  Depends on runtime-core.
- **symbolic**: SymbolGraph, streams, blending, resonance, principles.
  Depends on runtime-core.
- **memory**: ArchiveBackend, JsonlArchiveBackend, MemvidBackend stub,
  SemanticMemory, claims, evidence. Depends on runtime-core.
- **cognition**: Critic, Planner, ThoughtCandidate.
  Depends on runtime-core, modulation.
- **modulation**: InternalState, SomaticMap.
  Depends on runtime-core.
- **gw-workspace**: Global workspace router, ignition detector.
  Depends on runtime-core.
- **runtime-cli**: CLI binary. Depends on all crates.

## Memory role

The memory crate stores:
- Raw evidence (observations, tool results)
- Claims (assertions about world state with evidence links)
- Archive entries (append-only JSONL frames)
- Semantic context (keyword-scored key-value store)
- Retrieval packets (hits + evidence + claims)

MemvidBackend is stubbed. JsonlArchiveBackend is the default.

## Symbolic role

The symbolic crate maintains a graph of concepts, principles, and blends.
It is internal abstraction machinery — it does not represent consciousness.
Symbolic output is speculative unless validated by the critic.

## Evaluator role

The SimWorld evaluator:
1. Picks a scenario from the template set.
2. Feeds the scenario observation into RuntimeLoop.
3. RuntimeLoop independently selects an action.
4. The evaluator applies the action to the world.
5. Scores are recorded. `expected_action` is used only for match-rate scoring.
6. One EvaluatorTrace per cycle, with full detail.

## Python status

Python code under `src/global_workspace_runtime/` and `tests/` is
**legacy/reference only**. It is not the authoritative runtime. No bridge
connects Python and Rust at runtime.
