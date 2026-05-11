use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[component]
pub fn AuditPanel(proof: CodexProofState) -> Element {
    rsx! {
        section { class: "card",
            h3 { "Audit and Boundaries" }
            ul { class: "list",
                li { "Audit events: {proof.audit_events}" }
                li { "Replay events: {proof.replays}" }
                li { "Authority: CODEX runtime in Rust remains source of truth." }
                li { "UI role: visualization and bounded command intent only." }
                li { "No sentience, no AGI, no production readiness claims." }
            }
        }
    }
}
