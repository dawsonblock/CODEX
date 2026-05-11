use crate::bridge::types::CodexProofState;
use dioxus::prelude::*;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evidence_snapshot(proof: &CodexProofState) -> String {
    let replay_events = proof.replay.event_count;
    let contradictions = proof.replay.final_state.contradictions_detected;
    let claims = proof.replay.final_state.claims_asserted;
    let evidence_items = proof.evidence.total_entries;
    let contradiction_rate = if replay_events > 0 {
        contradictions as f64 / replay_events as f64
    } else {
        0.0
    };

    format!(
        "evidence_items={}\nclaims={}\ncontradictions={}\ncontradiction_rate={:.3}",
        evidence_items, claims, contradictions, contradiction_rate
    )
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::types::{EvidenceIntegrityReport, ReplayFinalState, ReplayReport};

    #[test]
    fn evidence_snapshot_matches_expected() {
        let proof = CodexProofState {
            evidence: EvidenceIntegrityReport {
                total_entries: 10,
                ..EvidenceIntegrityReport::default()
            },
            replay: ReplayReport {
                event_count: 20,
                final_state: ReplayFinalState {
                    claims_asserted: 6,
                    contradictions_detected: 2,
                    ..ReplayFinalState::default()
                },
                ..ReplayReport::default()
            },
            ..CodexProofState::default()
        };
        let snapshot = evidence_snapshot(&proof);
        let expected = "evidence_items=10\nclaims=6\ncontradictions=2\ncontradiction_rate=0.100";
        assert_eq!(snapshot, expected);
    }
}
