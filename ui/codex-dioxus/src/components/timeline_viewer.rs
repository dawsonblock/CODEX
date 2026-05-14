use dioxus::prelude::*;

/// Timeline visualization showing claim creation sequence and evidence linking
#[component]
pub fn TimelineViewer(claims_count: usize, evidence_count: usize, cycle_count: usize) -> Element {
    // Mock timeline data for 15 cycles
    let timeline_events = vec![
        ("Cycle 1", "● Claim cl-001 created", "event-claim"),
        ("Cycle 2", "● Evidence ev-001 linked", "event-evidence"),
        ("Cycle 2", "● Evidence ev-002 linked", "event-evidence"),
        ("Cycle 3", "● Query executed", "event-query"),
        ("Cycle 4", "● Answer generated", "event-answer"),
        ("Cycle 5", "⚡ Pressure spike (0.85)", "event-pressure"),
        ("Cycle 6", "● Claim cl-002 created", "event-claim"),
        ("Cycle 7", "● Evidence ev-003 linked", "event-evidence"),
        (
            "Cycle 8",
            "⚠️ Contradiction detected",
            "event-contradiction",
        ),
        ("Cycle 9", "● Answer generated", "event-answer"),
        ("Cycle 10", "● Evidence ev-004 linked", "event-evidence"),
        ("Cycle 11", "● Query executed", "event-query"),
        ("Cycle 12", "● Answer generated", "event-answer"),
        ("Cycle 13", "⚡ Pressure modulation", "event-pressure"),
        ("Cycle 14", "● Final answer ready", "event-answer"),
        ("Cycle 15", "✓ Reasoning complete", "event-complete"),
    ];

    rsx! {
        section { class: "card",
            h3 { "Timeline Visualization" }
            p { class: "muted small",
                "Claim creation, evidence linking, and event sequence across {cycle_count} cycles"
            }

            div { class: "timeline-container",
                div { class: "timeline-header",
                    span { class: "timeline-label", "Cycle" }
                    span { class: "timeline-label", "Event" }
                }

                div { class: "timeline-track",
                    for (idx, (cycle, event, event_type)) in timeline_events.iter().enumerate() {
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
