# Phase 9 Discovery: UI Current State Analysis

**Date:** Phase 9 Start  
**Status:** DISCOVERY COMPLETE  
**Finding:** AnswerEnvelope data fully supported in types, but not displayed in UI components

---

## Current UI Architecture Overview

### Technology Stack
- **Framework:** Dioxus 0.7+
- **Language:** Rust with JSX-like component syntax
- **Build:** cargo build for WASM target
- **Directory:** `/Users/dawsonblock/CODEX-1/ui/codex-dioxus/`

### Component Structure

```
src/
├── main.rs              # Dioxus app entry
├── app.rs               # Main application component
├── bridge/              # Runtime integration
│   ├── runtime_client.rs    # ← Core bridge to Rust runtime
│   ├── types.rs             # ← Data types (RuntimeStepResult, BasisItemSummary)
│   ├── proof_reader.rs
│   └── mod.rs
└── components/          # UI components
    ├── message_bubble.rs       # ← Answer display (NEEDS ENHANCEMENT)
    ├── chat_view.rs            # Chat thread renderer
    ├── action_trace_panel.rs   # ← Detailed trace metadata
    ├── evidence_panel.rs       # Summary statistics
    ├── pressure_panel.rs       # Pressure/regulation metrics
    ├── audit_panel.rs
    ├── action_schema_panel.rs
    ├── chat_input.rs
    ├── command_queue.rs
    ├── console_panel.rs
    ├── runtime_status.rs
    ├── settings_panel.rs
    └── mod.rs
```

---

## Data Types: Phase 8 Output Already in UI Types

### RuntimeStepResult (Complete Answer Data)

**Location:** `types.rs` lines 35-60

```rust
pub struct RuntimeStepResult {
    pub selected_action: String,
    pub response_text: String,
    
    // ✅ Phase 8 AnswerEnvelope Fields - ALL PRESENT
    pub answer_basis: Option<String>,           // "grounded_active_claims" or defer reason
    pub answer_basis_items: Vec<BasisItemSummary>,  // ← KEY: Claim basis items!
    pub answer_warnings: Vec<String>,           // Policy warnings
    
    // Supporting evidence/claim tracking
    pub audit_id: Option<String>,
    pub evidence_ids: Vec<String>,
    pub evidence_hashes: Vec<String>,
    pub claim_ids: Vec<String>,
    pub contradiction_ids: Vec<String>,
    
    // Pressure and policy
    pub dominant_pressures: Vec<String>,
    pub replay_safe: bool,
    pub missing_evidence_reason: Option<String>,
    pub tool_policy_decision: Option<String>,
    pub provider_policy_decision: Option<String>,
    
    pub metadata_quality: MetadataQuality,
    pub bridge_mode: String,
    pub pressure_updates: usize,
    pub policy_bias_applications: usize,
    pub provider_executions_local: usize,
    pub provider_counters: ProviderCountersSummary,
}
```

### BasisItemSummary (Claim-to-Evidence Link)

**Location:** `types.rs` lines 24-31

```rust
pub struct BasisItemSummary {
    pub claim_id: String,              // Link to source claim
    pub subject: String,               // Claim subject
    pub predicate: String,             // Claim predicate
    pub object: Option<String>,        // Claim object
    pub confidence_pct: u8,            // Confidence as 0-100
    pub evidence_ids: Vec<String>,     // Backing evidence IDs
}
```

**Status:** ✅ Full Phase 8 data structure present and serializable

---

## Current Component Usage Analysis

### 1. message_bubble.rs — Answer Display

**Current Output:**
```
┌─ User's Message ──────────────────────────────────────┐
│ [text of user input]                                   │
├─────────────────────────────────────────────────────────┤
│ time: 2026-05-14 10:30:45                              │
╚─────────────────────────────────────────────────────────╝

┌─ Codex's Response ────────────────────────────────────┐
│ The answer text goes here...                           │
├─────────────────────────────────────────────────────────┤
│ time: 2026-05-14 10:30:46                              │
│ action: answer                                          │
│ metadata: Runtime-grounded                             │
│ [⚠️ Non-Authoritative Provider (2) - if applicable]    │
└─────────────────────────────────────────────────────────┘
```

**What's Rendered:**
- ✅ Message text
- ✅ Timestamp
- ✅ Action type ("answer", "defer_insufficient_evidence", etc.)
- ✅ Metadata quality label
- ✅ Provider warning badge (if applicable)

**What's NOT Rendered:**
- ❌ answer_basis_items (the claim basis items table!)
- ❌ answer_basis status
- ❌ answer_warnings
- ❌ confidence scores
- ❌ evidence references per claim

**Code Location:** `src/components/message_bubble.rs` lines 10-45

---

### 2. action_trace_panel.rs — Detailed Metadata Panel

**Current Output:**
```
ACTION TRACE
────────────────────────────────────────
selected_action: answer
metadata_quality: Runtime-grounded
replay_safe: true
pressure_updates: 3
policy_bias_applications: 2
evidence_ids: ev-001, ev-002, ev-003
evidence_hashes: sha256:abc..., sha256:def...
claim_ids: cl-001, cl-002
contradiction_ids: none
Runtime audit ID: audit-12345
dominant_pressures: uncertainty, coherence
tool_policy_decision: approved
provider_policy_decision: allowed
missing_evidence_reason: none
provider_executions_local: 0
```

**What's Rendered:**
- ✅ Single-line evidence_ids list
- ✅ Single-line claim_ids list
- ✅ Contradiction IDs
- ✅ Pressure metrics (final values only)
- ✅ Policy decisions

**What's NOT Rendered:**
- ❌ Per-claim evidence linkage (just lists all IDs)
- ❌ Claim subjects/predicates
- ❌ Confidence per claim
- ❌ Cycle-by-cycle pressure dynamics
- ❌ Timeline of events

**Code Location:** `src/components/action_trace_panel.rs` lines 6-90+

---

### 3. evidence_panel.rs — Summary Statistics Only

**Current Output:**
```
EVIDENCE AND CONTRADICTION
───────────────────────────
Evidence Items:     96
Claims:             17
Contradictions:     2
Contradiction Rate: 0.118
```

**Status:** Summary metrics only, not per-answer visualization

---

### 4. pressure_panel.rs — Final Values Only

**Current Output:**
```
OPERATIONAL PRESSURE
────────────────────
Pressure:   0.425
Regulation: 0.681

(Note: "Pressure modulation is runtime-authored and read-only in the UI shell.")
```

**Status:** Only shows final cycle values, not dynamics across 15 cycles

---

## Integration Flow

### How AnswerEnvelope Data Reaches UI

```
Rust Runtime (runtime-cli)
        ↓ EventEnvelope
Claim Store + Evidence Vault
        ↓ AnswerBuilder.build()
   AnswerEnvelope
        ↓ JSON serialization
    HTTP Response / MockResponse
        ↓
   runtime_client.rs (parse into RuntimeStepResult)
        ↓
   ChatMessage { runtime: Some(RuntimeStepResult) }
        ↓
   ChatView → MessageBubble
        ↓
   Browser Display
```

**Verified:** All AnswerEnvelope fields correctly flow through to UI types

---

## Phase 9 Gap Analysis

### What Phases 5-8 Delivered

| Phase | Deliverable | In UI Types? | In UI Display? |
|-------|------------|--------------|-----------------|
| Phase 5 | EventEnvelope | ✅ Yes | ✅ Partial (audit_id shown) |
| Phase 6 | Evidence vault | ✅ Yes | ✅ Partial (evidence_ids shown) |
| Phase 7 | Retrieval policies | ✅ Yes (retrieval reasons) | ❓ Not clear |
| Phase 8 | AnswerBasisItems | ✅ Yes (full struct) | ❌ **NOT SHOWN** |

### Critical Gap

**AnswerBasisItems are in the data but not displayed:**

- ✅ Data structure in runtime_client.rs: `answer_basis_items: Vec<BasisItemSummary>`
- ✅ Data flowing from runtime to UI: Confirmed in RuntimeStepResult
- ❌ **NOT rendered in message_bubble.rs** — Currently ignored!

---

## Enhancement Opportunities

### Priority 1: Message Bubble Basis Items Table

**Enhancement:** Add collapsible basis items table below the answer text

```
┌─ Codex's Response ────────────────────────────────────┐
│ The answer text goes here...                           │
├─────────────────────────────────────────────────────────┤
│ time: 2026-05-14 10:30:46                              │
│ action: answer                                          │
│ metadata: Runtime-grounded                             │
│                                                        │
│ ✅ GROUNDED ANSWER (answer_basis_items: 2)             │
│                                                        │
│ ┌─ Basis Items ─────────────────────────────────────┐  │
│ │ Claim ID     │ Subject      │ Predicate │ Conf │  │  │
│ ├──────────────┼──────────────┼───────────┼──────┤  │  │
│ │ cl-001       │ Entity ABC   │ is_valid  │ 85%  │  │  │
│ │ cl-002       │ System XYZ   │ ready     │ 92%  │  │  │
│ └─ cl-001 evidence: ev-001, ev-003 ────────────────┘  │
│ └─ cl-002 evidence: ev-002, ev-005 ────────────────┘  │
│                                                        │
│ ⚠️ 1 Contradicted Claim (not shown): cl-099           │
└─────────────────────────────────────────────────────────┘
```

**Implementation:** Enhance message_bubble.rs with conditional basis item rendering

### Priority 2: Action Trace Enhanced with Claims

**Enhancement:** Show claim details, not just IDs

```
CLAIM DETAILS (from action_trace_panel)
────────────────────────────────────────
Claim cl-001:
  Subject:    Entity ABC
  Predicate:  is_valid
  Confidence: 85%
  Evidence:   ev-001, ev-003
  
Claim cl-002:
  Subject:    System XYZ
  Predicate:  ready
  Confidence: 92%
  Evidence:   ev-002, ev-005
```

### Priority 3: Pressure Dynamics Timeline

**Enhancement:** Show pressure evolution across 15 cycles

```
PRESSURE DYNAMICS (cycles 1-15)
────────────────────────────────
Pressure:    │▁▂▃▄▅▆▇█████▆▄
             0.0 ———————— 1.0
Regulation:  │▄▅▆▇██████▇▆▅▄▃
             0.0 ———————— 1.0
```

### Priority 4: Timeline Visualization

**Enhancement:** Show claim creation, evidence linking, action sequence

```
TIMELINE (15 cycles)
─────────────────────
Cycle 1: ● claim (cl-001) created
Cycle 2: ● evidence (ev-001) linked
         ● evidence (ev-003) linked
Cycle 3: ● query executed
Cycle 4: ● answer generated
         ⚡ pressure spike (0.85)
...
```

---

## Recommendation: Phase 9 Approach

**Phased Enhancement (not big-bang):**

1. **Immediate (Priority 1):** Enhance message_bubble.rs
   - Add answer_basis_items table
   - Show confidence scores
   - Show evidence references per claim
   - Show answer_warnings
   - **Impact:** Make Phase 8 data visible to users
   - **Time:** ~30 minutes

2. **Follow-up (Priority 2):** Enhanced action_trace_panel
   - Show claim details from basis items
   - Cross-reference with evidence
   - **Time:** ~20 minutes

3. **Timeline (Priority 3):** Add chart component
   - Pressure evolution chart
   - Cycle-by-cycle visualization
   - **Time:** ~25 minutes

4. **Advanced (Priority 4):** Long-horizon trace viewer
   - Per-cycle breakdown
   - Interactive navigation
   - **Time:** ~20 minutes

---

## Implementation Checklist for Phase 9

- [ ] Create enhanced AnswerBasisTable component
- [ ] Integrate BasisItemSummary table into MessageBubble
- [ ] Add answer_warnings display
- [ ] Add confidence color coding (green 80+%, yellow 50-80%, red <50%)
- [ ] Add evidence ID links in basis items
- [ ] Create pressure_dynamics_chart.rs component
- [ ] Create timeline_viewer.rs component
- [ ] Create claim_details_panel.rs component
- [ ] Update proof command to generate ui_integration_report.json
- [ ] Create generate_ui_integration_report.py script
- [ ] Update proof_manifest.json
- [ ] Create PHASE_9_SUMMARY.md

---

## Conclusion

**Discovery Status:** ✅ COMPLETE

**Key Findings:**
1. ✅ All Phase 8 AnswerEnvelope data is in the UI type system
2. ✅ Data correctly flows from runtime to UI components
3. ❌ **Critical Gap:** answer_basis_items displayed nowhere in UI
4. ✅ Perfect opportunity for Phase 9 to surface this data

**Next Step:** Begin Phase 9 Implementation starting with message_bubble enhancement
