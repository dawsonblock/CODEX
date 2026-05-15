use runtime_core::event::{EventEnvelope, RuntimeEvent};
use runtime_core::action::ActionType;
use crate::bridge::types::TimelineEvent;

/// Maps RuntimeEvent to timeline visualization events for the timeline viewer component.
pub struct TimelineEventBridge;

impl TimelineEventBridge {
    /// Convert a RuntimeEvent into a timeline visualization event.
    /// Returns None if the event should not be rendered on the timeline.
    pub fn from_envelope(env: &EventEnvelope) -> Option<TimelineEvent> {
        let cycle = env.cycle_id().unwrap_or(0) as usize;
        let timestamp = env.timestamp.format("%H:%M:%S%.3f").to_string();

        match &env.event {
            // Cycle management
            RuntimeEvent::CycleStarted { .. } => Some(TimelineEvent {
                cycle,
                event_type: "cycle".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: 1.0,
                message: format!("● Cycle {} started", cycle),
            }),

            // Action selection
            RuntimeEvent::CandidateSelected {
                action_type,
                score,
                reasoning: _,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "answer".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: *score,
                message: format!(
                    "✓ Action {} selected (score: {:.2})",
                    action_type_label(action_type),
                    score
                ),
            }),

            RuntimeEvent::ActionApplied { action_type, .. } => Some(TimelineEvent {
                cycle,
                event_type: "complete".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: 0.8,
                message: format!("→ Action {} applied", action_type_label(action_type)),
            }),

            // Memory system
            RuntimeEvent::MemoryQueried { query, .. } => Some(TimelineEvent {
                cycle,
                event_type: "query".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: 0.7,
                message: format!("? Memory query: {}", truncate_text(query, 50)),
            }),

            RuntimeEvent::MemoryHitReturned {
                hit_count,
                top_key,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "evidence".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: (*hit_count as f64 / 10.0).min(1.0),
                message: format!(
                    "📄 Memory hit: {} result(s) {}",
                    hit_count,
                    top_key
                        .as_ref()
                        .map(|k| format!("({})", k))
                        .unwrap_or_default()
                ),
            }),

            // Claims and evidence
            RuntimeEvent::ClaimAsserted {
                claim_id,
                subject,
                predicate,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "claim".to_string(),
                timestamp,
                claim_ids: vec![claim_id.clone()],
                evidence_ids: vec![],
                confidence: 0.5,
                message: format!("◆ Claim {}: {} {}", claim_id, subject, predicate),
            }),

            RuntimeEvent::ClaimValidated { claim_id, .. } => Some(TimelineEvent {
                cycle,
                event_type: "claim".to_string(),
                timestamp,
                claim_ids: vec![claim_id.clone()],
                evidence_ids: vec![],
                confidence: 0.85,
                message: format!("✅ Claim {} validated", claim_id),
            }),

            RuntimeEvent::EvidenceStored {
                entry_id,
                source,
                confidence,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "evidence".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![entry_id.clone()],
                confidence: *confidence,
                message: format!(
                    "📎 Evidence {}: {} ({:.0}%)",
                    entry_id,
                    truncate_text(source, 40),
                    confidence * 100.0
                ),
            }),

            // Contradictions
            RuntimeEvent::ContradictionDetected {
                claim_a,
                claim_b,
                subject,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "contradiction".to_string(),
                timestamp,
                claim_ids: vec![claim_a.clone(), claim_b.clone()],
                evidence_ids: vec![],
                confidence: 0.0,
                message: format!(
                    "⚠️  Contradiction: {} in claims {} and {}",
                    subject, claim_a, claim_b
                ),
            }),

            RuntimeEvent::ContradictionResolved {
                superseded_claim,
                active_claim,
                ..
            } => Some(TimelineEvent {
                cycle,
                event_type: "complete".to_string(),
                timestamp,
                claim_ids: vec![
                    superseded_claim.clone(),
                    active_claim.clone(),
                ],
                evidence_ids: vec![],
                confidence: 0.75,
                message: format!(
                    "→ Contradiction resolved: {} supersedes {}",
                    active_claim, superseded_claim
                ),
            }),

            // World state
            RuntimeEvent::WorldStateUpdated { outcome, .. } => Some(TimelineEvent {
                cycle,
                event_type: "pressure".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: 1.0 - outcome.harm_score,
                message: format!(
                    "🌍 World update: resources {:.2}, truth {:.2}, harm {:.2}",
                    outcome.resource_delta, outcome.truth_score, outcome.harm_score
                ),
            }),

            // Error reporting
            RuntimeEvent::ErrorOccurred { message, .. } => Some(TimelineEvent {
                cycle,
                event_type: "contradiction".to_string(),
                timestamp,
                claim_ids: vec![],
                evidence_ids: vec![],
                confidence: 0.0,
                message: format!("❌ Error: {}", truncate_text(message, 60)),
            }),

            // Archive and mode changes (skip)
            RuntimeEvent::ArchiveCommitted { .. } | RuntimeEvent::RuntimeModeChanged { .. } => {
                None
            }

            // Other events (skip rendering)
            _ => None,
        }
    }

    /// Build a timeline event list from multiple envelopes.
    /// Events are ordered by sequence number to preserve causality.
    pub fn build_timeline(envelopes: &[EventEnvelope]) -> Vec<TimelineEvent> {
        envelopes
            .iter()
            .filter_map(Self::from_envelope)
            .collect()
    }
}

/// Get a human-readable label for an action type.
fn action_type_label(action: &ActionType) -> String {
    match action {
        ActionType::Answer => "answer",
        ActionType::AskClarification => "ask_clarification",
        ActionType::RetrieveMemory => "retrieve_memory",
        ActionType::RefuseUnsafe => "refuse_unsafe",
        ActionType::DeferInsufficientEvidence => "defer",
        ActionType::Summarize => "summarize",
        ActionType::Plan => "plan",
        ActionType::ExecuteBoundedTool => "execute_tool",
        ActionType::NoOp => "no_op",
        ActionType::InternalDiagnostic => "diagnostic",
    }
    .to_string()
}

/// Truncate text to a maximum length with ellipsis.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}…", &text[..max_len - 1])
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_event_from_claim_asserted() {
        let env = EventEnvelope::new(
            1,
            runtime_core::event::EventOrigin::RuntimeLoop,
            RuntimeEvent::ClaimAsserted {
                cycle_id: 1,
                claim_id: "cl-001".to_string(),
                subject: "Alice".to_string(),
                predicate: "is_friendly".to_string(),
            },
        );

        let event = TimelineEventBridge::from_envelope(&env);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event_type, "claim");
        assert_eq!(event.claim_ids, vec!["cl-001"]);
    }

    #[test]
    fn test_timeline_skip_archive_events() {
        let env = EventEnvelope::new(
            1,
            runtime_core::event::EventOrigin::RuntimeLoop,
            RuntimeEvent::ArchiveCommitted {
                cycle_id: 1,
                frame_id: "f-001".to_string(),
                entry_count: 10,
            },
        );

        let event = TimelineEventBridge::from_envelope(&env);
        assert!(event.is_none());
    }
}
