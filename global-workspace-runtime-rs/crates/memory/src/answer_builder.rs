//! AnswerBuilder: compose user-facing answer envelopes from lifecycle-aware claims.

use crate::{ClaimStatus, MemoryClaim};
use serde::{Deserialize, Serialize};

/// Stable response contract for downstream UI/reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnswerEnvelope {
    pub text: String,
    pub basis: String,
    pub evidence_ids: Vec<String>,
    pub action_type: String,
    pub confidence: f64,
    pub warnings: Vec<String>,
    pub missing_evidence_reason: Option<String>,
    pub cited_claim_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AnswerBuildContext {
    pub action_type: String,
    pub evidence_ids: Vec<String>,
}

/// Deterministic builder for claim-grounded answer envelopes.
#[derive(Debug, Default)]
pub struct AnswerBuilder;

impl AnswerBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build an answer using claim lifecycle policy:
    /// - include Active claims in output and citations
    /// - surface Contradicted claims as warnings only
    /// - exclude Superseded and Unverified claims from answer body
    pub fn build(&self, query: &str, claims: &[MemoryClaim]) -> AnswerEnvelope {
        self.build_with_context(query, claims, AnswerBuildContext::default())
    }

    pub fn build_with_context(
        &self,
        query: &str,
        claims: &[MemoryClaim],
        mut ctx: AnswerBuildContext,
    ) -> AnswerEnvelope {
        let active_claims: Vec<&MemoryClaim> = claims
            .iter()
            .filter(|c| c.status == ClaimStatus::Active)
            .collect();
        let contradicted_claims: Vec<&MemoryClaim> = claims
            .iter()
            .filter(|c| c.status == ClaimStatus::Contradicted)
            .collect();

        let cited_claim_ids = active_claims
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();

        let confidence = if active_claims.is_empty() {
            0.0
        } else {
            (active_claims.iter().map(|c| c.confidence).sum::<f64>() / active_claims.len() as f64)
                .clamp(0.0, 1.0)
        };

        let mut warnings = Vec::new();
        if !contradicted_claims.is_empty() {
            warnings.push(format!(
                "disputed_claims_present:{}",
                contradicted_claims
                    .iter()
                    .map(|c| c.id.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        if ctx.action_type.is_empty() {
            ctx.action_type = if active_claims.is_empty() {
                "defer_insufficient_evidence".to_string()
            } else {
                "answer".to_string()
            };
        }

        let basis = if active_claims.is_empty() {
            "insufficient_grounded_claims".to_string()
        } else {
            "grounded_active_claims".to_string()
        };

        let missing_evidence_reason = if active_claims.is_empty() {
            Some("no_active_claims".to_string())
        } else if active_claims.iter().any(|c| c.evidence_ids.is_empty()) {
            warnings.push("active_claim_missing_evidence_link".to_string());
            Some("active_claim_without_evidence_link".to_string())
        } else {
            None
        };

        let text = if active_claims.is_empty() {
            format!(
                "Insufficient grounded active claims to answer query: {}",
                query
            )
        } else {
            let snippets = active_claims
                .iter()
                .map(|c| {
                    if let Some(obj) = &c.object {
                        format!("{} {} {}", c.subject, c.predicate, obj)
                    } else {
                        format!("{} {}", c.subject, c.predicate)
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            snippets
        };

        AnswerEnvelope {
            text,
            basis,
            evidence_ids: ctx.evidence_ids,
            action_type: ctx.action_type,
            confidence,
            warnings,
            missing_evidence_reason,
            cited_claim_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(id: &str, status: ClaimStatus, confidence: f64) -> MemoryClaim {
        MemoryClaim {
            id: id.to_string(),
            subject: "sky".to_string(),
            predicate: "is".to_string(),
            object: Some("blue".to_string()),
            status,
            confidence,
            evidence_ids: vec!["e1".to_string()],
            evidence_hashes: vec!["h1".to_string()],
            source_label: "test".to_string(),
            evidence_links: vec![],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
            audit_trail: vec![],
            superseded_by: None,
        }
    }

    #[test]
    fn includes_active_claim_citations() {
        let b = AnswerBuilder::new();
        let out = b.build(
            "what color is the sky",
            &[claim("c1", ClaimStatus::Active, 0.8)],
        );
        assert_eq!(out.cited_claim_ids, vec!["c1".to_string()]);
        assert!(out.text.contains("sky is blue"));
        assert_eq!(out.action_type, "answer");
        assert_eq!(out.basis, "grounded_active_claims");
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn contradicted_claims_emit_warning_only() {
        let b = AnswerBuilder::new();
        let out = b.build(
            "sky status",
            &[
                claim("c1", ClaimStatus::Active, 0.7),
                claim("c2", ClaimStatus::Contradicted, 0.9),
            ],
        );
        assert_eq!(out.cited_claim_ids, vec!["c1".to_string()]);
        assert_eq!(out.warnings.len(), 1);
        assert!(out.warnings[0].contains("c2"));
    }

    #[test]
    fn superseded_and_unverified_excluded_from_answer_body() {
        let b = AnswerBuilder::new();
        let out = b.build(
            "status",
            &[
                claim("c_old", ClaimStatus::Superseded, 0.95),
                claim("c_new", ClaimStatus::Unverified, 0.95),
            ],
        );
        assert!(out.cited_claim_ids.is_empty());
        assert!(out.text.contains("Insufficient grounded active claims"));
        assert_eq!(out.confidence, 0.0);
        assert_eq!(out.action_type, "defer_insufficient_evidence");
        assert_eq!(
            out.missing_evidence_reason.as_deref(),
            Some("no_active_claims")
        );
    }

    #[test]
    fn confidence_averages_only_active_claims() {
        let b = AnswerBuilder::new();
        let out = b.build(
            "status",
            &[
                claim("c1", ClaimStatus::Active, 0.2),
                claim("c2", ClaimStatus::Active, 0.8),
                claim("c3", ClaimStatus::Contradicted, 1.0),
            ],
        );
        assert!((out.confidence - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn build_with_context_sets_action_and_evidence() {
        let b = AnswerBuilder::new();
        let out = b.build_with_context(
            "status",
            &[claim("c1", ClaimStatus::Active, 0.5)],
            AnswerBuildContext {
                action_type: "summarize".to_string(),
                evidence_ids: vec!["e1".to_string(), "e2".to_string()],
            },
        );

        assert_eq!(out.action_type, "summarize");
        assert_eq!(out.evidence_ids, vec!["e1".to_string(), "e2".to_string()]);
    }
}
