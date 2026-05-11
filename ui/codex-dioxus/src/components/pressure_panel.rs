use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[component]
pub fn PressurePanel(proof: CodexProofState) -> Element {
    let pressure = proof.replay.final_state.last_pressure_uncertainty;
    let regulation = proof.replay.final_state.last_pressure_coherence;

    rsx! {
        section { class: "card",
            h3 { "Operational Pressure" }
            div { class: "metric-grid",
                div { class: "metric",
                    span { class: "metric-label", "Pressure" }
                    span { class: "metric-value", "{pressure:.3}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Regulation" }
                    span { class: "metric-value", "{regulation:.3}" }
                }
            }
            p { class: "muted", "Pressure modulation is runtime-authored and read-only in the UI shell." }
        }
    }
}
