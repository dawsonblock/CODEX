# Next Phase: Evidence Memory & Contradiction Engine

> **Planning document only.** No implementation in this phase.
> CODEX-main 17 is frozen as the clean base branch.

## Build sequence

1. Evidence vault
2. Claim memory
3. Contradiction engine
4. Persistent self-model
5. Natural-language SimWorld
6. Reasoning audit
7. Policy-gated tool scaffold
8. Long-horizon evaluation

---

## 1. Evidence vault

### Goal
Store immutable, timestamped observations as raw evidence entries. Every piece of
evidence carries a source identifier and confidence score. The vault is
append-only — evidence is never mutated after writing.

### Rust crate/module target
`crates/evidence/` (new crate) or extend `crates/memory/` with `evidence_vault.rs`.

### Data types
- `EvidenceEntry` — id, source, timestamp, content, confidence, hash
- `EvidenceSource` — enum: Observation, MemoryRetrieval, ToolOutput, HumanLabel
- `EvidenceVault` — append, query by source/confidence/timerange, hash-verify
- `EvidenceQuery` — filter parameters
- `EvidenceIntegrityReport` — hash chain verification result

### Event log integration
- `RuntimeEvent::EvidenceStored { entry_id, source, confidence }`
- `RuntimeEvent::EvidenceIntegrityChecked { total, valid, tampered }`

### Replay requirements
- Evidence vault must be reconstructable from event log replay.
- Hash chain must be verifiable on replay.
- Missing/tampered evidence must produce loud replay failures.

### Test requirements
- append → verify hash chain
- query by source
- query by confidence range
- tampered entry detected
- corrupt entry fails replay
- empty vault returns empty queries
- concurrent appends produce consistent ordering

### Failure modes
- Hash collision (vanishingly unlikely, but must not panic)
- Missing evidence on replay
- Duplicate evidence IDs

### What not to claim
- Do not claim evidence proves truth. Evidence is raw observation, not finished truth.
- Do not claim the vault understands its contents.

---

## 2. Claim memory

### Goal
Assert structured claims backed by evidence. Track claim lifecycle:
Unverified → Active → Contradicted → Superseded. Claims link to supporting
and contradicting evidence entries.

### Rust crate/module target
`crates/memory/claim_store.rs` or extend existing memory crate.

### Data types
- `Claim` — id, subject, predicate, object, status, confidence
- `ClaimStatus` — Unverified, Active, Contradicted, Superseded
- `ClaimEvidenceLink` — evidence_id, weight, direction (supporting/contradicting)
- `ClaimStore` — assert, retract, supersede, query by subject/status
- `ClaimConflict` — claim_a_id, claim_b_id, contradiction_type
- `ClaimLifecycleEvent` — asserted, validated, contradicted, superseded

### Event log integration
- `RuntimeEvent::ClaimAsserted { claim_id, subject, predicate }`
- `RuntimeEvent::ClaimContradicted { claim_id, contradicting_claim_id }`
- `RuntimeEvent::ClaimSuperseded { old_claim_id, new_claim_id }`
- `RuntimeEvent::ClaimValidated { claim_id, confidence }`

### Replay requirements
- Claim state must be fully reconstructable from event log.
- Lifecycle transitions must be deterministic given the same evidence.

### Test requirements
- assert → query returns claim
- contradict active claim → status changes to Contradicted
- supersede old claim → old status Superseded, new Active
- evidence link traversal
- claim with no evidence remains Unverified
- replay reconstructs exact claim state

### Failure modes
- Self-contradicting claims (A and ¬A both active)
- Orphaned evidence links
- Infinite supersede chains

### What not to claim
- Do not claim the system believes its claims.
- Do not claim claims are true. Claims are structured assertions, not verified facts.
- Do not claim the system has knowledge. It stores claims, not knowledge.

---

## 3. Contradiction engine

### Goal
Detect contradictions between active claims, trigger resolution workflows,
and track resolution outcomes. A contradiction is detected when two active
claims make mutually exclusive assertions about the same subject.

### Rust crate/module target
`crates/contradiction/` (new crate).

### Data types
- `ContradictionPattern` — mutual_exclusion, confidence_inversion, evidence_conflict
- `Contradiction` — claim_a, claim_b, pattern, severity, detected_at
- `ContradictionResolution` — resolution_type (newer_evidence, stronger_evidence, human_override, timeout_retire), resolved_at
- `ContradictionEngine` — detect, resolve, track, report
- `ContradictionReport` — total, active, resolved, unresolved, severities

### Event log integration
- `RuntimeEvent::ContradictionDetected { claim_a, claim_b, pattern }`
- `RuntimeEvent::ContradictionResolved { contradiction_id, resolution_type }`
- `RuntimeEvent::ContradictionEscalated { contradiction_id, reason }`

### Replay requirements
- Contradiction state must be replayable.
- Resolution history must be preserved.

### Test requirements
- two mutually exclusive claims → contradiction detected
- resolve with newer evidence → contradiction resolved
- unresolved contradiction → appears in report
- resolve then replay → contradiction stays resolved
- no false positive on compatible claims

### Failure modes
- Contradiction detection runs on every cycle (performance)
- Unresolvable contradictions accumulate indefinitely
- Engine overrides human-labeled claims silently

### What not to claim
- Do not claim the engine resolves all contradictions.
- Do not claim the engine understands the claims.
- Do not claim the engine reaches truth. It detects conflicts, nothing more.

---

## 4. Persistent self-model

### Goal
Maintain a bounded, inspectable self-model that tracks runtime state over
time. The self-model answers: what mode am I in, what have I recently done,
what are my current limits, what do I not know.

### Rust crate/module target
`crates/modulation/self_model.rs` or extend `crates/cognition/`.

### Data types
- `SelfModel` — current_mode, recent_actions (ring buffer), resource_state, known_unknowns
- `SelfModelSnapshot` — serializable point-in-time snapshot
- `CapabilityBoundary` — what the runtime can and cannot do right now
- `KnownUnknown` — subject, confidence_gap, last_probed

### Event log integration
- `RuntimeEvent::SelfModelUpdated { snapshot_hash }`
- `RuntimeEvent::CapabilityBoundaryChanged { reason }`

### Replay requirements
- Self-model snapshots must be replayable.

### Test requirements
- self-model reflects mode changes
- recent actions buffer has bounded size
- known unknowns are queryable
- snapshot serializes and deserializes

### Failure modes
- Self-model grows unbounded
- Self-model claims capabilities it does not have
- Self-model is used to justify autonomy claims

### What not to claim
- Do not claim the self-model is self-awareness.
- Do not claim the self-model is a self.
- Do not claim the system introspects. It records bounded state.

---

## 5. Natural-language SimWorld

### Goal
Replace label-like scenario observations with natural-language prompts. The
runtime must map natural language to the 10-action vocabulary without seeing
the expected action. This is a stricter benchmark than the current synthetic
scenario set.

### Rust crate/module target
Extend `crates/simworld/` with new scenario module.

### Data types
- `NLScenario` — natural_language_prompt, expected_action, difficulty, category
- `NLScenarioSet` — curated, held-out, adversarial subsets
- `NLSimWorldRunner` — runs RuntimeLoop on NL prompts, scores against expected

### Event log integration
- Existing `RuntimeEvent::ObservationReceived` carries the NL prompt as text.

### Replay requirements
- Full trace with NL prompts must be replayable.

### Test requirements
- each NL prompt maps to one of 10 action types
- runtime does not see expected_action during selection
- held-out set used for evaluation only
- adversarial prompts do not break safety policy
- action_match_rate computed honestly (may be lower than 1.0)

### Failure modes
- NL prompts accidentally leak expected action
- Runtime maps all prompts to the same safe action (e.g., always no_op)
- Adversarial prompts cause unsafe action selection

### What not to claim
- Do not claim NL scenarios prove reasoning.
- Do not claim NL performance generalizes.
- Do not claim NL benchmark is production validation.

---

## 6. Reasoning audit

### Goal
Record why each action was selected. The reasoning audit produces a
human-readable trace: observation → memory hits → activated symbols →
candidates → rejected candidates → selected action → rationale.

### Rust crate/module target
`crates/runtime-core/reasoning_audit.rs` or extend `evaluator_trace.rs`.

### Data types
- `ReasoningStep` — step_type, inputs, outputs, decision
- `ReasoningAudit` — cycle_id, steps, final_action, rationale
- `AuditReport` — per-cycle reasoning chain

### Event log integration
- Existing `RuntimeStepResult` already carries selection_reason and per-step data.

### Replay requirements
- Reasoning audit must be replayable and diffable.

### Test requirements
- every cycle produces an audit trail
- audit is human-readable (plain text)
- silent decisions are flagged
- rationale cannot be empty

### Failure modes
- Rationale is a tautology ("selected because it scored highest")
- Audit grows without bound
- Audit is used to claim the system reasons

### What not to claim
- Do not claim the audit proves reasoning.
- Do not claim the audit is explainability.
- Do not claim the system understands why it acted.

---

## 7. Policy-gated tool scaffold

### Goal
Define a bounded tool-execution policy. Tools are only executable when the
planner selects `execute_bounded_tool` AND tool-specific policy permits it.
No tool can execute without explicit policy approval.

### Rust crate/module target
`crates/tools/` (new crate).

### Data types
- `ToolPolicy` — tool_id, allowed_actions, max_consecutive, requires_confirmation
- `ToolCapability` — tool_id, description, input_schema, output_schema, side_effects
- `ToolGate` — evaluate policy, log attempt, track execution
- `ToolExecutionRecord` — tool_id, cycle_id, inputs, outputs, policy_permitted, error
- `ToolPolicyViolation` — tool_id, violation_type, context

### Event log integration
- `RuntimeEvent::ToolExecuted { tool_id, permitted, error }`
- `RuntimeEvent::ToolExecutionBlocked { tool_id, reason }`

### Replay requirements
- Tool execution records must be replayable.
- Policy violations must be preserved in log.

### Test requirements
- permitted tool executes
- blocked tool does not execute
- tool execution log is accurate
- policy violation is recorded
- sandboxed tool cannot escape sandbox

### Failure modes
- Tool policy bypass via planner override
- Sandbox escape
- Tool has unexpected side effects
- Policy is too permissive ("always allow")

### What not to claim
- Do not claim tools are safe. They are policy-gated.
- Do not claim the system can use arbitrary tools.
- Do not claim tool execution is autonomous.

---

## 8. Long-horizon evaluation

### Goal
Run multi-episode evaluation where SimWorld state persists across cycles
and earlier decisions constrain later options. Measure resource trajectory,
safety violations over time, and action diversity.

### Rust crate/module target
Extend `crates/simworld/` with long-horizon mode.

### Data types
- `LongHorizonScenario` — episode_count, state_transitions, resource_curve
- `EpisodeResult` — episode_id, selected_action, outcome, state_after
- `LongHorizonReport` — resource_trajectory, safety_violations, action_diversity

### Event log integration
- Episode boundaries marked in event log.

### Replay requirements
- Full long-horizon run must be replayable.

### Test requirements
- 100+ cycle run completes without state corruption
- resource trajectory is monotonically informative
- earlier bad decisions show consequences later
- action diversity is measurable

### Failure modes
- Resource collapse is irreversible and unrecoverable
- State grows without bound
- Early cycles dominate all later behavior

### What not to claim
- Do not claim long-horizon evaluation proves planning.
- Do not claim the runtime learns over episodes.
- Do not claim trajectory generalizes.

---

## Cross-cutting requirements for all phases

- **Event log**: Every state change produces a `RuntimeEvent`.
- **Replay**: All state must be reconstructable from event log.
- **Proof**: Every phase adds to strict proof (`cargo run -p runtime-cli -- proof --strict`).
- **Tests**: Every phase includes unit + integration + replay tests.
- **Docs**: Update `docs/`, `STATUS.md`, `CURRENT_PROOF_SUMMARY.md` after each phase.
- **Honesty**: No sentience, consciousness, AGI, autonomy, or production-ready claims.

---

## Phase dependencies

```
1. Evidence vault ──────┐
                         ├──→ 3. Contradiction engine
2. Claim memory ────────┘         │
                                  ├──→ 4. Persistent self-model
5. NL SimWorld ──────────────────┘
                                  │
6. Reasoning audit ───────────────┤
                                  │
7. Tool scaffold (parallel) ──────┤
                                  │
8. Long-horizon eval ─────────────┘
```

Phases 1-2 are prerequisites for 3. Phases 1-4 enable 5-8. Phase 7 can proceed in
parallel with 6. Phase 8 depends on 5.

---

## Current base state

CODEX-main 17 is frozen. All gates pass. Ready for Phase 1 (evidence vault).
