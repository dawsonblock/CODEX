use runtime_core::event::{EventEnvelope, RuntimeEvent};
use crate::bridge::types::{LiveClaimDisplay, GroundingStatus};
use std::collections::HashMap;

/// Maps RuntimeEvent to claim store display data for the claim_details_panel.
pub struct LiveClaimStoreBridge {
    claims: HashMap<String, LiveClaimDisplay>,
}

impl LiveClaimStoreBridge {
    pub fn new() -> Self {
        Self {
            claims: HashMap::new(),
        }
    }

    /// Process a runtime event and update claim store state.
    /// Returns true if the claim state changed, false otherwise.
    pub fn process_event(&mut self, env: &EventEnvelope) -> bool {
        match &env.event {
            RuntimeEvent::ClaimAsserted {
                claim_id,
                subject,
                predicate,
                ..
            } => {
                self.claims.insert(
                    claim_id.clone(),
                    LiveClaimDisplay {
                        claim_id: claim_id.clone(),
                        subject: subject.clone(),
                        predicate: predicate.clone(),
                        object: None,
                        grounding_status: GroundingStatus::Unverified,
                        evidence_count: 0,
                        contradiction_count: 0,
                        confidence_pct: 50,
                        evidence_ids: Vec::new(),
                    },
                );
                true
            }

            RuntimeEvent::ClaimValidated { claim_id, .. } => {
                if let Some(claim) = self.claims.get_mut(claim_id) {
                    claim.grounding_status = GroundingStatus::Validated;
                    claim.confidence_pct = 85;
                    true
                } else {
                    false
                }
            }

            RuntimeEvent::EvidenceStored {
                cycle_id: _,
                entry_id,
                confidence,
                ..
            } => {
                let mut changed = false;
                // Evidence could be linked to multiple claims
                for claim in self.claims.values_mut() {
                    if !claim.evidence_ids.contains(entry_id) {
                        claim.evidence_ids.push(entry_id.clone());
                        claim.evidence_count += 1;
                        // Increase confidence based on evidence quality
                        claim.confidence_pct = std::cmp::min(
                            100,
                            claim.confidence_pct + (confidence * 15.0) as u8,
                        );
                        changed = true;
                    }
                }
                changed
            }

            RuntimeEvent::ContradictionDetected {
                claim_a,
                claim_b,
                ..
            } => {
                let mut changed = false;
                if let Some(claim) = self.claims.get_mut(claim_a) {
                    claim.contradiction_count += 1;
                    changed = true;
                }
                if let Some(claim) = self.claims.get_mut(claim_b) {
                    claim.contradiction_count += 1;
                    changed = true;
                }
                changed
            }

            RuntimeEvent::ContradictionResolved {
                superseded_claim,
                active_claim,
                ..
            } => {
                let mut changed = false;
                if let Some(claim) = self.claims.get_mut(superseded_claim) {
                    claim.grounding_status = GroundingStatus::Contradicted;
                    changed = true;
                }
                if let Some(claim) = self.claims.get_mut(active_claim) {
                    if claim.grounding_status == GroundingStatus::Unverified {
                        claim.grounding_status = GroundingStatus::Validated;
                        claim.confidence_pct = 90;
                        changed = true;
                    }
                }
                changed
            }

            // Events we process but don't directly update claims
            RuntimeEvent::EvidenceIntegrityChecked {
                tampered, ..
            } => {
                if *tampered > 0 {
                    // Mark affected claims as failed
                    for claim in self.claims.values_mut() {
                        if !claim.evidence_ids.is_empty() {
                            claim.grounding_status = GroundingStatus::Failed;
                            claim.confidence_pct = std::cmp::max(10, claim.confidence_pct / 2);
                        }
                    }
                    true
                } else {
                    false
                }
            }

            // Don't update on other events
            _ => false,
        }
    }

    /// Get all claims in display format, sorted by confidence.
    pub fn get_all_claims(&self) -> Vec<LiveClaimDisplay> {
        let mut claims: Vec<_> = self.claims.values().cloned().collect();
        claims.sort_by(|a, b| b.confidence_pct.cmp(&a.confidence_pct));
        claims
    }

    /// Get a specific claim by ID.
    pub fn get_claim(&self, claim_id: &str) -> Option<LiveClaimDisplay> {
        self.claims.get(claim_id).cloned()
    }

    /// Get claims filtered by grounding status.
    pub fn get_claims_by_status(&self, status: GroundingStatus) -> Vec<LiveClaimDisplay> {
        self.claims
            .values()
            .filter(|c| c.grounding_status == status)
            .cloned()
            .collect()
    }

    /// Get statistics about claims.
    pub fn get_statistics(&self) -> ClaimStatistics {
        let total = self.claims.len();
        let validated = self.get_claims_by_status(GroundingStatus::Validated).len();
        let unverified = self.get_claims_by_status(GroundingStatus::Unverified).len();
        let failed = self.get_claims_by_status(GroundingStatus::Failed).len();
        let contradicted = self.get_claims_by_status(GroundingStatus::Contradicted).len();
        let avg_confidence = if total > 0 {
            self.claims
                .values()
                .map(|c| c.confidence_pct as f64)
                .sum::<f64>()
                / total as f64
        } else {
            0.0
        };

        let total_evidence = self.claims
            .values()
            .map(|c| c.evidence_count)
            .sum();

        let total_contradictions = self.claims
            .values()
            .map(|c| c.contradiction_count)
            .sum();

        ClaimStatistics {
            total,
            validated,
            unverified,
            failed,
            contradicted,
            avg_confidence,
            total_evidence,
            total_contradictions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClaimStatistics {
    pub total: usize,
    pub validated: usize,
    pub unverified: usize,
    pub failed: usize,
    pub contradicted: usize,
    pub avg_confidence: f64,
    pub total_evidence: usize,
    pub total_contradictions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::EventOrigin;

    #[test]
    fn test_claim_asserted() {
        let mut bridge = LiveClaimStoreBridge::new();
        let env = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::ClaimAsserted {
                cycle_id: 1,
                claim_id: "cl-001".to_string(),
                subject: "Alice".to_string(),
                predicate: "is_friendly".to_string(),
            },
        );

        assert!(bridge.process_event(&env));
        let claims = bridge.get_all_claims();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].claim_id, "cl-001");
        assert_eq!(claims[0].grounding_status, GroundingStatus::Unverified);
    }

    #[test]
    fn test_claim_validated() {
        let mut bridge = LiveClaimStoreBridge::new();

        // First assert
        let env1 = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::ClaimAsserted {
                cycle_id: 1,
                claim_id: "cl-001".to_string(),
                subject: "Alice".to_string(),
                predicate: "is_friendly".to_string(),
            },
        );
        bridge.process_event(&env1);

        // Then validate
        let env2 = EventEnvelope::new(
            2,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::ClaimValidated {
                cycle_id: 1,
                claim_id: "cl-001".to_string(),
            },
        );
        assert!(bridge.process_event(&env2));

        let claim = bridge.get_claim("cl-001").unwrap();
        assert_eq!(claim.grounding_status, GroundingStatus::Validated);
        assert_eq!(claim.confidence_pct, 85);
    }

    #[test]
    fn test_statistics() {
        let mut bridge = LiveClaimStoreBridge::new();

        for i in 0..3 {
            let env = EventEnvelope::new(
                i as u64,
                EventOrigin::RuntimeLoop,
                RuntimeEvent::ClaimAsserted {
                    cycle_id: 1,
                    claim_id: format!("cl-{:03}", i),
                    subject: "Test".to_string(),
                    predicate: "property".to_string(),
                },
            );
            bridge.process_event(&env);
        }

        let stats = bridge.get_statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.unverified, 3);
        assert_eq!(stats.validated, 0);
    }
}
