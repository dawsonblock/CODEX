# PHASE 10 DISCOVERY: Runtime Event Stream Analysis

## Available Runtime Events

Discovered 30+ event types emitted by the Rust runtime, organized by subsystem:

### Cycle Management (2 events)
- `CycleStarted` → New cycle begins (cycle_id, timestamp)
- `WorldStateUpdated` → Cycle outcome (resource_delta, social_score, harm_score, etc.)

### Action Selection (5 events)
- `CandidateGenerated` → Action candidate created (action_type, score, reasoning)
- `CandidateRejected` → Candidate failed critic (reason)
- `CandidateSelected` → Final action chosen (score, resonance, reasoning)
- `ActionApplied` → Action executed in sim (conserve mode)
- `ObservationReceived` → External observation entered

### Memory System (4 events)
- `MemoryQueried` → Query issued (query text)
- `MemoryHitReturned` → Results returned (hit_count, top_key, top_value)
- `MemoryWritten` → Entry committed (key name)
- `ScratchpadUpdated` → Working memory changed (entry_count)

### Evidence & Claims (6 events)
- `EvidenceStored` → Evidence added to vault (entry_id, source, confidence, content_hash)
- `EvidenceIntegrityChecked` → Vault validation (total, valid, tampered, all_valid)
- `ClaimAsserted` → New claim created (claim_id, subject, predicate)
- `ClaimValidated` → Claim validated (Unverified → Active)
- `ClaimLifecycleRecorded` → Lifecycle transition (lifecycle_event)
- `ContradictionDetected` → Conflicts identified (claim_a, claim_b, subject)

### Contradiction Resolution (2 events)
- `ContradictionResolved` → Resolution applied (superseded_claim, active_claim)
- `ContradictionDetected` → (see above)

### Archive & Persistence (2 events)
- `ArchiveCommitted` → Frame committed (frame_id, entry_count)
- `RuntimeModeChanged` → Mode transition (from, to)

### System (2 events)
- `ErrorOccurred` → Runtime error (cycle_id, message)
- (Reserved for future expansion)

---

## Data Flow Architecture

### Current (Phase 9)
```
Rust Runtime (per-cycle)
    ↓
RuntimeLoop::run_cycle()
    ↓
RuntimeStepResult (single shot)
    ↓
runtime_client::local_runtime_response()
    ↓
Extract events once
    ↓ (no streaming)
Dioxus UI (static mock)
```

### Target (Phase 10)
```
Rust Runtime (live)
    ↓
EventEnvelope Stream (sequence, timestamp, origin, event)
    ↓
Named bridges (event → UI data mapping)
    ├── TimelineEventBridge
    ├── ClaimStoreBridge
    ├── EvidenceVaultBridge
    └── PressureMetricsBridge
    ↓
Dioxus Signals (reactive state)
    ├── signal_timeline_events
    ├── signal_claims
    ├── signal_evidence
    └── signal_pressure_metrics
    ↓
UI Components (live rendering)
    ├── timeline_viewer
    ├── claim_details_panel
    ├── basis_items_table
    ├── pressure_dynamics_chart
    └── trace_viewer
```

---

## Bridge Design Pattern

### TimelineEventBridge
**Input:** RuntimeEvent  
**Output:** TimelineEvent (cycle, type, claim_ids, evidence_ids, confidence, message)

```rust
pub struct TimelineEventBridge;
impl TimelineEventBridge {
    pub fn from_runtime_event(env: &EventEnvelope) -> Option<TimelineEvent> {
        match &env.event {
            RuntimeEvent::CycleStarted { cycle_id, .. } => Some(TimelineEvent {
                cycle: *cycle_id as usize,
                event_type: "cycle",
                timestamp: env.timestamp,
                // ... extract other fields
            }),
            RuntimeEvent::ClaimAsserted { claim_id, subject, predicate, .. } => Some(TimelineEvent {
                event_type: "claim",
                claim_ids: vec![claim_id.clone()],
                message: format!("{} {} *", subject, predicate),
                // ...
            }),
            // ... other event types
            _ => None,
        }
    }
}
```

### ClaimStoreBridge
**Input:** RuntimeEvent  
**Output:** ClaimDisplay (claim_id, subject, predicate, object, status, evidence_count)

```rust
pub struct ClaimStoreBridge;
impl ClaimStoreBridge {
    pub fn from_runtime_event(env: &EventEnvelope) -> Option<ClaimDisplay> {
        match &env.event {
            RuntimeEvent::ClaimAsserted { claim_id, subject, predicate, .. } => Some(ClaimDisplay {
                claim_id: claim_id.clone(),
                subject: subject.clone(),
                predicate: predicate.clone(),
                grounding_status: GroundingStatus::Unverified,
                // ...
            }),
            RuntimeEvent::ClaimValidated { claim_id } => Some(ClaimDisplay {
                claim_id: claim_id.clone(),
                grounding_status: GroundingStatus::Validated,
                // ...
            }),
            // ...
            _ => None,
        }
    }
}
```

### EvidenceVaultBridge
**Input:** RuntimeEvent  
**Output:** EvidenceDisplay (entry_id, source, confidence, content_hash)

```rust
pub struct EvidenceVaultBridge;
impl EvidenceVaultBridge {
    pub fn from_runtime_event(env: &EventEnvelope) -> Option<EvidenceDisplay> {
        match &env.event {
            RuntimeEvent::EvidenceStored { entry_id, source, confidence, content_hash, .. } => {
                Some(EvidenceDisplay {
                    entry_id: entry_id.clone(),
                    source: source.clone(),
                    confidence: (*confidence * 100.0) as u8,
                    content_hash: content_hash.clone(),
                    // ...
                })
            },
            // ...
            _ => None,
        }
    }
}
```

### PressureMetricsBridge
**Input:** RuntimeEvent (WorldStateUpdated)  
**Output:** PressureMetrics (pressure, regulation, averages, peaks)

```rust
pub struct PressureMetricsBridge;
impl PressureMetricsBridge {
    pub fn calculate_pressure(outcome: &WorldOutcome) -> PressureMetrics {
        // Derived metrics:
        // pressure = 1.0 - (truth_score + logic_score) / 2.0
        // regulation = avg(kindness_score, utility_score, social_score, logic_score)
        PressureMetrics {
            pressure: 1.0 - (outcome.truth_score + outcome.logic_score) / 2.0,
            regulation: (outcome.kindness_score + outcome.utility_score + 
                        outcome.social_score + outcome.logic_score) / 4.0,
            // ...
        }
    }
}
```

---

## State Management Strategy

### Dioxus Signals
```rust
pub struct UIRuntimeState {
    // Timeline
    timeline_events: Signal<Vec<TimelineEvent>>,
    
    // Claims
    claims: Signal<Vec<ClaimDisplay>>,
    claim_by_id: Signal<HashMap<String, ClaimDisplay>>,
    
    // Evidence
    evidence: Signal<Vec<EvidenceDisplay>>,
    evidence_by_id: Signal<HashMap<String, EvidenceDisplay>>,
    
    // Pressure
    pressure_metrics: Signal<Vec<PressureMetrics>>,
    current_pressure: Signal<PressureMetrics>,
    
    // Metadata
    current_cycle: Signal<usize>,
    last_update: Signal<String>,
}
```

### Update Mechanisms

**Poll-based (Option A):**
```rust
use_effect(move || {
    let state = ui_state.clone();
    tokio::spawn(async move {
        loop {
            let events = fetch_new_events().await;
            for env in events {
                state.process_event(&env);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
});
```

**Channel-based (Option B):**
```rust
use_effect(move || {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let state = ui_state.clone();
    
    // Event producer in runtime
    spawn_event_listener(tx);
    
    // Event consumer in UI
    tokio::spawn(async move {
        while let Some(env) = rx.recv().await {
            state.process_event(&env);
        }
    });
});
```

**Signal-based (Option C - Recommended):**
```rust
// In bridge module
pub static TIMELINE_EVENTS: Signal<Vec<TimelineEvent>> = Signal::new(Vec::new());
pub static CLAIMS: Signal<Vec<ClaimDisplay>> = Signal::new(Vec::new());

// In runtime
for env in event_stream {
    if let Some(event) = TimelineEventBridge::from_runtime_event(&env) {
        TIMELINE_EVENTS.write().push(event);
    }
}

// In component
let timeline = TIMELINE_EVENTS.read();
```

---

## Integration Checkpoints

### Step 1 Validation
- [ ] TimelineEventBridge correctly maps 80%+ of event types
- [ ] Event → Timeline event mapping preserves cycle ordering
- [ ] Timeline viewer renders live events without mock data

### Step 2 Validation
- [ ] ClaimStoreBridge maps all claim lifecycle events
- [ ] Grounding status transitions visible in claim_details_panel
- [ ] Evidence links updated when EvidenceStored fires

### Step 3 Validation
- [ ] EvidenceVaultBridge resolves evidence by entry_id
- [ ] Confidence scores render correctly
- [ ] basis_items_table shows real evidence

### Step 4 Validation
- [ ] PressureMetricsBridge calculates correct metrics
- [ ] pressure_dynamics_chart updates on WorldStateUpdated
- [ ] Threshold warnings trigger at correct values

### Step 5 Validation
- [ ] trace_viewer navigates actual cycles
- [ ] Claims per cycle accurate
- [ ] Actions match runtime selection

### Step 6 Validation
- [ ] All signals update synchronously
- [ ] No stale state between components
- [ ] Error boundaries catch bridge failures
- [ ] Performance: UI updates <100ms

---

## Critical Implementation Notes

1. **Event Ordering:** EventEnvelope.sequence ensures causal ordering
2. **Cycle Grouping:** Group events by cycle_id for trace_viewer
3. **Type Safety:** Strong typing across all bridge conversions
4. **Error Recovery:** Bridge failures don't crash UI
5. **Memory Management:** Prune old timeline events periodically
6. **Performance:** Use memoization for expensive calculations

---

## Next Steps

1. Implement `live_event_bridge.rs` module with TimelineEventBridge
2. Create `live_claim_store_bridge.rs` module
3. Create `live_evidence_vault_bridge.rs` module
4. Create `pressure_metrics_bridge.rs` module
5. Create `trace_cycle_bridge.rs` module
6. Create `ui_state.rs` with Signal-based state management
7. Update all 6 Phase 9 components to consume live signals

---

**Discovery Complete**  
**Total Event Types:** 30+  
**Bridge Modules Needed:** 6  
**UI Components to Update:** 6  
**Estimated Impact:** 2,000+ LOC across bridges and state management
