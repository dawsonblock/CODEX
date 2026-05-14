use crate::action::ActionType;
use crate::types::ResonanceEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Structured outcome from the simworld after an action is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldOutcome {
    pub resource_delta: f64,
    pub social_score: f64,
    pub harm_score: f64,
    pub truth_score: f64,
    pub kindness_score: f64,
    pub logic_score: f64,
    pub utility_score: f64,
    pub matches_expected: bool,
}

/// Snapshot of provider counters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderCountersSnapshot {
    pub local_requests: u64,
    pub local_successes: u64,
    pub local_failures: u64,
    pub local_disabled_blocks: u64,
    pub cloud_requests: u64,
    pub external_requests: u64,
    pub feature_enabled: bool,
}

/// Provenance marker for where an event was emitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventOrigin {
    RuntimeLoop,
    Evaluator,
    ClaimStore,
    ToolGate,
    ProofHarness,
}

/// Typed wrapper around a [`RuntimeEvent`] with provenance metadata.
///
/// `EventEnvelope` is the at-rest representation written to the event log.  Each
/// envelope carries a monotonically increasing sequence number (within a session),
/// the wall-clock timestamp at emission time, and the subsystem origin so that
/// readers can filter, audit, or replay events by provenance without unpacking the
/// inner payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Monotonically increasing sequence number within a session.
    pub sequence: u64,
    /// Wall-clock timestamp when the event was emitted (UTC).
    pub timestamp: DateTime<Utc>,
    /// The subsystem that emitted this event.
    pub origin: EventOrigin,
    /// The event payload.
    pub event: RuntimeEvent,
}

impl EventEnvelope {
    /// Construct a new envelope, capturing `Utc::now()` as the timestamp.
    pub fn new(sequence: u64, origin: EventOrigin, event: RuntimeEvent) -> Self {
        Self {
            sequence,
            timestamp: Utc::now(),
            origin,
            event,
        }
    }

    /// Delegate to the inner event's cycle-ID helper.
    pub fn cycle_id(&self) -> Option<u64> {
        self.event.cycle_id()
    }
}

/// All events that can be appended to the event log.
/// Variants are tagged in JSONL as `{"type": "...", "payload": {...}}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RuntimeEvent {
    CycleStarted {
        cycle_id: u64,
        timestamp: DateTime<Utc>,
    },
    ObservationReceived {
        cycle_id: u64,
        observation_len: usize,
        world_resources: f64,
    },
    /// A memory query was issued.
    MemoryQueried {
        cycle_id: u64,
        query: String,
    },
    /// A memory retrieval returned hits.
    MemoryHitReturned {
        cycle_id: u64,
        hit_count: usize,
        top_key: Option<String>,
        top_value: Option<String>,
    },
    CandidateGenerated {
        cycle_id: u64,
        action_type: ActionType,
        score: f64,
        reasoning: Option<String>,
    },
    /// A candidate was rejected by the critic.
    CandidateRejected {
        cycle_id: u64,
        action_type: ActionType,
        reason: String,
    },
    CandidateSelected {
        cycle_id: u64,
        action_type: ActionType,
        score: f64,
        resonance: Vec<ResonanceEntry>,
        reasoning: Option<String>,
    },
    ActionApplied {
        cycle_id: u64,
        action_type: ActionType,
        conserve: bool,
    },
    WorldStateUpdated {
        cycle_id: u64,
        outcome: WorldOutcome,
    },
    RuntimeModeChanged {
        from: String,
        to: String,
    },
    MemoryWritten {
        cycle_id: u64,
        key: String,
    },
    ScratchpadUpdated {
        cycle_id: u64,
        entry_count: usize,
    },
    ErrorOccurred {
        cycle_id: u64,
        message: String,
    },
    /// An archive commit was performed.
    ArchiveCommitted {
        cycle_id: u64,
        frame_id: String,
        entry_count: usize,
    },
    /// A contradiction was detected between two claims.
    ContradictionDetected {
        cycle_id: u64,
        claim_a: String,
        claim_b: String,
        subject: String,
    },
    /// Evidence was stored in the evidence vault.
    EvidenceStored {
        cycle_id: u64,
        entry_id: String,
        source: String,
        confidence: f64,
        content_hash: String,
    },
    /// An evidence vault integrity check was performed.
    EvidenceIntegrityChecked {
        cycle_id: u64,
        total: usize,
        valid: usize,
        tampered: usize,
        all_valid: bool,
    },
    /// A contradiction was resolved (old claim superseded).
    ContradictionResolved {
        cycle_id: u64,
        superseded_claim: String,
        active_claim: String,
        resolution: String,
    },
    /// A new claim was asserted (enters as Unverified).
    ClaimAsserted {
        cycle_id: u64,
        claim_id: String,
        subject: String,
        predicate: String,
    },
    /// A claim was validated (Unverified → Active).
    ClaimValidated {
        cycle_id: u64,
        claim_id: String,
    },
    /// Lifecycle transition emitted with explicit origin metadata.
    ClaimLifecycleRecorded {
        cycle_id: u64,
        claim_id: String,
        lifecycle_event: String,
        event_origin: EventOrigin,
    },
    /// A claim was retrieved for action scoring.
    ClaimRetrieved {
        cycle_id: u64,
        claim_id: String,
        subject: String,
        predicate: String,
        object: Option<String>,
        evidence_id: Option<String>,
        status: String,
        confidence: f64,
    },
    /// A claim was superseded by a newer claim (Active → Superseded).
    ClaimSuperseded {
        cycle_id: u64,
        old_claim_id: String,
        new_claim_id: String,
    },
    /// A contradiction was escalated (requires human attention).
    ContradictionEscalated {
        cycle_id: u64,
        contradiction_id: String,
        reason: String,
    },
    /// A contradiction pass was executed over retrieved claims.
    ContradictionChecked {
        cycle_id: u64,
        checked_claim_ids: Vec<String>,
        contradiction_ids: Vec<String>,
        active_contradictions: usize,
    },
    /// Governed-memory live admission advisory decision for a candidate.
    GovernedMemoryAdmissionEvaluated {
        cycle_id: u64,
        candidate_id: String,
        decision_kind: String,
        reason_codes: Vec<String>,
        confidence: f64,
        source_trust_score: f64,
        live_hook: bool,
        claimstore_writer: String,
        governed_memory_writer: bool,
        claim_written: bool,
        override_applied: bool,
    },
    /// Governed-memory retrieval intent routing/planning advisory event.
    GovernedMemoryRetrievalPlanned {
        cycle_id: u64,
        query_id: String,
        intent_category: String,
        recommended_action: String,
        reason_codes: Vec<String>,
    },
    /// A reasoning audit was generated for a cycle.
    ReasoningAuditGenerated {
        cycle_id: u64,
        audit_id: String,
        selected_action: String,
        evidence_ids: Vec<String>,
        claim_ids: Vec<String>,
        contradiction_ids: Vec<String>,
        dominant_pressures: Vec<String>,
        audit_text: String,
    },
    /// A tool was executed (policy permitted).
    ToolExecuted {
        cycle_id: u64,
        tool_id: String,
        permitted: bool,
        error: Option<String>,
    },
    /// A tool execution was blocked by policy.
    ToolExecutionBlocked {
        cycle_id: u64,
        tool_id: String,
        reason: String,
    },
    /// An operational pressure field was updated.
    PressureUpdated {
        cycle_id: u64,
        field: String,
        old_value: f64,
        new_value: f64,
        source: String,
        reason: String,
    },
    /// Policy bias was applied from pressure state.
    PolicyBiasApplied {
        cycle_id: u64,
        dominant_pressures: Vec<String>,
        selected_action: String,
    },
    // ── Symbolic events ─────────────────────────────────────────────────
    /// A symbol was activated in the symbolic graph.
    SymbolActivated {
        cycle_id: u64,
        symbol_id: String,
        glyph: String,
        activation: f64,
    },
    /// Two symbols were linked in the symbolic graph.
    SymbolLinked {
        cycle_id: u64,
        source_id: String,
        target_id: String,
        edge_kind: String,
    },
    /// A symbolic trace was recorded (frame written).
    SymbolicTraceRecorded {
        cycle_id: u64,
        frame_id: String,
        symbol_count: usize,
        edge_count: usize,
    },
    /// A concept blend was generated by the creative stream.
    ConceptBlendGenerated {
        cycle_id: u64,
        blend_id: String,
        principle_key: String,
        action_type: ActionType,
    },
    /// A principle was extracted from memory.
    PrincipleExtracted {
        cycle_id: u64,
        principle_key: String,
        source_frame: Option<String>,
        confidence: f64,
    },
    /// Symbolic compression was applied (old frame reduced).
    SymbolicCompressionApplied {
        cycle_id: u64,
        source_frame_id: String,
        compressed_frame_id: String,
        compression_ratio: f64,
    },
    /// A resonance score was computed for a candidate.
    ResonanceScoreComputed {
        cycle_id: u64,
        action_type: ActionType,
        entries: Vec<ResonanceEntry>,
        total_score: f64,
    },
    /// Periodic or per-message report of live provider counters.
    ProviderCountersReported {
        cycle_id: u64,
        snapshot: ProviderCountersSnapshot,
    },
    /// An answer envelope was built from claim-grounded context.
    AnswerEnvelopeBuilt {
        cycle_id: u64,
        cited_claim_ids: Vec<String>,
        warning_count: usize,
        confidence: f64,
        event_origin: EventOrigin,
    },
}

impl RuntimeEvent {
    /// The cycle ID carried by this event, if any.
    pub fn cycle_id(&self) -> Option<u64> {
        match self {
            Self::CycleStarted { cycle_id, .. }
            | Self::ObservationReceived { cycle_id, .. }
            | Self::MemoryQueried { cycle_id, .. }
            | Self::MemoryHitReturned { cycle_id, .. }
            | Self::CandidateGenerated { cycle_id, .. }
            | Self::CandidateRejected { cycle_id, .. }
            | Self::CandidateSelected { cycle_id, .. }
            | Self::ActionApplied { cycle_id, .. }
            | Self::WorldStateUpdated { cycle_id, .. }
            | Self::MemoryWritten { cycle_id, .. }
            | Self::ScratchpadUpdated { cycle_id, .. }
            | Self::ErrorOccurred { cycle_id, .. }
            | Self::ArchiveCommitted { cycle_id, .. }
            | Self::EvidenceStored { cycle_id, .. }
            | Self::EvidenceIntegrityChecked { cycle_id, .. }
            | Self::ContradictionDetected { cycle_id, .. }
            | Self::ContradictionResolved { cycle_id, .. }
            | Self::ClaimAsserted { cycle_id, .. }
            | Self::ClaimValidated { cycle_id, .. }
            | Self::ClaimLifecycleRecorded { cycle_id, .. }
            | Self::ClaimRetrieved { cycle_id, .. }
            | Self::ClaimSuperseded { cycle_id, .. }
            | Self::ContradictionEscalated { cycle_id, .. }
            | Self::ContradictionChecked { cycle_id, .. }
            | Self::GovernedMemoryAdmissionEvaluated { cycle_id, .. }
            | Self::GovernedMemoryRetrievalPlanned { cycle_id, .. }
            | Self::ReasoningAuditGenerated { cycle_id, .. }
            | Self::ToolExecuted { cycle_id, .. }
            | Self::ToolExecutionBlocked { cycle_id, .. }
            | Self::PressureUpdated { cycle_id, .. }
            | Self::PolicyBiasApplied { cycle_id, .. }
            | Self::SymbolActivated { cycle_id, .. }
            | Self::SymbolLinked { cycle_id, .. }
            | Self::SymbolicTraceRecorded { cycle_id, .. }
            | Self::ConceptBlendGenerated { cycle_id, .. }
            | Self::PrincipleExtracted { cycle_id, .. }
            | Self::SymbolicCompressionApplied { cycle_id, .. }
            | Self::ResonanceScoreComputed { cycle_id, .. }
            | Self::ProviderCountersReported { cycle_id, .. }
            | Self::AnswerEnvelopeBuilt { cycle_id, .. } => Some(*cycle_id),
            Self::RuntimeModeChanged { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn cycle_started(id: u64) -> RuntimeEvent {
        RuntimeEvent::CycleStarted {
            cycle_id: id,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn event_envelope_carries_sequence_and_origin() {
        let env = EventEnvelope::new(42, EventOrigin::Evaluator, cycle_started(1));
        assert_eq!(env.sequence, 42);
        assert_eq!(env.origin, EventOrigin::Evaluator);
    }

    #[test]
    fn event_envelope_timestamp_is_utc() {
        let before = Utc::now();
        let env = EventEnvelope::new(0, EventOrigin::RuntimeLoop, cycle_started(1));
        let after = Utc::now();
        assert!(env.timestamp >= before);
        assert!(env.timestamp <= after);
    }

    #[test]
    fn event_envelope_cycle_id_delegates_to_inner_event() {
        let env = EventEnvelope::new(1, EventOrigin::ClaimStore, cycle_started(7));
        assert_eq!(env.cycle_id(), Some(7));
    }

    #[test]
    fn event_envelope_mode_changed_has_no_cycle_id() {
        let env = EventEnvelope::new(
            2,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::RuntimeModeChanged {
                from: "Normal".to_string(),
                to: "SafeMode".to_string(),
            },
        );
        assert_eq!(env.cycle_id(), None);
    }

    #[test]
    fn event_envelope_roundtrips_json() {
        let env = EventEnvelope::new(99, EventOrigin::ProofHarness, cycle_started(5));
        let json = serde_json::to_string(&env).expect("serialize");
        let decoded: EventEnvelope = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.sequence, 99);
        assert_eq!(decoded.origin, EventOrigin::ProofHarness);
    }
}
