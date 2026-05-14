# Legacy Boundaries and Code Authority

**Clarifies which code is active authority vs. reference/superseded.**

---

## Authority Hierarchy

### **1. Tier 1: Active Authoritative**

**Location:** `global-workspace-runtime-rs/crates/runtime-core/`

This is the **single source of truth** for all runtime decisions.

**Key Files:**
- `src/event.rs` — Event type definitions
- `src/reducer.rs` — State transition logic
- `src/runtime_loop.rs` — Central event loop (only one in repo)
- `src/runtime_state.rs` — State storage
- `src/lib.rs` — Public API

**Properties:**
- Verified by CI: `cargo test --workspace --all-targets --all-features`
- Verified by CI: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Single authority enforced by architecture_guard.py (only one runtime_loop.rs)
- No competing implementations in repo

**Bindings to other crates:** All other crates import runtime-core; runtime-core imports none of them (edges point inward).

---

### **2. Tier 2: Active Reference (Rust Ecosystem)**

**Locations:**
- `global-workspace-runtime-rs/crates/simworld/`
- `global-workspace-runtime-rs/crates/modulation/`
- `global-workspace-runtime-rs/crates/cognition/`
- `global-workspace-runtime-rs/crates/symbolic/`
- `global-workspace-runtime-rs/crates/memory/`
- `global-workspace-runtime-rs/crates/evidence/`
- `global-workspace-runtime-rs/crates/contradiction/`
- `global-workspace-runtime-rs/crates/tools/`
- `global-workspace-runtime-rs/crates/gw-workspace/`
- `global-workspace-runtime-rs/crates/runtime-cli/`

**Properties:**
- Compiled and tested as part of Rust workspace
- All tested via `cargo test --workspace --all-targets --all-features`
- All linted via `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Used by runtime-core or runtime-cli
- May be replaced/refactored without affecting authority

**To Replace/Refactor:** Implement new module with same trait/public API; update imports in runtime-core or runtime-cli.

---

### **3. Tier 3: Legacy Reference (Python)**

**Location:** `src/global_workspace_runtime/`

**Status:** Reference only. Not imported by Rust, not compiled into binary.

**Key Modules:**
- `src/global_workspace_runtime/core/` — Legacy Python runtime (superseded)
- `src/global_workspace_runtime/cognition/` — Legacy streams, critic, planner
- `src/global_workspace_runtime/memory/` — Legacy archive backends
- `src/global_workspace_runtime/scripts/` — Utility functions

**You May:**
- ✅ Reference for understanding old design decisions
- ✅ Copy patterns that proved useful
- ✅ Use in Python test suite for reference validation
- ✅ Document lessons learned

**You May NOT:**
- ❌ Import into Rust runtime (architecture guard enforces this)
- ❌ Treat as current authority
- ❌ Copy wholesale without understanding
- ❌ Claim Python behavior drives Rust behavior

**Testing:** `python -m pytest -q` tests only Python modules; does not validate Rust.

---

### **4. Tier 4: Vendored Reference (Memvid)**

**Location:** `vendor/memvid-main/`

**Status:** Vendored source, **not integrated at runtime.**

**Current Integration:**
- MemvidBackend trait in `crates/memory/src/lib.rs`
- Implementation returns `NotImplemented` for all operations
- No real Memvid execution occurs

**You May:**
- ✅ Reference Memvid papers/design docs in this directory
- ✅ Understand original Memvid architecture
- ✅ Salvage specific algorithms if ported and tested in Rust

**You May NOT:**
- ❌ Assume Memvid runs at runtime (it doesn't)
- ❌ Link against Memvid C binaries (stub only)
- ❌ Claim multi-modal reasoning works
- ❌ Build on unverified Memvid integration

---

### **5. Tier 5: Superseded Legacy (Old Kernel)**

**Location:** `runtime/kernel/`

**Status:** Superseded by runtime-core. Kept for reference only.

**Reason for Keeping:**
- Historical reference for architectural decisions
- Comparison point for improvements
- Learning resource

**You May:**
- ✅ Reference architectural patterns
- ✅ Extract lessons learned
- ✅ Compare performance characteristics (if tested)

**You May NOT:**
- ❌ Import or compile
- ❌ Treat as active code
- ❌ Link against it
- ❌ Claim it's still used
- ❌ Port code directly (must validate in Rust first)

---

## Code Salvage Rules

### **Pattern: "Useful Pattern Found in Legacy"**

**Process:**

1. **Identify** — Find pattern in legacy code (Python, old Kernel, Memvid)
2. **Understand** — Why does it work? What problem does it solve?
3. **Reimplement** — Write in Rust-idiomatic way, in target crate
4. **Test** — Add tests to verify behavior (unit + integration)
5. **Verify** — Run `cargo test` and `cargo clippy` on target crate
6. **Document** — Add comment linking to legacy reference and explaining adaptation
7. **Update** — Integrate into runtime-core or call site
8. **Re-verify** — Full test suite (`cargo test --workspace --all-targets --all-features`)

**Example: Salvaging Memory Retrieval Logic**

```rust
// Bad: Copy Python directly
// let stored_memory = python_legacy_retrieval(query)  // DON'T DO THIS

// Good: Understand, reimplement, test
// See src/global_workspace_runtime/memory/retrieval.py for original logic
// We adapted the keyword-matching algorithm (modified Jensen-Shannon distance)
// into our keyword_memory_provider.rs and tested it independently.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn keyword_match_identical_to_legacy() {
        // Verify behavior matches legacy for backward compatibility
        let query = MemoryQuery { kind: Memory Kind::Episodic, text: "what time is it?" };
        let legacy_result = python_legacy_retrieval(query);
        let rust_result = keyword_memory_provider::retrieve(&self, query);
        assert_eq!(rust_result.count(), legacy_result.count());
    }
}
```

---

## UI Bridge Safety Rules

**Location:** `ui/codex-dioxus/src/bridge/`

**Authority:** Dioxus UI **calls** runtime-core; runtime-core does **not** call UI.

**Safe Patterns:**

```rust
// ✅ Good: UI calls Rust runtime
let result = runtime_client.send_user_message("user query").await;
let (action, confidence, _evidence) = result;
// UI displays action, confidence (no direct action execution)

// ❌ Bad: Rust runtime calls UI functions
// (rust-core never imports ui crate)
```

**Provider/Tool Execution in UI:**

```rust
// ✅ Good: Provider gate checked before any external call
if !self.provider_gate.is_enabled() {
    return ProviderDenied { reason: "disabled by policy" }.into();
}
// Only if gate enabled, call provider (and even then, only in feature-gated code)

// ❌ Bad: Feature-gated provider call without gate check
#[cfg(feature = "ui-local-providers")]
let response = ollama_runtime_response(input, "llama3").await;  // GATE CHECK MISSING
```

**Rule:** Any provider or tool execution path must go through a centralized guarded function that checks policy, increments counters, and respects safe defaults.

---

## CI Guard Verification

**All these guards run on every commit:**

```bash
# Enforces single runtime_loop.rs (Tier 1 authority single source)
python architecture_guard.py

# Enforces no sentience claims anywhere in repo
PYTHONPATH=src python -m global_workspace_runtime.scripts.check_sentience_claims

# Enforces no Memvid v2 format (Tier 4 boundary)
cargo run -p runtime-cli -- check-no-fake-mv2

# Enforces no Python Bytecode committed (Tier 3 isolation)
python scripts/clean_python_artifacts.py && git check -clean

# Enforces proof artifacts consistent (Tier 1 output validation)
python scripts/check_proof_manifest_consistency.py
```

---

## File Summary

| Path | Tier | Authority | May Import? | May Be Imported? |
|---|---|---|---|---|
| `global-workspace-runtime-rs/crates/runtime-core/` | **1** | **Yes** | runtime-core only | All crates |
| `global-workspace-runtime-rs/crates/simworld/` | 2 | Reference | runtime-core | runtime-cli |
| `global-workspace-runtime-rs/crates/modulation/` | 2 | Reference | runtime-core, cognition | cognition, runtime-cli |
| `global-workspace-runtime-rs/crates/cognition/` | 2 | Reference | runtime-core, modulation | runtime-cli |
| `global-workspace-runtime-rs/crates/symbolic/` | 2 | Reference | runtime-core | gw-workspace, runtime-cli |
| `global-workspace-runtime-rs/crates/memory/` | 2 | Reference | runtime-core, evidence | simworld, runtime-cli |
| `global-workspace-runtime-rs/crates/evidence/` | 2 | Reference | runtime-core | memory, contradiction, runtime-cli |
| `global-workspace-runtime-rs/crates/contradiction/` | 2 | Reference | runtime-core | memory, runtime-cli |
| `global-workspace-runtime-rs/crates/tools/` | 2 | Reference | runtime-core | cognition, runtime-cli |
| `global-workspace-runtime-rs/crates/gw-workspace/` | 2 | Reference | runtime-core, symbolic | runtime-cli |
| `global-workspace-runtime-rs/crates/runtime-cli/` | 2 | Reference | all Tier 2 crates | (binary, no imports) |
| `src/global_workspace_runtime/` | 3 | Legacy | None (Python only) | Tests only |
| `vendor/memvid-main/` | 4 | Vendored | None (memory crate stub) | (reference only) |
| `runtime/kernel/` | 5 | Superseded | None | (reference only) |

---

## Decision Tree: "Should I Use This Code?"

```
Code found in repo?
├─ YES, in global-workspace-runtime-rs/crates/runtime-core/ ?
│  └─ Tier 1: This IS the authority. Use it.
│
├─ YES, in global-workspace-runtime-rs/crates/{other}/ ?
│  └─ Tier 2: Active, tested. Use it if it solves your problem.
│     └─ Want to change? Understand contract, add tests, update imports.
│
├─ YES, in src/global_workspace_runtime/ (Python) ?
│  └─ Tier 3: Legacy reference. Understand it, reimplement in Rust.
│     └─ For test validation? Use in Python test suite only.
│
├─ YES, in vendor/memvid-main/ ?
│  └─ Tier 4: Vendored reference. Study it, don't link to it.
│     └─ If useful pattern? Reimplement in Rust, test, add to memory crate.
│
├─ YES, in runtime/kernel/ ?
│  └─ Tier 5: Superseded. Reference only for design lessons.
│     └─ Don't import. Don't link. Don't claim it works.
│
└─ NOT in repo? Write it new (Tier 1 level), test it, integrate it.
```

---

## Examples of Correct Usage

### **Example 1: Add New Memory Retrieval Algorithm**

❌ **Wrong:**
```rust
// Copy Python code directly
fn retrieve(&self, query: &MemoryQuery) -> Vec<MemoryHit> {
    // ... code from src/global_workspace_runtime/memory/retrieval.py ...
    // (Python semantics may not map to Rust)
}
```

✅ **Right:**
```rust
// Understand the algorithm, test it independently, document source
// Adapted from: src/global_workspace_runtime/memory/retrieval.py
// Changes: Rust iterators instead of Python list comprehensions, 
// added salience decay calculation based on valid_to timestamp

fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError> {
    // ... Rust implementation ...
    #[test]
    fn verify_against_legacy() {
        // Test behavior against legacy to ensure equivalent semantics
    }
}
```

### **Example 2: Port Memvid Algorithm**

❌ **Wrong:**
```rust
// Trust Memvid works, assume C-binding is safe
let result = unsafe { memvid_retrieve_c_function(...) };
// (Memvid is vendored, not tested, not in runtime)
```

✅ **Right:**
```rust
// Study vendored Memvid source (vendor/memvid-main/)
// Extract algorithm logic
// Implement in pure Rust in crates/memory/
// Test with benchmark in simworld
// Then integrate if benchmarks show improvement

// See: vendor/memvid-main/[paper] for algorithm overview
// Implemented as: crates/memory/src/advanced_retrieval.rs
// Tested via: cargo test advanced_retrieval --
// Benchmarked via: cargo run -p runtime-cli -- benchmark

fn advanced_retrieval(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError> {
    // ... implementation ...
}

#[cfg(test)]
mod tests {
    #[test]
    fn advanced_retrieval_improves_over_keyword() {
        // Prove new algorithm is better
    }
}
```

### **Example 3: Use Existing Tier 2 Code**

✅ **Right:**
```rust
// runtime-core calls simworld for proof
use simworld::SimWorldEvaluator;

let evaluator = SimWorldEvaluator::new(scenarios);
let result = evaluator.run_scenario(scenario_id, self.state).await;
// (simworld is tested, in Tier 2, safe to use)
```

---

## Refactoring Steps: "Improve Existing Code"

### **If you want to refactor Tier 2 (e.g., simworld)**

1. **Understand** — Why does the current implementation exist? What problem does it solve?
2. **Test** — Write tests for current behavior (`cargo test simworld`)
3. **Improve** — Modify implementation (keep public API same if possible)
4. **Verify** — Run `cargo test --all-features` on modified crate
5. **Integrate** — If API changed, update callers (runtime-cli, simworld)
6. **Full Test** — Run `cargo test --workspace --all-targets --all-features`
7. **Verify CI** — Push and verify CI passes

### **If you want to replace Tier 2 (e.g., simworld)**

1. **Prototype** — New implementation in feature branch
2. **Test** — Achieve same or better test coverage
3. **Benchmark** — If performance mattered, show improvement
4. **Integration** — Integrate with existing public API (minimal update to callers)
5. **Deprecate** — Old implementation stays until callers updated
6. **Full Suite** — All tests pass
7. **Documented** — Comment explains what was replaced and why

---

## What NOT to Do

### **❌ Do NOT**

1. **Import Python code directly into Rust** — Understand it, reimplement it
2. **Claim Memvid executes** — It's stubbed; acknowledge in docs
3. **Treat old kernel as authority** — runtime-core is the authority
4. **Link against vendored C code** — All vendored code is reference only
5. **Assume Python tests validate Rust** — Python tests only validate Python
6. **Bypass provider/tool gates** — All such paths must go through one guarded function
7. **Create competing runtime loops** — Single runtime-core authority enforced by CI
8. **Store API keys** — All provider paths disabled by default; no key storage
9. **Execute real tools without policy gate** — Tool candidates only; no real execution
10. **Claim consciousness/sentience based on symbolic routing** — Symbolic is machinery, not proof

---

## Governance Summary

| Decision | Authority | Reference | Override? |
|---|---|---|---|
| Runtime behavior | runtime-core | Tier 2 crates | **No** — runtime-core is final |
| Feature behavior | Tier 2 crate (local) | Tier 3 legacy, Tier 4 vendored | Yes — if tests pass |
| Proof correctness | runtime-cli | All crates tested | No — proof is final output |
| UI bridge safety | runtime_client.rs | (none) | No — safety enforced by CI |
| Tool/provider policy | tools.rs, modulation.rs | (none) | No — disabled by default |
| Test requirements | `cargo test --all-targets --all-features` | (none) | No — all tests must pass |

---

## Next Steps for Contributors

1. ✅ Read this document fully
2. ✅ Read `docs/REPO_INVENTORY.md` to understand crate structure
3. ✅ Identify what you want to change (which tier?)
4. ✅ Follow the appropriate salvage/refactor/new code pattern above
5. ✅ Run full test suite before PR
6. ✅ All CI checks must pass: format, lint, test, proof command, architecture guards
