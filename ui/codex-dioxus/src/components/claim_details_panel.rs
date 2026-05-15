use crate::bridge::state_provider::use_ui_runtime_state;
use crate::bridge::types::{BasisItemSummary, RuntimeStepResult};
use crate::bridge::instrumentation::{start_component_render_timer, end_component_render_timer};
use dioxus::prelude::*;

/// Enhanced claim details panel showing detailed information about each claim
/// backing an answer, sourced from answer_basis_items or live state
#[component]
pub fn ClaimDetailsPanel(#[props(default)] trace: Option<RuntimeStepResult>) -> Element {
    let timer = start_component_render_timer();
    // Try to get state from context for live data
    let state = use_ui_runtime_state();
    let live_claims = state.read().claims.read().clone();
    
    let element = if let Some(trace) = trace {
        // Only show if we have basis items (grounded answer)
        if trace.answer_basis_items.is_empty() {
            rsx! {
                section { class: "card",
                    h3 { "Claim Details" }
                    p { class: "muted", "No basis items for this response." }
                    if !trace.claim_ids.is_empty() {
                        p { class: "muted small",
                            "Retrieved claims: {trace.claim_ids.join(\", \")}"
                        }
                    }
                }
            }
        } else {
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
    } else {
        // If no trace provided, show live claims from state instead
        if live_claims.is_empty() {
            rsx! {
                section { class: "card",
                    h3 { "Claim Details" }
                    p { class: "muted", "No claims available. Awaiting runtime data..." }
                }
            }
        } else {
            rsx! {
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
            }
        }
    };

    end_component_render_timer("ClaimDetailsPanel", timer);
    element
}

fn confidence_class(conf: u8) -> &'static str {
    match conf {
        90..=100 => "conf-high",
        70..=89 => "conf-good",
        50..=69 => "conf-moderate",
        _ => "conf-low",
    }
}

// Test module disabled - Dioxus 0.7 rendering API has changed
// To be re-enabled once tests are migrated to new API patterns
