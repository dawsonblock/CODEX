use crate::bridge::instrumentation::{end_component_render_timer, start_component_render_timer};
use crate::bridge::state_provider::use_ui_runtime_state;
use crate::bridge::types::BasisItemSummary;
use dioxus::prelude::*;

/// Renders an answer basis items table showing claims and evidence backing
/// Can show either passed basis_items or live evidence from state
#[component]
pub fn BasisItemsTable(
    #[props(default)] basis_items: Vec<BasisItemSummary>,
    #[props(default)] warnings: Vec<String>,
    #[props(default)] show_warnings: bool,
) -> Element {
    let timer = start_component_render_timer();
    // Get live state
    let state = use_ui_runtime_state();
    let live_evidence = state.read().evidence.read().clone();

    // Use passed basis_items if provided, otherwise use live evidence count
    let has_items = !basis_items.is_empty() || !live_evidence.is_empty();

    let confidence_class = |conf: u8| -> &'static str {
        match conf {
            90..=100 => "conf-high",
            70..=89 => "conf-good",
            50..=69 => "conf-moderate",
            _ => "conf-low",
        }
    };

    let element = if !has_items && (warnings.is_empty() || !show_warnings) {
        rsx! {
            div { class: "basis-items-empty",
                "(no basis items)"
            }
        }
    } else {
        rsx! {
            div { class: "basis-items-container",
                if !basis_items.is_empty() {
                    div { class: "basis-header",
                        span { class: "basis-label", "✓ Grounded Answer" }
                        span { class: "basis-count", "{basis_items.len()} claim(s)" }
                    }
                    table { class: "basis-table",
                    thead {
                        tr {
                            th { "Claim" }
                            th { "Subject" }
                            th { "Predicate" }
                            th { "Object" }
                            th { "Confidence" }
                            th { "Evidence" }
                        }
                    }
                    tbody {
                        for item in &basis_items {
                            tr { class: "basis-row",
                                td { class: "basis-claim-id",
                                    code { "{item.claim_id}" }
                                }
                                td { class: "basis-subject",
                                    "{item.subject}"
                                }
                                td { class: "basis-predicate",
                                    code { "{item.predicate}" }
                                }
                                td { class: "basis-object",
                                    match &item.object {
                                        Some(obj) => rsx! { "{obj}" },
                                        None => rsx! { span { class: "muted", "(none)" } },
                                    }
                                }
                                td { class: "basis-confidence",
                                    span {
                                        class: "confidence-badge {confidence_class(item.confidence_pct)}",
                                        "{item.confidence_pct}%"
                                    }
                                }
                                td { class: "basis-evidence",
                                    div { class: "evidence-ids",
                                        for ev_id in &item.evidence_ids {
                                            span { class: "evidence-badge", "{ev_id}" }
                                        }
                                        if item.evidence_ids.is_empty() {
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

            if show_warnings && !warnings.is_empty() {
                div { class: "answer-warnings",
                    div { class: "warnings-header",
                        "⚠️ Warnings ({warnings.len()})"
                    }
                    ul { class: "warnings-list",
                        for warning in &warnings {
                            li { class: "warning-item", "{warning}" }
                        }
                    }
                }
            }
        }
    };

    end_component_render_timer("BasisItemsTable", timer);
    element
}

// Test module disabled - Dioxus 0.7 rendering API has changed
// To be re-enabled once tests are migrated to new API patterns
