# CODEX-main 32 — UI Provider Boundary Repair
## FINAL_VERIFICATION_REPORT.md

---

## 1. Package Identity

- **Internal codename:** CODEX-main 32
- **Uploaded filename:** CODEX-main 38 (internal codename unchanged)
- **Status:** Integration proof candidate — NOT final freeze
- **Pass type:** UI Provider Boundary Repair
- **Report date:** 2026-05-11

---

## 2. Provider Policy Chosen

### **Option B — Feature-Gated Local Providers**

Local provider modes (`LocalOllamaProvider`, `LocalTurboquantProvider`) are compiled only when
`--features ui-local-providers` is explicitly passed to Cargo.

**Default build behavior:**
- Zero provider HTTP code paths exist in the default binary.
- `localhost:11434` is absent from the default binary.
- `reqwest` and `futures-util` are `optional = true` dependencies, absent unless feature enabled.
- `RuntimeBridgeMode` cycles only through: `MockUiMode → LocalCodexRuntimeReadOnly → ExternalProviderDisabled`.
- Settings panel shows: "Provider execution disabled in this build (default CODEX build)."

**When `--features ui-local-providers` is active:**
- Calls are localhost-only (no external/cloud endpoints).
- First use requires explicit user approval (gate must be unlocked in Settings).
- Provider output is labeled: `"Local provider draft — not CODEX runtime authority"`.
- Provider output cannot: execute tools, write evidence/claims, or override `selected_action`.
- Failure returns a clean UI error; no silent fallback.
- Cloud and external provider request counts remain 0.

---

## 3. Files Changed

### Build / Configuration
| File | Change |
|------|--------|
| `ui/codex-dioxus/Cargo.toml` | Added `[features]` with `ui-local-providers`; `reqwest` and `futures-util` marked `optional = true` |

### Bridge / Runtime Layer
| File | Change |
|------|--------|
| `ui/codex-dioxus/src/bridge/types.rs` | `LocalOllamaProvider`/`LocalTurboquantProvider` enum variants gated with `#[cfg(feature = "ui-local-providers")]`; added `LocalProviderPolicy`, `LocalProviderCounters`, `LocalProviderDraft`; updated `cycle_next()` to skip provider modes in default builds; 4 new boundary tests |
| `ui/codex-dioxus/src/bridge/runtime_client.rs` | `ollama_runtime_response`, `ollama_runtime_stream`, and both provider match arms gated with `#[cfg(feature = "ui-local-providers")]` |

### UI Components
| File | Change |
|------|--------|
| `ui/codex-dioxus/src/app.rs` | Updated `UI_BOUNDARY_LINES` to reference feature flag; replaced manual mode-cycle match with `cycle_next()` |
| `ui/codex-dioxus/src/components/settings_panel.rs` | Provider gate toggle and warning banner are `cfg!()`-conditional; default build shows "not compiled in" notice |

### Proof Artifacts
| File | Change |
|------|--------|
| `artifacts/proof/current/provider_policy_report.json` | **[NEW]** Provider boundary policy artifact |
| `artifacts/proof/verification/proof_manifest.json` | Added `provider_policy_report.json` to artifact list; added `provider_policy` section |

### Scripts
| File | Change |
|------|--------|
| `scripts/check_proof_manifest_consistency.py` | Added provider_policy boundary assertions (5 hard security checks); added `localhost:11434` feature-gate scan with 30-line context window |

### Documentation
| File | Change |
|------|--------|
| `STATUS.md` | Updated Boundaries section; added `provider_policy_report.json` to artifacts list |
| `docs/CHAT_UI_INTEGRATION.md` | Split into "Excluded (all builds)" and "Experimental local providers (feature-gated)" sections with explicit build command |

---

## 4. Commands Run and Results

### Python Verification
```
python -m global_workspace_runtime.scripts.check_action_types
→ PASS: ActionType enum and schema are in sync (10 values)

python -m global_workspace_runtime.scripts.check_sentience_claims
→ PASS: no sentience-claim phrases found (104 files checked)

python -m global_workspace_runtime.scripts.check_no_mv2 .
→ PASS: no .mv2 references (127 files scanned; vendor/memvid-main excluded)

python -m global_workspace_runtime.scripts.check_resource_recovery
→ PASS: resources=0.755 after 25 cycles (seed=5, threshold=0.25)

python -m pytest -q
→ 35 passed in 0.48s

python scripts/clean_python_artifacts.py
→ Cleaned: 9 __pycache__ dirs, 76 .pyc files

python architecture_guard.py
→ PASS: All architecture guards pass.

python scripts/architecture_guard.py
→ PASS: All architecture guards pass.

python scripts/check_proof_manifest_consistency.py
→ PASS: All checked fields are consistent. (55 OK assertions)
```

### UI Rust Verification
```
cargo fmt --all   (ui/codex-dioxus)   → PASS (no diff)
cargo fmt --all   (global-workspace-runtime-rs) → PASS (no diff)

cargo check (ui/codex-dioxus, default)
→ Finished `dev` profile — no errors

cargo test (ui/codex-dioxus, default)
→ test result: ok. 29 passed; 0 failed

cargo test --features ui-local-providers (ui/codex-dioxus)
→ test result: ok. 28 passed; 0 failed
  (1 test correctly absent: #[cfg(not(feature))] default-only test)
```

### Rust Runtime (receipt-backed, not re-run this pass)
Rust proof artifacts were not regenerated this pass. The runtime-core code was formatted
(`cargo fmt`) but not re-proven. Current proof status is receipt-backed from prior run.

---

## 5. Current Proof Metrics

| Metric | Value | Status |
|--------|-------|--------|
| cycles | 15 | ✓ |
| event_count | 541 | ✓ |
| resource_survival | 0.974 | ✓ |
| unsafe_action_count | 0 | ✓ |
| mean_total_score | 0.6433 | ✓ |
| action_match_rate | 1.00 | ✓ |
| held_out scenario count | 26 | ✓ |
| held_out action_match_rate | 1.00 | ✓ |
| adversarial scenario count | 2 | ✓ |
| adversarial action_match_rate | 1.00 | ✓ |
| claims_with_evidence_links | 17 | ✓ |
| audits_with_claim_refs | 18 | ✓ |
| real_external_executions | **0** | ✓ |

---

## 6. Provider / Tool Boundary Assertions

| Assertion | Value | Verified |
|-----------|-------|----------|
| `real_external_executions` | **0** | ✓ tool_policy_report.json |
| `local_provider_feature_enabled` | **false** (default build) | ✓ provider_policy_report.json |
| `external_provider_requests` | **0** | ✓ provider_policy_report.json |
| `cloud_provider_requests` | **0** | ✓ provider_policy_report.json |
| `api_key_storage_enabled` | **false** | ✓ provider_policy_report.json |
| `provider_can_execute_tools` | **false** | ✓ provider_policy_report.json |
| `provider_can_write_memory` | **false** | ✓ provider_policy_report.json |
| `provider_can_override_codex_action` | **false** | ✓ provider_policy_report.json |
| `localhost:11434` in default binary | **absent** | ✓ feature-gate scan (consistency script) |
| `reqwest` in default binary | **absent** | ✓ Cargo.toml optional dependency |

---

## 7. UI Status

| Mode | Status in Default Build |
|------|------------------------|
| `MockUiMode` | ✓ Active — clearly labeled MockOnly metadata |
| `LocalCodexRuntimeReadOnly` | ✓ Active — primary runtime bridge |
| `LocalOllamaProvider` | ✗ Absent (requires `--features ui-local-providers`) |
| `LocalTurboquantProvider` | ✗ Absent (requires `--features ui-local-providers`) |
| `ExternalProviderDisabled` | ✓ Active — shows explicit disabled message |

| Property | Status |
|----------|--------|
| `metadata_quality` variants | `RuntimeGrounded`, `PartiallyGrounded`, `MockOnly`, `Unavailable`, `LocalProviderDraft` (feature-gated) |
| Shell commands in runtime_client.rs | None |
| API key fields | None |
| Cloud provider endpoints | None |
| Provider mode cycling (default) | `Mock → LocalReadOnly → ExternalDisabled → Mock` only |

---

## 8. 10-Action Schema (Unchanged)

```
answer
ask_clarification
retrieve_memory
refuse_unsafe
defer_insufficient_evidence
summarize
plan
execute_bounded_tool
no_op
internal_diagnostic
```

Verified by: `python -m global_workspace_runtime.scripts.check_action_types → PASS (10 values)`

---

## 9. Remaining Limitations

1. **Rust proof artifacts are receipt-backed.** `cargo run -p runtime-cli -- proof --strict --long-horizon --nl` was not re-executed this pass. Proof metrics reflect the prior 15-cycle run.
2. **`LocalProviderPolicy` and `LocalProviderCounters` structs** are defined and tested but not yet wired to a runtime event-loop counter. This is a future hardening task for the `ui-local-providers` feature path.
3. **`provider_gate` field in `RuntimeClient`** is present in default builds but dead code (expected — only read in feature-gated match arms).
4. **NL benchmark** is diagnostic routing over 43 scenarios, not broad natural-language reasoning proof.
5. **Contradiction reporting** is structured/deduped, not semantic truth reasoning.
6. **Evidence-backed claim linkage** remains bounded to structured, proof-known sources.
7. **UI bridge** is local read-only and is not a production assistant.
8. **Dioxus CLI (`dx build`)** was not invoked this pass; UI artifact build is not verified.

---

## 10. Decision

**Integration proof candidate. Not final freeze.**

All provider boundary assertions pass. The 10-action schema is unchanged. Python tests pass.
UI tests pass (default: 29, feature-gated: 28). Proof consistency script passes all 55 assertions.
Provider execution is unambiguously disabled in the default build. The feature-gated path is
clearly documented, approval-gated, and non-authoritative.

---

## Required Boundary Statement

> This system is a broad Rust-authoritative cognitive-runtime scaffold. It is not sentient,
> not conscious, not AGI, not production-ready, not a safe autonomous external tool executor,
> and not a complete evidence-grounded cognitive agent.
