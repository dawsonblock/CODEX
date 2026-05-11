use dioxus::prelude::*;

use crate::bridge::types::CodexProofState;

#[component]
pub fn ProofDashboard(state: Option<CodexProofState>) -> Element {
    let s = state.unwrap_or_default();
    let sim = s.simworld.scorecard;
    let replay = s.replay;
    let evidence = s.evidence;
    let long = s.long_horizon;

    let curated = s
        .nl_benchmark
        .curated
        .map(|v| format!("{:.3}", v.scorecard.action_match_rate))
        .unwrap_or_else(|| "n/a".to_string());
    let held = s
        .nl_benchmark
        .held_out
        .map(|v| format!("{:.3}", v.scorecard.action_match_rate))
        .unwrap_or_else(|| "n/a".to_string());
    let adversarial = s
        .nl_benchmark
        .adversarial
        .map(|v| format!("{:.3}", v.scorecard.action_match_rate))
        .unwrap_or_else(|| "n/a".to_string());

    rsx! {
        section { class: "card",
            h3 { "Proof Dashboard" }
            div { class: "kv",
                div { class: "k", "simworld cycles" }
                div { "{sim.cycles}" }

                div { class: "k", "resource_survival" }
                div { "{sim.resource_survival:.3}" }

                div { class: "k", "unsafe_action_count" }
                div { "{sim.unsafe_action_count}" }

                div { class: "k", "mean_total_score" }
                div { "{sim.mean_total_score:.3}" }

                div { class: "k", "action_match_rate" }
                div { "{sim.action_match_rate:.3}" }

                div { class: "k", "replay_passes" }
                div { "{replay.replay_passes}" }

                div { class: "k", "is_idempotent" }
                div { "{replay.is_idempotent}" }

                div { class: "k", "evidence all_valid" }
                div { "{evidence.all_valid}" }

                div { class: "k", "long-horizon safety_violations" }
                div { "{long.safety_violations}" }

                div { class: "k", "nl benchmark (curated/held/adversarial)" }
                div { "{curated} / {held} / {adversarial}" }
            }
        }
    }
}
