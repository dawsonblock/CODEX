use runtime_core::event::{EventEnvelope, RuntimeEvent};
use crate::bridge::types::{LiveClaimDisplay, GroundingStatus};
use std::collections::HashMap;

/// Maps RuntimeEvent to cycle-aware trace data for the trace_viewer component.
pub struct TraceCycleBridge {
    /// Claims and actions grouped by cycle.
    cycles: HashMap<usize, CycleTrace>,
}

#[derive(Debug, Clone)]
pub struct CycleTrace {
    pub cycle: usize,
    pub claims: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub action_type: Option<String>,
    pub confidence: f64,
    pub message: String,
}

impl TraceCycleBridge {
    pub fn new() -> Self {
        Self {
            cycles: HashMap::new(),
        }
    }

    /// Process a runtime event and build cycle trace data.
    pub fn process_event(&mut self, env: &EventEnvelope) {
        let cycle = env.cycle_id().unwrap_or(0) as usize;
        let entry = self.cycles.entry(cycle).or_insert_with(|| CycleTrace {
            cycle,
            claims: Vec::new(),
            evidence_ids: Vec::new(),
            action_type: None,
            confidence: 0.5,
            message: format!("Cycle {}", cycle),
        });

        match &env.event {
            RuntimeEvent::ClaimAsserted {
                claim_id,
                subject,
                predicate,
                ..
            } => {
                if !entry.claims.contains(claim_id) {
                    entry.claims.push(claim_id.clone());
                }
                entry.message = format!("{} - Claim: {} {}", entry.message, subject, predicate);
            }

            RuntimeEvent::ClaimValidated { claim_id, .. } => {
                if !entry.claims.contains(claim_id) {
                    entry.claims.push(claim_id.clone());
                }
            }

            RuntimeEvent::EvidenceStored {
                entry_id,
                confidence,
                ..
            } => {
                if !entry.evidence_ids.contains(entry_id) {
                    entry.evidence_ids.push(entry_id.clone());
                }
                entry.confidence = (*confidence).max(entry.confidence);
            }

            RuntimeEvent::CandidateSelected {
                action_type,
                score,
                ..
            } => {
                entry.action_type = Some(format!("{:?}", action_type));
                entry.confidence = *score;
            }

            RuntimeEvent::ActionApplied { action_type, .. } => {
                entry.action_type = Some(format!("{:?}", action_type));
            }

            RuntimeEvent::WorldStateUpdated { outcome, .. } => {
                entry.confidence = outcome.truth_score;
                entry.message = format!(
                    "{} - Truth: {:.2}, Logic: {:.2}",
                    entry.message, outcome.truth_score, outcome.logic_score
                );
            }

            _ => {}
        }
    }

    /// Get cycle trace by number.
    pub fn get_cycle(&self, cycle: usize) -> Option<CycleTrace> {
        self.cycles.get(&cycle).cloned()
    }

    /// Get all cycles sorted by cycle number.
    pub fn get_all_cycles(&self) -> Vec<CycleTrace> {
        let mut cycles: Vec<_> = self.cycles.values().cloned().collect();
        cycles.sort_by_key(|c| c.cycle);
        cycles
    }

    /// Get cycles with specific claim.
    pub fn get_cycles_with_claim(&self, claim_id: &str) -> Vec<CycleTrace> {
        self.cycles
            .values()
            .filter(|c| c.claims.contains(&claim_id.to_string()))
            .cloned()
            .collect()
    }

    /// Get cycles with specific evidence.
    pub fn get_cycles_with_evidence(&self, evidence_id: &str) -> Vec<CycleTrace> {
        self.cycles
            .values()
            .filter(|c| c.evidence_ids.contains(&evidence_id.to_string()))
            .cloned()
            .collect()
    }

    /// Get action timeline.
    pub fn get_action_timeline(&self) -> Vec<(usize, String, f64)> {
        let mut timeline: Vec<_> = self.cycles
            .values()
            .filter_map(|c| {
                c.action_type.as_ref().map(|action| {
                    (c.cycle, action.clone(), c.confidence)
                })
            })
            .collect();
        timeline.sort_by_key(|t| t.0);
        timeline
    }

    /// Get trace statistics.
    pub fn get_statistics(&self) -> TraceStatistics {
        let total_cycles = self.cycles.len();
        let cycles_with_actions = self.cycles
            .values()
            .filter(|c| c.action_type.is_some())
            .count();
        let total_claims: usize = self.cycles
            .values()
            .map(|c| c.claims.len())
            .sum();
        let total_evidence: usize = self.cycles
            .values()
            .map(|c| c.evidence_ids.len())
            .sum();
        let avg_confidence = if total_cycles > 0 {
            self.cycles
                .values()
                .map(|c| c.confidence)
                .sum::<f64>()
                / total_cycles as f64
        } else {
            0.0
        };

        TraceStatistics {
            total_cycles,
            cycles_with_actions,
            total_claims,
            total_evidence,
            avg_confidence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceStatistics {
    pub total_cycles: usize,
    pub cycles_with_actions: usize,
    pub total_claims: usize,
    pub total_evidence: usize,
    pub avg_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::EventOrigin;
    use runtime_core::action::ActionType;

    #[test]
    fn test_cycle_trace_building() {
        let mut bridge = TraceCycleBridge::new();

        let env1 = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::ClaimAsserted {
                cycle_id: 1,
                claim_id: "cl-001".to_string(),
                subject: "Test".to_string(),
                predicate: "property".to_string(),
            },
        );
        bridge.process_event(&env1);

        let env2 = EventEnvelope::new(
            2,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::CandidateSelected {
                cycle_id: 1,
                action_type: ActionType::Answer,
                score: 0.9,
                resonance: vec![],
                reasoning: None,
            },
        );
        bridge.process_event(&env2);

        let cycle_trace = bridge.get_cycle(1).unwrap();
        assert_eq!(cycle_trace.cycle, 1);
        assert_eq!(cycle_trace.claims.len(), 1);
        assert_eq!(cycle_trace.action_type, Some("Answer".to_string()));
    }

    #[test]
    fn test_statistics() {
        let mut bridge = TraceCycleBridge::new();

        for i in 0..3 {
            let env = EventEnvelope::new(
                i as u64,
                EventOrigin::RuntimeLoop,
                RuntimeEvent::ClaimAsserted {
                    cycle_id: i as u64,
                    claim_id: format!("cl-{}", i),
                    subject: "Test".to_string(),
                    predicate: "prop".to_string(),
                },
            );
            bridge.process_event(&env);
        }

        let stats = bridge.get_statistics();
        assert_eq!(stats.total_cycles, 3);
    }
}
