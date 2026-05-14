use runtime_core::event::{EventEnvelope, RuntimeEvent};
use crate::bridge::types::EvidenceDisplay;
use std::collections::HashMap;

/// Maps RuntimeEvent to evidence vault display data for basis_items_table and evidence display.
pub struct LiveEvidenceVaultBridge {
    evidence: HashMap<String, EvidenceDisplay>,
}

impl LiveEvidenceVaultBridge {
    pub fn new() -> Self {
        Self {
            evidence: HashMap::new(),
        }
    }

    /// Process a runtime event and update evidence vault state.
    /// Returns true if the vault state changed, false otherwise.
    pub fn process_event(&mut self, env: &EventEnvelope) -> bool {
        match &env.event {
            RuntimeEvent::EvidenceStored {
                entry_id,
                source,
                confidence,
                content_hash,
                ..
            } => {
                let provenance = if source.contains("query") {
                    "query"
                } else if source.contains("memory") || source.contains("recall") {
                    "memory"
                } else {
                    "assertion"
                };

                self.evidence.insert(
                    entry_id.clone(),
                    EvidenceDisplay {
                        entry_id: entry_id.clone(),
                        source: source.clone(),
                        confidence_pct: (*confidence * 100.0) as u8,
                        content_hash: content_hash.clone(),
                        provenance: provenance.to_string(),
                    },
                );
                true
            }

            // Evidence integrity check: no change to individual entries,
            // but validation status could affect display
            RuntimeEvent::EvidenceIntegrityChecked {
                tampered, ..
            } => {
                // If tampering detected, mark evidence as potentially invalid
                if *tampered > 0 {
                    // In a real implementation, we'd mark specific evidence
                    // For now, return true to indicate state change
                    true
                } else {
                    false
                }
            }

            _ => false,
        }
    }

    /// Get all evidence in the vault.
    pub fn get_all_evidence(&self) -> Vec<EvidenceDisplay> {
        let mut evidence: Vec<_> = self.evidence.values().cloned().collect();
        evidence.sort_by(|a, b| b.confidence_pct.cmp(&a.confidence_pct));
        evidence
    }

    /// Get evidence by entry ID.
    pub fn get_evidence(&self, entry_id: &str) -> Option<EvidenceDisplay> {
        self.evidence.get(entry_id).cloned()
    }

    /// Get evidence filtered by provenance.
    pub fn get_evidence_by_provenance(&self, provenance: &str) -> Vec<EvidenceDisplay> {
        self.evidence
            .values()
            .filter(|e| e.provenance == provenance)
            .cloned()
            .collect()
    }

    /// Get evidence sorted by confidence.
    pub fn get_evidence_by_confidence(&self, min_confidence: u8) -> Vec<EvidenceDisplay> {
        let mut evidence: Vec<_> = self
            .evidence
            .values()
            .filter(|e| e.confidence_pct >= min_confidence)
            .cloned()
            .collect();
        evidence.sort_by(|a, b| b.confidence_pct.cmp(&a.confidence_pct));
        evidence
    }

    /// Resolve evidence for a specific claim (by ID list).
    pub fn resolve_evidence_for_claim(&self, evidence_ids: &[String]) -> Vec<EvidenceDisplay> {
        evidence_ids
            .iter()
            .filter_map(|id| self.get_evidence(id))
            .collect()
    }

    /// Get statistics about the evidence vault.
    pub fn get_statistics(&self) -> EvidenceStatistics {
        let total = self.evidence.len();
        let avg_confidence = if total > 0 {
            self.evidence
                .values()
                .map(|e| e.confidence_pct as f64)
                .sum::<f64>()
                / total as f64
        } else {
            0.0
        };

        let mut provenance_counts = std::collections::HashMap::new();
        for evidence in self.evidence.values() {
            *provenance_counts
                .entry(evidence.provenance.clone())
                .or_insert(0) += 1;
        }

        let high_confidence = self.evidence
            .values()
            .filter(|e| e.confidence_pct >= 80)
            .count();

        let low_confidence = self.evidence
            .values()
            .filter(|e| e.confidence_pct < 50)
            .count();

        EvidenceStatistics {
            total,
            avg_confidence,
            high_confidence,
            low_confidence,
            provenance_distribution: provenance_counts,
        }
    }

    /// Get evidence content hash for integrity verification.
    pub fn verify_integrity(&self, entry_id: &str, expected_hash: &str) -> bool {
        if let Some(evidence) = self.get_evidence(entry_id) {
            evidence.content_hash == expected_hash
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceStatistics {
    pub total: usize,
    pub avg_confidence: f64,
    pub high_confidence: usize,
    pub low_confidence: usize,
    pub provenance_distribution: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::EventOrigin;

    #[test]
    fn test_evidence_stored() {
        let mut bridge = LiveEvidenceVaultBridge::new();
        let env = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::EvidenceStored {
                cycle_id: 1,
                entry_id: "ev-001".to_string(),
                source: "memory_query".to_string(),
                confidence: 0.85,
                content_hash: "abc123".to_string(),
            },
        );

        assert!(bridge.process_event(&env));
        let evidence = bridge.get_all_evidence();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].entry_id, "ev-001");
        assert_eq!(evidence[0].confidence_pct, 85);
        assert_eq!(evidence[0].provenance, "query");
    }

    #[test]
    fn test_evidence_provenance_classification() {
        let mut bridge = LiveEvidenceVaultBridge::new();

        // Query evidence
        let env1 = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::EvidenceStored {
                cycle_id: 1,
                entry_id: "ev-001".to_string(),
                source: "search_query_result".to_string(),
                confidence: 0.8,
                content_hash: "hash1".to_string(),
            },
        );
        bridge.process_event(&env1);

        // Memory evidence
        let env2 = EventEnvelope::new(
            2,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::EvidenceStored {
                cycle_id: 1,
                entry_id: "ev-002".to_string(),
                source: "memory_recall".to_string(),
                confidence: 0.9,
                content_hash: "hash2".to_string(),
            },
        );
        bridge.process_event(&env2);

        let query_evidence = bridge.get_evidence_by_provenance("query");
        let memory_evidence = bridge.get_evidence_by_provenance("memory");

        assert_eq!(query_evidence.len(), 1);
        assert_eq!(memory_evidence.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut bridge = LiveEvidenceVaultBridge::new();

        for i in 0..5 {
            let confidence = if i < 3 { 0.9 } else { 0.4 };
            let env = EventEnvelope::new(
                i as u64,
                EventOrigin::RuntimeLoop,
                RuntimeEvent::EvidenceStored {
                    cycle_id: 1,
                    entry_id: format!("ev-{:03}", i),
                    source: "test_source".to_string(),
                    confidence,
                    content_hash: format!("hash{}", i),
                },
            );
            bridge.process_event(&env);
        }

        let stats = bridge.get_statistics();
        assert_eq!(stats.total, 5);
        assert_eq!(stats.high_confidence, 3);
        assert_eq!(stats.low_confidence, 2);
    }
}
