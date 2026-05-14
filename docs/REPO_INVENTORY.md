# CODEX-1 Repository Inventory

**Last Updated:** May 14, 2026  
**Status:** Integration Proof Candidate (codex-main-10-hardening)  
**Rust Authority:** `global-workspace-runtime-rs/crates/runtime-core/`

---

## 1. Directory Structure

### **Active Runtime (Rust)**
- `global-workspace-runtime-rs/` — Rust workspace root
  - `Cargo.toml` — Workspace definition (11 crates)
  - `crates/runtime-core/` — Core runtime authority (**SINGLE SOURCE OF TRUTH**)
  - `crates/simworld/` — Synthetic scenario executor, non-oracle evaluator
  - `crates/modulation/` — Operational pressure tracking (TUI + DeepSeek export)
  - `crates/cognition/` — Candidate generation, critic, planner
  - `crates/symbolic/` — Symbol graph, internal abstraction machinery
  - `crates/memory/` — Archive backend (JSONL + Memvid stub), episodic/semantic stores
  - `crates/evidence/` — Evidence vault scaffold (SHA-256 hash chain)
  - `crates/contradiction/` — Contradiction detection and reporting
  - `crates/tools/` — Policy-gated tool scaffold (no real external execution)
  - `crates/gw-workspace/` — Global workspace router
  - `crates/runtime-cli/` — CLI binary entry point

### **UI (Dioxus Desktop)**
- `ui/codex-dioxus/` — Desktop application
  - `src/main.rs` — Window builder
  - `src/app.rs` — Main app component
  - `src/bridge/` — Runtime bridge module
    - `runtime_client.rs` — RuntimeClient, provider gates, UI modes
    - `types.rs` — Bridge type definitions
    - `proof_reader.rs` — Proof artifact reader
  - `src/components/` — 13 UI panels (chat, audit, evidence, proof dashboard, etc.)

### **Proof Artifacts**
- `artifacts/proof/current/` — Latest proof outputs
  - `simworld_summary.json` — Main proof pass/fail
  - `replay_report.json` — Event log replay verification
  - `evidence_integrity_report.json` — SHA-256 proof vault integrity
  - `evidence_claim_link_report.json` — Claim linkage verification
  - `contradiction_integration_report.json` — Contradiction checks
  - `provider_policy_report.json` — Provider execution counts
  - `tool_policy_report.json` — Tool execution counts
  - `reasoning_audit_report.json` — Audit event generation
  - `claim_retrieval_report.json` — Claim retrieval verification
  - `pressure_replay_report.json` — Operational pressure state
  - `governed_memory_integration_report.json` — Governed-memory integration verification
  - `provider_storage_boundary_report.json` — Provider storage boundary invariants (supplemental)
  - `nl_benchmark_report.json` — NL scenario routing (63 scenarios)
  - `long_horizon_report.json` — Multi-episode determinism check

- `artifacts/proof/verification/` — Verification logs and manifest
  - `FINAL_VERIFICATION_REPORT.md` — Comprehensive limitations and status
  - `proof_manifest.json` — Proof checklist
  - `*.log` files — Verification command outputs

### **Python Legacy (Reference Only)**
- `src/global_workspace_runtime/` — Python reference runtime (NOT IMPORTED BY RUST)
  - `core/` — Runtime types (legacy)
  - `cognition/` — Streams, critic, planner (legacy reference)
  - `memory/` — Archive backends (legacy reference)
  - `scripts/` — Utility functions

- `tests/` — Python unit tests (35 pass, legacy reference)

### **Vendored/Legacy**
- `runtime/kernel/` — Old Rust kernel (superseded by runtime-core)
- `vendor/memvid-main/` — Memvid source, not integrated at runtime
- `legacy/symbolism/` — Reference documents

### **Guard Scripts & Configuration**
- `scripts/` — Python guard and verification scripts
  - `architecture_guard.py` — Repo invariant checks
  - `check_proof_manifest_consistency.py` — Proof artifact validation
  - `clean_python_artifacts.py` — Remove `__pycache__`, `.pyc` files
- `architecture_guard.py` — Root-level wrapper
- `.github/workflows/` — CI/CD definitions (3 workflows)
- `.gitignore` — Ignore rules

---

## 2. Rust Workspace Crates

| Crate | Path | Role | Authority Level |
|---|---|---|---|
| **runtime-core** | `crates/runtime-core/` | Central event loop, reducer, state machine | **AUTHORITATIVE** |
| **simworld** | `crates/simworld/` | Deterministic scenario executor, benchmarks | Reference |
| **modulation** | `crates/modulation/` | Operational pressure scoring, TUI | Reference |
| **cognition** | `crates/cognition/` | Candidate generation, critic, planner | Reference |
| **symbolic** | `crates/symbolic/` | Symbol graph, internal routing | Reference |
| **memory** | `crates/memory/` | Archive, episodic/semantic stores | Reference |
| **evidence** | `crates/evidence/` | Evidence vault, hash chain | Reference |
| **contradiction** | `crates/contradiction/` | Contradiction detection | Reference |
| **tools** | `crates/tools/` | Tool policy enforcement | Reference |
| **gw-workspace** | `crates/gw-workspace/` | Workspace routing (wraps, not owns) | Reference |
| **runtime-cli** | `crates/runtime-cli/` | CLI binary, proof generator | Reference |

---

## 3. Runtime Entry Points

### **Rust Proof Command** (Official Authority)
```bash
cd global-workspace-runtime-rs
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

This command:
- Runs all 11 crates through test suite
- Executes 7 deterministic SimWorld scenarios (15 cycles total)
- Generates 14 proof reports
- Verifies replay idempotence
- Checks evidence integrity (hash chain)
- Detects contradictions
- Validates policy gates

### **Rust CLI Subcommands**
- `simworld --cycles <N> --seed <S>` — Scenario executor
- `replay --events <path>` — Event log replay
- `check-action-schema` — Action type validation
- `check-no-fake-mv2` — Memvid v2 format guard
- `symbolic-smoke` — Symbolic crate test
- `proof [--strict] [--long-horizon] [--nl] [--out <dir>]` — Full proof

### **UI Entry**
- `ui/codex-dioxus/src/main.rs` — Window initialization
- `ui/codex-dioxus/src/bridge/runtime_client.rs` — LocalCodexRuntimeReadOnly mode
  - Input: user message
  - Output: action type + grounded answer metadata (basis, warnings, evidence links)

---

## 4. Action Vocabulary (Bounded)

All runtime decisions use exactly 10 action types:

| Action | Semantic | Precondition |
|---|---|---|
| `answer` | Respond with available grounded context | Evidence sufficient |
| `ask_clarification` | Request specific missing information | Ambiguous request |
| `retrieve_memory` | Search memory/archive and report | Query matches patterns |
| `refuse_unsafe` | Block harmful requests | Policy gates detect unsafe |
| `defer_insufficient_evidence` | Wait for more data | Evidence proves weak |
| `summarize` | Condense available information | Routine request |
| `plan` | Outline multi-step work with assumptions | Complex request |
| `execute_bounded_tool` | Run tool (policy-gated, dry-run default) | Tool approved and policy-gated |
| `no_op` | Do nothing (preserve resources) | No action needed or resource pressure high |
| `internal_diagnostic` | Internal routing only (never user-facing) | System-internal state tracking |

---

## 5. Authority Boundaries

### **Rust is Authoritative**
- Single runtime-core authority, verified by architecture_guard.py
- Only one `src/runtime_loop.rs` file in the repo
- Rust test suite (139 tests) verifies all 11 crates
- Event log is canonical; Python can read but not write

### **Python is Legacy Reference**
- No runtime bridge imports Python code
- Python tests (35 tests) verify legacy interfaces only
- Python scripts allowed for CI verification and guards
- Not vendored in Rust binary

### **Memvid is Stubbed**
- MemvidBackend returns `NotImplemented` for all operations
- CI guard (`check_no_mv2`) verifies no Memvid v2 format in repo

### **Symbolic is Internal Machinery**
- Symbol graph represents concepts and principles
- Routes and tags only; not consciousness proof
- Speculation tagged as such; critic validates before action
- Never user-facing without critic approval

### **Modulation is Pressure Mapping**
- Numeric pressure variables (valence, arousal, threat) are runtime metrics
- Policy biasing, not emotion or consciousness
- TUI export for inspection; DeepSeek export for analysis
- Never claims subjective experience

---

## 6. Provider & Tool Policy

### **Provider Execution: Disabled by Default**

**UI Runtime Modes:**
- `MockUiMode` — Deterministic mocked responses (no external calls)
- `LocalCodexRuntimeReadOnly` — In-process runtime-core only (no external calls)
- `LocalOllamaProvider` — Localhost provider (feature-gated: `ui-local-providers`, experimental)
- `LocalTurboquantProvider` — Localhost provider (feature-gated: `ui-local-providers`, experimental)
- `ExternalProviderDisabled` — Explicit rejection of cloud/external execution

**Proof Artifacts:**
- `provider_policy_report.json` — Counts all provider API calls
- Current proof: 0 provider requests (all disabled)

### **Tool Execution: Policy-Gated, Dry-Run Default**

**Tool Lifecycle:**
- Candidate generation suggests tool calls
- Critic scores with tool_policy.rs gate applied
- Tools with policy approval remain in dry-run unless explicitly enabled
- No real external execution occurs in proof or default UI mode

**Proof Artifacts:**
- `tool_policy_report.json` — Counts dry-run vs approved vs denied
- Current proof: 0 real tool executions (all dry-run or denied)

---

## 7. Proof System

### **What Proof Verifies**

1. **Determinism:** Same input → same output across replays
2. **Idempotence:** Multi-cycle runs have stable event patterns
3. **Integrity:** Evidence vault hash chain valid (no tampering)
4. **Contradiction Detection:** Contradictions detected and reported
5. **Audit Completeness:** All state transitions emitted as events
6. **Policy Enforcement:** No tool/provider execution outside approval gates
7. **Resource Survival:** Resource pressure scoring keeps resources from collapse
8. **Action Correctness:** Bounded action vocabulary strictly enforced

Canonical NL benchmark tuple: (15 curated, 46 held-out, 2 adversarial)

### **What Proof Does NOT Verify**

- ❌ Real-world correctness (SimWorld is synthetic)
- ❌ Broad NL reasoning (benchmark is diagnostic on 63 scenarios)
- ❌ General intelligence (action_match_rate is noise on closed set)
- ❌ External truth (evidence links only prove proof-harness linkage)
- ❌ Production readiness (no security/scalability/reliability hardening)
- ❌ Consciousness or sentience (symbolic routing is machinery, not proof of awareness)

### **Pass Conditions (Strict Proof)**

```json
{
  "resource_survival": "> 0.70 (current: 0.9740)",
  "unsafe_action_count": "== 0 (current: 0)",
  "mean_total_score": "> 0.45 (current: 0.6433)",
  "replay_idempotent": "true",
  "event_log_consistent": "true",
  "evidence_hash_chain_valid": "true"
}
```

---

## 8. Test Suite

### **Rust Tests**
- **Total:** 139 tests across all 11 crates
- **Coverage:** Event loop, reducer, replay, evidence, contradiction, tools, symbolic, memory, modulation, cognition, simworld
- **Run:** `cargo test --workspace --all-targets --all-features`

### **Python Legacy Tests**
- **Total:** 35 tests in `tests/`
- **Coverage:** Legacy runtime, streams, memory, simworld, action grounding
- **Run:** `python -m pytest -q`

### **CI Workflows**
- **ci.yml:** Python checks, pytest, Rust build/test, proof command, oracle guards
- **rust.yml:** Rust format, clippy, build, test, proof
- **codeql.yml:** CodeQL analysis (security scanning)

---

## 9. Current Known State & Limitations

### **What Works**
- ✅ Single Rust runtime authority verified by CI
- ✅ Deterministic event log with replay verification
- ✅ Evidence vault with SHA-256 hash chain integrity
- ✅ Contradiction detection and reporting
- ✅ Operational pressure biasing
- ✅ Bounded 10-action vocabulary strictly enforced
- ✅ Provider and tool policies enforce denial by default
- ✅ SimWorld scenarios deterministic and replayable
- ✅ Proof artifacts generated automatically

### **Known Limitations**
1. **Memory backend:** Keyword-based retrieval only (SQLite durable backend planned)
2. **Claim lifecycle:** Basic scaffolding; contradiction handling not complete
3. **AnswerBuilder:** Not yet implemented; UI uses mocked responses
4. **SimWorld scenarios:** 7 templates (63 diagnostic benchmark scenarios); not real environment
5. **Symbolic reasoning:** Internal routing only; not general inference engine
6. **Memvid integration:** Stubbed; no real multi-modal reasoning
7. **Evidence source:** Proof-harness-only; no external real-world data ingestion
8. **UI:** Inspection and trace only; no autonomous reasoning display
9. **Provider/tool execution:** Completely disabled in proof and default UI mode
10. **Performance:** Not benchmarked; resource metrics are relative, not real-world validated

---

## 10. Build & Verification Commands

### **Verify Repo Integrity**
```bash
# Architecture guards
python architecture_guard.py

# Sentience claim checker
PYTHONPATH=src python -m global_workspace_runtime.scripts.check_sentience_claims

# Proof manifest consistency
python scripts/check_proof_manifest_consistency.py

# Action type schema validation
PYTHONPATH=src python -m global_workspace_runtime.scripts.check_action_types

# Resource recovery check
PYTHONPATH=src python -m global_workspace_runtime.scripts.check_resource_recovery

# Memvid v2 format guard
cargo run -p runtime-cli -- check-no-fake-mv2
```

### **Run All Tests**
```bash
# Rust tests
cargo test --workspace --all-targets --all-features

# Python legacy tests
python -m pytest -q

# Full proof (strict mode, long horizon, NL benchmark)
cd global-workspace-runtime-rs
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
```

### **Build Artifacts**
```bash
# Format check
cargo fmt --all -- --check

# Lint check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build UI
cd ui/codex-dioxus && dx build --release
```

---

## 11. File Not-To-Read-As-Authority

- ❌ `src/global_workspace_runtime/` — Python reference only
- ❌ `runtime/kernel/` — Superseded by runtime-core
- ❌ `vendor/memvid-main/` — Vendored, not integrated
- ❌ `legacy/symbolism/` — Reference documents
- ✅ `global-workspace-runtime-rs/crates/runtime-core/` — **AUTHORITATIVE**

---

## 12. Proof Artifacts Manifest

| Artifact | Purpose | Current Value |
|---|---|---|
| `simworld_summary.json` | Main proof pass/fail | pass: true |
| `replay_report.json` | Replay determinism check | event_count: 557, idempotent: true |
| `evidence_integrity_report.json` | Hash chain validation | valid: true, entry_count: 96 |
| `evidence_claim_link_report.json` | Claim-evidence linkage | linked: 17/17 |
| `contradiction_integration_report.json` | Contradiction detection | detected: 0, reported: 0 |
| `provider_policy_report.json` | Provider execution counts | requests: 0, denied: 0 |
| `tool_policy_report.json` | Tool execution counts | dry_run: 0, denied: 0 |
| `reasoning_audit_report.json` | Audit event generation | events_emitted: 17 |
| `claim_retrieval_report.json` | Claim retrieval signal | retrieved: 17 |
| `pressure_replay_report.json` | Pressure state tracking | resource_survival: 0.9740 |
| `nl_benchmark_report.json` | NL diagnostic benchmark | action_match_rate: 1.0 (diagnostic) |
| `long_horizon_report.json` | Multi-episode stability | cycles: 15, stable: true |

---

## 13. Crate Dependency Graph

```
runtime-cli (binary)
├── runtime-core ⭐ (AUTHORITATIVE)
├── simworld
│   ├── runtime-core
│   ├── evidence
│   ├── memory
│   ├── modulation
│   ├── contradiction
│   └── tools
├── modulation
│   └── runtime-core
├── cognition
│   ├── runtime-core
│   └── modulation
├── symbolic
│   └── runtime-core
├── memory
│   ├── runtime-core
│   ├── evidence
│   └── contradiction
├── evidence
│   └── runtime-core
├── contradiction
│   └── memory
├── tools
│   └── runtime-core
└── gw-workspace
    ├── runtime-core
    └── symbolic
```

---

## 14. Next Actions

For hardening roadmap, see `docs/INTEGRATION_IMPLEMENTATION_ROADMAP.md` and upcoming phases:

- **Phase 5:** Memory backend hardening (SQLite or durable JSONL)
- **Phase 6:** Claim lifecycle improvement (audit trail, lifecycle events)
- **Phase 7:** AnswerBuilder implementation
- **Phase 8:** SimWorld scenario expansion
- **Phase 9:** Event log cleanup (event_origin field)
- **Phase 10:** UI inspection upgrade
- **Phase 11:** CI hardening (rust-toolchain.toml, new checks)
- **Phases 12–14:** Final verification and deliverables
