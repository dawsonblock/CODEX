# Phase 8: Provider-Gate Denial Semantics — Analysis & Fix

**Date:** May 14, 2026  
**Package:** CODEX-main 36 hardening candidate  
**Phase Status:** 🔴 IDENTIFIED - Ready for Implementation

---

## Problem Statement

**Semantic Confusion:** When provider execution is blocked, callers cannot distinguish between:
1. **Provider gate disabled** (admin/security policy) — user can't enable providers
2. **User input unsafe** (content policy) — user should revise their query

Both currently return a generic error message that conflates the two cases.

### Current Behavior (Line 564-566)

```rust
if !provider_gate {
    PROVIDER_LOCAL_DISABLED_BLOCKS.fetch_add(1, Ordering::Relaxed);
    let err_msg = format!(
        "Security Error: Local provider execution is gated. Enable 'Provider Security Gate' in Settings to use {}.",
        if model_name == "turboquant" { "Turboquant" } else { "Ollama" }
    );
```

**Problem:** This message appears whenever `provider_gate == false`, regardless of whether the user's input was safe or not. From the UI perspective, there's no way to know if the blockage was due to:
- A. Policy (gate disabled) — user should contact admin
- B. Content (input unsafe) — user should reformulate

### Impact of Non-Distinction

1. **User Confusion:**  "Enable 'Provider Security Gate' in Settings" but maybe there IS no such setting (gate is disabled by compile-time feature)
2. **Audit Trail:** No separation between policy denials and content denials in counters
3. **Transparency:** Unclear to user why their specific input was rejected

---

## Solution: Two-Level Denial Breakdown

### New Flow

```
User Input
    ↓
┌─────────────────────┐
│ Provider Gate Check │
└──────┬──────────────┘
       │
       ├─ No (gate disabled)
       │  └→ "Provider execution disabled by admin. Remaining in read-only mode."
       │     [provider_policy_decision: "provider_disabled"]
       │
       └─ Yes (gate enabled)
          ↓
       ┌──────────────────────┐
       │ Content Safety Check  │
       └──────┬───────────────┘
              │
              ├─ Unsafe
              │  └→ "I cannot help with that request."
              │     [tool_policy_decision: "deny_unsafe"]
              │
              └─ Safe
                 └→ [Call provider normally]
```

### Implementation Changes

**RuntimeStepResult Fields:**
- `provider_policy_decision: Option<String>` — Populated ONLY when provider gate blocks
- `tool_policy_decision: Option<String>` — Populated when input content is unsafe

**Distinction:**
- Provider gate → `provider_policy_decision: Some("provider_disabled")`
- Input unsafe → `tool_policy_decision: Some("deny_unsafe")`
- Never both at once

---

## Current Code State

**File:** `ui/codex-dioxus/src/bridge/runtime_client.rs`

Current RuntimeStepResult constructor when provider_gate blocks (line 566-577):
```rust
return RuntimeStepResult::with_error(err_msg);  // Generic error, loses context
```

This needs to be:
```rust
return RuntimeStepResult {
    response_text: "Provider execution disabled by admin. Remaining in read-only mode.".to_string(),
    selected_action: "defer_insufficient_evidence".to_string(),
    provider_policy_decision: Some("provider_disabled".to_string()),  // NEW distinction
    tool_policy_decision: None,  // Explicitly None—input was safe, policy blocked it
    // ... other fields ...
};
```

---

## Non-Negotiable Constraints

✅ Do NOT change actual blocking behavior (provider still blocks when gate is false)
✅ Do NOT change counters (PROVIDER_LOCAL_DISABLED_BLOCKS still increments)
✅ Do NOT add provider execution when gate disabled (stays blocked)
✅ Do NOT fabricate content safety assessment (only distinguish gate vs. content)
✅ Maintain Rust type safety (both Option<String> fields)

---

## Acceptance Criteria

Phase 8 complete when:
1. ✅ RuntimeStepResult distinguishes provider vs. content denial
2. ✅ `provider_policy_decision` set ONLY for gate blocks
3. ✅ `tool_policy_decision` set ONLY for unsafe content
4. ✅ Message text reflects the distinction
5. ✅ All tests updated and passing (76→77 expected)
6. ✅ Proof artifacts regenerated

---

## Next Phases Preview

- **Phase 9:** Expand event origins (EventOrigin mostly defaults to RuntimeLoop, should be subsystem-specific)
- **Phase 10:** Fix/document 6 NL held-out failures (currently undocumented in proof)
- **Phase 11-14:** Final artifact semantics, documentation, validation

---

## Status for Continuation

This phase is identified and ready. The fix requires:
1. Modifying guarded_provider_response function logic (10 lines)
2. Updating RuntimeStepResult constructors (2 sites)
3. Adding test to verify distinction (1 test)
4. Regenerating proof

**Estimated complexity:** Low (straightforward distinguishing logic)
**Risk level:** Minimal (only changes message text and field population, not blocking behavior)
