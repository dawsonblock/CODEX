use dioxus::prelude::*;

use crate::bridge::types::{CodexProofState, HistoricalSummary, TimeRange};

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn proof_dashboard_snapshot(
    state: &CodexProofState,
    history: &HistoricalSummary,
    range: TimeRange,
) -> String {
    let sim = &state.simworld.scorecard;
    format!(
        "range={}\ncycles={}\nresource_survival={:.3}\nunsafe={}\nmean_total={:.3}\naction_match={:.3}\nreplay_passes={}\nidempotent={}\nevidence_valid={}\nlong_horizon_safety={}\nhistory_total={}\nhistory_async={}\nhistory_test={}",
        range.label(),
        sim.cycles,
        sim.resource_survival,
        sim.unsafe_action_count,
        sim.mean_total_score,
        sim.action_match_rate,
        state.replay.replay_passes,
        state.replay.is_idempotent,
        state.evidence.all_valid,
        state.long_horizon.safety_violations,
        history.total_traces,
        history.async_traces,
        history.test_traces
    )
}

#[component]
pub fn ProofDashboard(
    state: Option<CodexProofState>,
    history: HistoricalSummary,
    range: TimeRange,
) -> Element {
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

                div { class: "k", "history range" }
                div { "{range.label()}" }

                div { class: "k", "history traces" }
                div { "{history.total_traces}" }

                div { class: "k", "history async traces" }
                div { "{history.async_traces}" }

                div { class: "k", "history test traces" }
                div { "{history.test_traces}" }
            }

            if let Some(epoch) = history.latest_epoch {
                p { class: "muted", "Latest trace epoch: {epoch}" }
            }

            if !history.latest_files.is_empty() {
                p { class: "muted", "Recent files:" }
                ul { class: "list",
                    for name in history.latest_files {
                        li { "{name}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::types::{
        EvidenceIntegrityReport, LongHorizonReport, ReplayFinalState, ReplayReport,
        ScorecardMetrics, SimworldSummary,
    };

    #[test]
    fn proof_dashboard_snapshot_matches_expected() {
        let state = CodexProofState {
            simworld: SimworldSummary {
                scorecard: ScorecardMetrics {
                    cycles: 25,
                    resource_survival: 1.0,
                    unsafe_action_count: 0,
                    mean_total_score: 0.827,
                    action_match_rate: 1.0,
                },
            },
            replay: ReplayReport {
                replay_passes: true,
                is_idempotent: true,
                final_state: ReplayFinalState::default(),
                event_count: 10,
            },
            evidence: EvidenceIntegrityReport {
                all_valid: true,
                ..EvidenceIntegrityReport::default()
            },
            long_horizon: LongHorizonReport {
                safety_violations: 0,
                ..LongHorizonReport::default()
            },
            ..CodexProofState::default()
        };
        let history = HistoricalSummary {
            range: TimeRange::Last24Hours,
            total_traces: 5,
            async_traces: 2,
            test_traces: 1,
            ..HistoricalSummary::default()
        };

        let snapshot = proof_dashboard_snapshot(&state, &history, TimeRange::Last24Hours);
        let expected = "range=Last 24h\ncycles=25\nresource_survival=1.000\nunsafe=0\nmean_total=0.827\naction_match=1.000\nreplay_passes=true\nidempotent=true\nevidence_valid=true\nlong_horizon_safety=0\nhistory_total=5\nhistory_async=2\nhistory_test=1";
        assert_eq!(snapshot, expected);
    }
}
