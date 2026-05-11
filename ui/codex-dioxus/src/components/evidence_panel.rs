use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[component]
pub fn EvidencePanel(proof: CodexProofState) -> Element {
    let contradiction_rate = if proof.replays > 0 {
        proof.contradictions as f64 / proof.replays as f64
    } else {
        0.0
    };

    rsx! {
        section { class: "card",
            h3 { "Evidence and Contradiction" }
            div { class: "metric-grid",
                div { class: "metric",
                    span { class: "metric-label", "Evidence Items" }
                    span { class: "metric-value", "{proof.evidence_items}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Claims" }
                    span { class: "metric-value", "{proof.claims}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Contradictions" }
                    span { class: "metric-value", "{proof.contradictions}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Contradiction Rate" }
                    span { class: "metric-value", "{contradiction_rate:.3}" }
                }
            }
        }
    }
}
