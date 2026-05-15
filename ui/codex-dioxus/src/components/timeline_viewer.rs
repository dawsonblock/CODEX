use crate::bridge::state_provider::use_ui_runtime_state;
use dioxus::prelude::*;

/// Timeline visualization showing claim creation sequence and evidence linking
/// Now integrated with UIRuntimeState for live updates
#[component]
pub fn TimelineViewer(
    #[props(default)] claims_count: Option<usize>,
    #[props(default)] evidence_count: Option<usize>,
    #[props(default)] cycle_count: Option<usize>,
) -> Element {
    // Get state from context
    let state = use_ui_runtime_state();

    // Use live data from state, with fallbacks for backward compatibility
    let timeline_data = state.read().timeline_events.read().clone();
    let cycle_current = *state.read().current_cycle.read() as usize;
    let claims_total = claims_count.unwrap_or_else(|| state.read().claims.read().len());
    let evidence_total = evidence_count.unwrap_or_else(|| state.read().evidence.read().len());
    let cycles_total = cycle_count.unwrap_or(cycle_current);

    // Convert internal TimelineEvent structs to displayable format
    let display_events: Vec<(String, String, String)> = timeline_data
        .iter()
        .map(|evt| {
            let event_type = match evt.event_type.as_str() {
                "claim" => "event-claim",
                "evidence" => "event-evidence",
                "answer" => "event-answer",
                "pressure" => "event-pressure",
                "contradiction" => "event-contradiction",
                "query" => "event-query",
                "complete" => "event-complete",
                _ => "event-generic",
            };
            (
                format!("Cycle {}", evt.cycle),
                evt.message.clone(),
                event_type.to_string(),
            )
        })
        .collect();

    rsx! {
        section { class: "card",
            h3 { "Timeline Visualization" }
            p { class: "muted small",
                "Claim creation, evidence linking, and event sequence across {cycles_total} cycles (Live: {claims_total} claims, {evidence_total} evidence)"
            }

            div { class: "timeline-container",
                div { class: "timeline-header",
                    span { class: "timeline-label", "Cycle" }
                    span { class: "timeline-label", "Event" }
                }

                div { class: "timeline-track",
                    if display_events.is_empty() {
                        div { class: "timeline-placeholder",
                            "No events yet. Awaiting runtime events..."
                        }
                    } else {
                        for (idx, (cycle, event, event_type)) in display_events.iter().enumerate() {
                            div { class: "timeline-event {event_type}",
                                div { class: "timeline-marker" }
                                div { class: "timeline-content",
                                    span { class: "timeline-cycle", "{cycle}" }
                                    span { class: "timeline-text", "{event}" }
                                }
                                if (idx + 1) % 3 == 0 {
                                    div { class: "timeline-checkpoint" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "timeline-legend",
                span { class: "legend-item event-claim", "● Claim" }
                span { class: "legend-item event-evidence", "● Evidence" }
                span { class: "legend-item event-answer", "● Answer" }
                span { class: "legend-item event-pressure", "⚡ Pressure" }
                span { class: "legend-item event-contradiction", "⚠️ Contradiction" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_viewer_renders() {
        let mut vdom = VirtualDom::new_with_props(
            TimelineViewer,
            TimelineViewerProps {
                claims_count: 17,
                evidence_count: 96,
                cycle_count: 15,
            },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("timeline-container"));
        assert!(html.contains("Cycle"));
    }
}
