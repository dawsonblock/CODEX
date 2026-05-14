use crate::bridge::types::BasisItemSummary;
use dioxus::prelude::*;

/// Renders an answer basis items table showing claims and evidence backing
#[component]
pub fn BasisItemsTable(
    basis_items: Vec<BasisItemSummary>,
    warnings: Vec<String>,
    #[props(default)] show_warnings: bool,
) -> Element {
    if basis_items.is_empty() && (warnings.is_empty() || !show_warnings) {
        return rsx! {
            div { class: "basis-items-empty",
                "(no basis items)"
            }
        };
    }

    let confidence_class = |conf: u8| -> &'static str {
        match conf {
            90..=100 => "conf-high",
            70..=89 => "conf-good",
            50..=69 => "conf-moderate",
            _ => "conf-low",
        }
    };

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_items_table_empty() {
        let mut vdom = VirtualDom::new_with_props(
            BasisItemsTable,
            BasisItemsTableProps {
                basis_items: vec![],
                warnings: vec![],
                show_warnings: false,
            },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("basis-items-empty"));
    }

    #[test]
    fn basis_items_shows_confidence_class() {
        let items = vec![
            BasisItemSummary {
                claim_id: "c1".to_string(),
                subject: "Test".to_string(),
                predicate: "is_valid".to_string(),
                object: None,
                confidence_pct: 95,
                evidence_ids: vec!["e1".to_string()],
            },
            BasisItemSummary {
                claim_id: "c2".to_string(),
                subject: "Other".to_string(),
                predicate: "exists".to_string(),
                object: Some("here".to_string()),
                confidence_pct: 45,
                evidence_ids: vec![],
            },
        ];

        let mut vdom = VirtualDom::new_with_props(
            BasisItemsTable,
            BasisItemsTableProps {
                basis_items: items,
                warnings: vec![],
                show_warnings: false,
            },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("conf-high"));
        assert!(html.contains("conf-low"));
    }

    #[test]
    fn basis_items_shows_warnings() {
        let items = vec![BasisItemSummary {
            claim_id: "c1".to_string(),
            subject: "Test".to_string(),
            predicate: "is_valid".to_string(),
            object: None,
            confidence_pct: 75,
            evidence_ids: vec!["e1".to_string()],
        }];

        let warnings = vec!["Claim cl-042: contradicted by newer evidence".to_string()];

        let mut vdom = VirtualDom::new_with_props(
            BasisItemsTable,
            BasisItemsTableProps {
                basis_items: items,
                warnings: warnings.clone(),
                show_warnings: true,
            },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("answer-warnings"));
        assert!(html.contains("contradicted"));
    }
}
