# PHASE 10 PLAN: Runtime Integration & Live Data Connection

## Overview

Phase 10 transforms the UI from mock-data visualization to live runtime-driven components. All Phase 9 UI components will be connected to actual EventEnvelope streams, claim store, evidence vault, and pressure regulatory signals.

**Objective:** Achieve full data binding between Rust runtime and Dioxus frontend  
**Scope:** 6-step integration roadmap  
**Deliverables:** Live UI with real metrics, state management, and bidirectional updates  

---

## Architecture Shift

### Phase 9 (Mock Data)
```
Dioxus Components (Static Mock Data)
└── Hard-coded test arrays
```

### Phase 10 (Live Data)
```
Rust Runtime
├── EventEnvelope Stream
├── Claim Store (17 claims)
├── Evidence Vault (96 entries)
└── Pressure Regulatory Signals
    ↓
runtime_client.rs (State Management)
├── LiveEventBridge
├── LiveClaimStore
├── LiveEvidenceVault
└── PressureMetrics
    ↓
Dioxus Components (Signal-driven)
├── Timeline Viewer (Live events)
├── Pressure Dynamics Chart (Real signals)
├── Trace Viewer (Actual cycles)
└── Basis Items Table (Real claims)
```

---

## 6-Step Integration Plan

### Step 1: Event Stream Bridge
**Goal:** Connect timeline viewer to EventEnvelope stream

- Create `live_event_bridge.rs` in runtime_client
- Implement EventEnvelope → Timeline event mapping
- Add signal-based state for event list
- Update `timeline_viewer.rs` to consume live events
- Test with 15-cycle runtime proof

**Deliverables:**
- Live event stream subscription
- Real timeline visualization
- Event filtering by type
- Cycle-accurate event ordering

---

### Step 2: Claim Store Integration
**Goal:** Wire claim_details_panel to actual claim store

- Create `live_claim_store_bridge.rs`
- Implement Claim → UI claim card mapping
- Add grounding status tracking
- Update `claim_details_panel.rs` to render real claims
- Display contradiction warnings from actual contradictions

**Deliverables:**
- Live claim list with real data
- Grounding metrics (17 total, 16 validated)
- Contradiction detection
- Evidence link resolution

---

### Step 3: Evidence Resolution
**Goal:** Link basis_items_table and claim evidence to vault

- Create `live_evidence_vault_bridge.rs`
- Implement evidence_id → evidence_entry lookup
- Add evidence text rendering
- Display confidence scores from actual vault
- Show evidence provenance (assertion/query/memory)

**Deliverables:**
- Live evidence display
- Confidence score rendering
- Evidence search by ID
- Provenance labeling

---

### Step 4: Pressure Signal Binding
**Goal:** Connect pressure_dynamics_chart to real regulatory signals

- Create `pressure_metrics_bridge.rs`
- Map regulatory signals to pressure/regulation values
- Calculate real averages, peaks, ranges
- Wire chart component to signal updates
- Add pressure threshold warnings

**Deliverables:**
- Real pressure curve visualization
- Regulation signal tracking
- Peak/average calculations
- Threshold-based styling

---

### Step 5: Trace Cycle Navigation
**Goal:** Link trace_viewer to actual claim store cycles

- Implement cycle-aware state management
- Map claim IDs to actual cycles
- Show real evidence links per cycle
- Display actual action types from runtime
- Update confidence from real metrics

**Deliverables:**
- Per-cycle claim breakdown
- Real evidence attribution
- Actual action type tracking
- Confidence per cycle

---

### Step 6: State Management & Bidirectional Updates
**Goal:** Establish full runtime-UI state synchronization

- Implement Dioxus signals for all major data types
- Add subscription model for runtime updates
- Implement UI invalidation on data changes
- Add error boundary components
- Implement loading states and fallbacks

**Deliverables:**
- Dioxus signal-based state
- Real-time updates on runtime changes
- Error handling
- Loading placeholders
- State persistence (local storage)

---

## Implementation Details

### Data Structure Mappings

#### EventEnvelope → Timeline Event
```rust
pub struct LiveTimelineEvent {
    cycle: usize,
    timestamp: f64,
    event_type: EventType,  // claim, evidence, query, answer, etc.
    claim_ids: Vec<String>,
    evidence_ids: Vec<String>,
    confidence: f64,
    message: String,
}
```

#### Claim → Claim Details Card
```rust
pub struct LiveClaimDisplay {
    claim_id: String,
    subject: String,
    predicate: String,
    object: Option<String>,
    grounding_status: GroundingStatus,  // validated, pending, failed
    evidence_count: usize,
    contradiction_count: usize,
    confidence: u8,
    evidence_ids: Vec<String>,
}
```

#### Pressure Signal → Chart Data
```rust
pub struct PressureMetrics {
    cycle: usize,
    pressure: f64,           // 0.0-1.0
    regulation: f64,         // 0.0-1.0
    peak_pressure: f64,
    avg_pressure: f64,
    avg_regulation: f64,
    threshold_exceeded: bool,
}
```

---

## Integration Order

**Week 1:**
- Step 1: Event Stream Bridge
- Step 2: Claim Store Integration

**Week 2:**
- Step 3: Evidence Resolution
- Step 4: Pressure Signal Binding

**Week 3:**
- Step 5: Trace Cycle Navigation
- Step 6: State Management

---

## Testing Strategy

### Unit Tests
- Event stream subscription
- Signal updates
- State mutations
- Bridge transformations

### Integration Tests
- Runtime → UI data flow
- UI component rendering with real data
- State synchronization
- Error recovery

### E2E Tests
- Full 15-cycle runtime
- Live UI visualization
- Interactive component testing
- Performance benchmarks

---

## Success Criteria

✅ **Timeline Viewer:** Shows real events from runtime proof  
✅ **Claim Details:** Displays all 17 actual claims with grounding status  
✅ **Basis Items Table:** Shows real claims with live confidence scores  
✅ **Pressure Chart:** Displays real pressure/regulation signals  
✅ **Trace Viewer:** Navigates to actual runtime cycles  
✅ **State Sync:** All components update when runtime changes  
✅ **Performance:** UI updates complete in <100ms  
✅ **Error Handling:** Graceful fallbacks on bridge failures  

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Runtime → UI blocking | Async channels, non-blocking updates |
| Stale state | Signal-based invalidation |
| Memory leaks | Proper cleanup in drop() |
| Performance degradation | Memoization, lazy rendering |
| Type mismatches | Strong typing across bridges |

---

## Deliverables

**Code:**
- 6 new bridge modules
- Updated 6 UI components
- Signal state management
- Error boundaries

**Documentation:**
- PHASE_10_PLAN.md (this file)
- PHASE_10_DISCOVERY.md (implementation findings)
- PHASE_10_SUMMARY.md (completion report)

**Artifacts:**
- Live data proof (runtime + UI integrated)
- Performance metrics
- Integration test results

**Proof Codename:** CODEX-live-runtime-integration

---

## Next Phase (Phase 11)

### Advanced Features
1. UI-to-Runtime commands (claim creation, memory queries)
2. Real-time policy enforcement visualization
3. Performance metrics dashboard
4. Audit trail export
5. Memory introspection tools

---

**Status:** PLANNING  
**Codename:** CODEX-live  
**Target Completion:** Week 3 of Phase 10
