//! Concrete memory-provider contracts shared by in-memory and durable stores.

use crate::claim_store::ClaimStore;
use crate::durable_memory_provider::{ClaimStatus as DurableClaimStatus, DurableMemoryProvider};
use crate::status_mapping::durable_to_memory;
use crate::ClaimStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuery {
    pub observation: String,
    pub status: Option<ClaimStatus>,
    pub min_confidence: Option<f64>,
    pub limit: usize,
}

impl MemoryQuery {
    pub fn new(observation: impl Into<String>) -> Self {
        Self {
            observation: observation.into(),
            status: None,
            min_confidence: None,
            limit: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHit {
    pub claim_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub status: ClaimStatus,
    pub confidence: f64,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryProviderError {
    #[error("durable provider error: {0}")]
    Durable(String),
}

pub trait MemoryProvider: Send + Sync {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError>;
    fn get_by_status(
        &self,
        status: ClaimStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError>;
}

impl MemoryProvider for ClaimStore {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let obs = query.observation.to_lowercase();
        let mut hits = self
            .all_claims()
            .filter(|claim| {
                let status_ok = query.status.is_none_or(|status| claim.status == status);
                let confidence_ok = query
                    .min_confidence
                    .is_none_or(|min| claim.confidence >= min);
                let text = format!(
                    "{} {} {}",
                    claim.subject,
                    claim.predicate,
                    claim.object.clone().unwrap_or_default()
                )
                .to_lowercase();
                let relevance_ok =
                    text.contains(&obs) || obs.contains(&claim.subject.to_lowercase());
                status_ok && confidence_ok && relevance_ok
            })
            .map(|claim| MemoryHit {
                claim_id: claim.id.clone(),
                subject: claim.subject.clone(),
                predicate: claim.predicate.clone(),
                object: claim.object.clone(),
                status: claim.status,
                confidence: claim.confidence,
                evidence_ids: claim.evidence_ids.clone(),
            })
            .collect::<Vec<_>>();

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.limit);
        Ok(hits)
    }

    fn get_by_status(
        &self,
        status: ClaimStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let mut hits = self
            .query_by_status(status)
            .into_iter()
            .map(|claim| MemoryHit {
                claim_id: claim.id.clone(),
                subject: claim.subject.clone(),
                predicate: claim.predicate.clone(),
                object: claim.object.clone(),
                status: claim.status,
                confidence: claim.confidence,
                evidence_ids: claim.evidence_ids.clone(),
            })
            .collect::<Vec<_>>();

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

impl MemoryProvider for DurableMemoryProvider {
    fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let statuses = if let Some(status) = query.status {
            vec![status]
        } else {
            vec![
                ClaimStatus::Unverified,
                ClaimStatus::Active,
                ClaimStatus::Contradicted,
                ClaimStatus::Superseded,
            ]
        };

        let mut hits = Vec::new();
        for status in statuses {
            let durable_statuses: Vec<DurableClaimStatus> = match status {
                ClaimStatus::Unverified => {
                    vec![DurableClaimStatus::Asserted, DurableClaimStatus::Unknown]
                }
                ClaimStatus::Active => vec![DurableClaimStatus::Validated],
                ClaimStatus::Contradicted => {
                    vec![DurableClaimStatus::Rejected, DurableClaimStatus::Disputed]
                }
                ClaimStatus::Superseded => {
                    vec![DurableClaimStatus::Superseded, DurableClaimStatus::Stale]
                }
            };

            for durable_status in durable_statuses {
                let rows = self
                    .get_by_status(durable_status, query.limit)
                    .map_err(|e| MemoryProviderError::Durable(e.to_string()))?;
                for row in rows {
                    if query
                        .min_confidence
                        .is_some_and(|min| row.confidence < min as f32)
                    {
                        continue;
                    }
                    if !query.observation.is_empty()
                        && !row
                            .claim_text
                            .to_lowercase()
                            .contains(&query.observation.to_lowercase())
                    {
                        continue;
                    }
                    hits.push(MemoryHit {
                        claim_id: row.claim_id,
                        subject: row.claim_text,
                        predicate: "matches".to_string(),
                        object: None,
                        status: durable_to_memory(row.status),
                        confidence: row.confidence as f64,
                        evidence_ids: Vec::new(),
                    });
                }
            }
        }

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.limit);
        Ok(hits)
    }

    fn get_by_status(
        &self,
        status: ClaimStatus,
        limit: usize,
    ) -> Result<Vec<MemoryHit>, MemoryProviderError> {
        let durable_statuses: Vec<DurableClaimStatus> = match status {
            ClaimStatus::Unverified => {
                vec![DurableClaimStatus::Asserted, DurableClaimStatus::Unknown]
            }
            ClaimStatus::Active => vec![DurableClaimStatus::Validated],
            ClaimStatus::Contradicted => {
                vec![DurableClaimStatus::Rejected, DurableClaimStatus::Disputed]
            }
            ClaimStatus::Superseded => {
                vec![DurableClaimStatus::Superseded, DurableClaimStatus::Stale]
            }
        };

        let mut hits = Vec::new();
        for durable_status in durable_statuses {
            let rows = self
                .get_by_status(durable_status, limit)
                .map_err(|e| MemoryProviderError::Durable(e.to_string()))?;
            hits.extend(rows.into_iter().map(|row| MemoryHit {
                claim_id: row.claim_id,
                subject: row.claim_text,
                predicate: "matches".to_string(),
                object: None,
                status: durable_to_memory(row.status),
                confidence: row.confidence as f64,
                evidence_ids: Vec::new(),
            }));
        }

        hits.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}
