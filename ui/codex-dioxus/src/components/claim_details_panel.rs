use crate::bridge::state_provider::use_ui_runtime_state;
use crate::bridge::types::RuntimeStepResult;
use dioxus::prelude::*;

/// Enhanced claim details panel showing detailed information about each claim
/// backing an answer, sourced from answer_basis_items or live state
#[component]
pub fn ClaimDetailsPanel(#[props(default)] trace: Option<RuntimeStepResult>) -> Element {
    // Try to get state from context for live data
    let state = use_ui_runtime_state();
    let live_claims = state.read().claims.read().clone();
    let Some(trace) = trace else {
        // If no trace provided, show live claims from state instead
        if live_claims.is_empty() {
            return rsx! {
                section { class: "card",
                    h3 { "Claim Details" }
                    p { class: "muted", "No claims available. Awaiting runtime data..." }
                }
            };
        }

        // Show live claims from state
        return rsx! {
            section { class: "card",
                h3 { "Claim Details (Live)" }
                p { class: "muted small", "{live_claims.len()} claims in runtime state" }

                div { class: "claims-list",
                    for (idx, claim) in live_claims.iter().enumerate() {
                        div { class: "claim-card",
                            div { class: "claim-header",
                                span { class: "claim-index", "{idx + 1}." }
                                code { class: "claim-id", "{claim.claim_id}" }
                                span { class: "claim-confidence",
                                    span {
                                        class: "confidence-badge claim-badge {confidence_class(claim.confidence_pct)}",
                                        "{claim.confidence_pct}%"
                                    }
                                }
                            }
                            div { class: "claim-content",
                                div { class: "claim-triple",
                                    div { class: "claim-part",
                                        span { class: "claim-label", "Subject" }
                                        span { class: "claim-value subject", "{claim.subject}" }
                                    }
                                    div { class: "claim-part",
                                        span { class: "claim-label", "Predicate" }
                                        code { class: "claim-value predicate", "{claim.predicate}" }
                                    }
                                    div { class: "claim-part",
                                        span { class: "claim-label", "Object" }
                                        span { class: "claim-value object",
                                            if let Some(obj) = &claim.object {
                                                "{obj}"
                                            } else {
                                                span { class: "muted", "(none)" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    };

    // Only show if we have basis items (grounded answer)
    if trace.answer_basis_items.is_empty() {
        return rsx! {
            section { class: "card",
                h3 { "Claim Details" }
                p { class: "muted", "No basis items for this response." }
                if !trace.claim_ids.is_empty() {
                    p { class: "muted small",
                        "Retrieved claims: {trace.claim_ids.join(\", \")}"
                    }
                }
            }
        };
    }

    let total_claims = trace.claim_ids.len();
    let grounded_claims = trace.answer_basis_items.len();
    let contradicted = trace.contradiction_ids.len();

    rsx! {
        section { class: "card",
            h3 { "Claim Details" }

            div { class: "claim-summary-row",
                span { class: "claim-summary-item",
                    span { class: "summary-label", "Grounded Claims" }
                    span { class: "summary-value", "{grounded_claims}" }
                }
                span { class: "claim-summary-item",
                    span { class: "summary-label", "Total Retrieved" }
                    span { class: "summary-value", "{total_claims}" }
                }
                if contradicted > 0 {
                    span { class: "claim-summary-item",
                        span { class: "summary-label", "Contradicted" }
                        span { class: "summary-value bad", "{contradicted}" }
                    }
                }
            }

            div { class: "claims-list",
                for (idx, item) in trace.answer_basis_items.iter().enumerate() {
                    div { class: "claim-card",
                        div { class: "claim-header",
                            span { class: "claim-index", "{idx + 1}." }
                            code { class: "claim-id", "{item.claim_id}" }
                            span { class: "claim-confidence",
                                span {
                                    class: "confidence-badge claim-badge {confidence_class(item.confidence_pct)}",
                                    "{item.confidence_pct}%"
                                }
                            }
                        }

                        div { class: "claim-content",
                            div { class: "claim-triple",
                                div { class: "claim-part",
                                    span { class: "claim-label", "Subject" }
                                    span { class: "claim-value subject", "{item.subject}" }
                                }
                                div { class: "claim-part",
                                    span { class: "claim-label", "Predicate" }
                                    code { class: "claim-value predicate", "{item.predicate}" }
                                }
                                div { class: "claim-part",
                                    span { class: "claim-label", "Object" }
                                    span { class: "claim-value object",
                                        if let Some(obj) = &item.object {
                                            "{obj}"
                                        } else {
                                            span { class: "muted", "(none)" }
                                        }
                                    }
                                }
                            }
                        }

                        if !item.evidence_ids.is_empty() {
                            div { class: "claim-evidence",
                                span { class: "evidence-label", "Backing Evidence ({item.evidence_ids.len()})" }
                                div { class: "evidence-list",
                                    for ev_id in &item.evidence_ids {
                                        span { class: "evidence-ref", "{ev_id}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !trace.contradiction_ids.is_empty() {
                div { class: "contradicted-claims",
                    h4 { "⚠️ Contradicted Claims" }
                    div { class: "contradiction-list",
                        for contra_id in &trace.contradiction_ids {
                            span { class: "contradiction-badge", "{contra_id}" }
                        }
                    }
                    p { class: "muted small",
                        "These claims are superseded or contradicted and are not included in the answer."
                    }
                }
            }
        }
    }
}

fn confidence_class(conf: u8) -> &'static str {
    match conf {
        90..=100 => "conf-high",
        70..=89 => "conf-good",
        50..=69 => "conf-moderate",
        _ => "conf-low",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::types::MetadataQuality;

    #[test]
    fn claim_details_empty_when_no_basis_items() {
        let trace = RuntimeStepResult {
            selected_action: "answer".to_string(),
            response_text: "Some answer".to_string(),
            answer_basis_items: vec![],
            ..Default::default()
        };

        let mut vdom = VirtualDom::new_with_props(
            ClaimDetailsPanel,
            ClaimDetailsPanelProps { trace: Some(trace) },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("No basis items"));
    }

    #[test]
    fn claim_details_shows_basis_items() {
        let items = vec![BasisItemSummary {
            claim_id: "cl-001".to_string(),
            subject: "Entity A".to_string(),
            predicate: "has_property".to_string(),
            object: Some("value_x".to_string()),
            confidence_pct: 85,
            evidence_ids: vec!["ev-001".to_string(), "ev-002".to_string()],
        }];

        let trace = RuntimeStepResult {
            selected_action: "answer".to_string(),
            response_text: "Some answer".to_string(),
            answer_basis_items: items,
            claim_ids: vec!["cl-001".to_string()],
            metadata_quality: MetadataQuality::RuntimeGrounded,
            ..Default::default()
        };

        let mut vdom = VirtualDom::new_with_props(
            ClaimDetailsPanel,
            ClaimDetailsPanelProps { trace: Some(trace) },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("Entity A"));
        assert!(html.contains("has_property"));
        assert!(html.contains("value_x"));
        assert!(html.contains("85%"));
    }
}
