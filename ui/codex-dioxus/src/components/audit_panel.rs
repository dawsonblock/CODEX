use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[component]
pub fn AuditPanel(proof: CodexProofState) -> Element {
    let audit_events = proof.replay.final_state.reasoning_audits;
    let replay_events = proof.replay.event_count;

    rsx! {
        section { class: "card",
            h3 { "Audit and Boundaries" }
            ul { class: "list",
                li { "Audit events: {audit_events}" }
                li { "Replay events: {replay_events}" }
                li { "Authority: Codex runtime in Rust remains source of truth." }
                li { "UI role: visualization and bounded command intent only." }
                li { "No sentience, no AGI, no production readiness claims." }
            }
        }
    }
}
