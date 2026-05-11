use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[component]
pub fn EvidencePanel(proof: CodexProofState) -> Element {
    let replay_events = proof.replay.event_count;
    let contradictions = proof.replay.final_state.contradictions_detected;
    let claims = proof.replay.final_state.claims_asserted;
    let evidence_items = proof.evidence.total_entries;

    let contradiction_rate = if replay_events > 0 {
        contradictions as f64 / replay_events as f64
    } else {
        0.0
    };

    rsx! {
        section { class: "card",
            h3 { "Evidence and Contradiction" }
            div { class: "metric-grid",
                div { class: "metric",
                    span { class: "metric-label", "Evidence Items" }
                    span { class: "metric-value", "{evidence_items}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Claims" }
                    span { class: "metric-value", "{claims}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Contradictions" }
                    span { class: "metric-value", "{contradictions}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Contradiction Rate" }
                    span { class: "metric-value", "{contradiction_rate:.3}" }
                }
            }
        }
    }
}
