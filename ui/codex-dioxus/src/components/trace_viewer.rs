use dioxus::prelude::*;

/// Long-horizon trace viewer for detailed per-cycle analysis
#[component]
pub fn LongHorizonTraceViewer(current_cycle: usize) -> Element {
    let mut selected_cycle = use_signal(|| current_cycle.max(1).min(15));

    // Mock trace data for different cycles
    let trace_data = match *selected_cycle.read() {
        1 => ("Cycle 1", vec!["cl-001"], vec!["ev-001"], "answer", 0.75),
        2 => ("Cycle 2", vec!["cl-001", "cl-002"], vec!["ev-001", "ev-002", "ev-003"], "answer", 0.82),
        3 => ("Cycle 3", vec!["cl-001"], vec!["ev-001"], "retrieve_memory", 0.68),
        4 => ("Cycle 4", vec!["cl-001", "cl-002"], vec!["ev-001", "ev-002"], "answer", 0.85),
        5 => ("Cycle 5", vec!["cl-001", "cl-002"], vec!["ev-001", "ev-002", "ev-003"], "defer_insufficient_evidence", 0.45),
        6 => ("Cycle 6", vec!["cl-001", "cl-002", "cl-003"], vec!["ev-001", "ev-002", "ev-003", "ev-004"], "answer", 0.88),
        7 => ("Cycle 7", vec!["cl-001", "cl-003"], vec!["ev-001", "ev-003", "ev-004"], "answer", 0.79),
        8 => ("Cycle 8", vec!["cl-001"], vec!["ev-001"], "ask_clarification", 0.65),
        9 => ("Cycle 9", vec!["cl-001", "cl-002", "cl-003"], vec!["ev-001", "ev-002", "ev-003", "ev-004"], "answer", 0.86),
        10 => ("Cycle 10", vec!["cl-001", "cl-002"], vec!["ev-001", "ev-002"], "answer", 0.81),
        11 => ("Cycle 11", vec!["cl-001", "cl-002", "cl-003"], vec!["ev-001", "ev-002", "ev-003"], "plan", 0.77),
        12 => ("Cycle 12", vec!["cl-002", "cl-003"], vec!["ev-002", "ev-003", "ev-004"], "answer", 0.84),
        13 => ("Cycle 13", vec!["cl-001", "cl-002", "cl-003"], vec!["ev-001", "ev-002", "ev-003", "ev-004"], "answer", 0.89),
        14 => ("Cycle 14", vec!["cl-001", "cl-002"], vec!["ev-001", "ev-002"], "answer", 0.80),
        _ => ("Cycle 15", vec!["cl-001", "cl-002", "cl-003"], vec!["ev-001", "ev-002", "ev-003", "ev-004"], "answer", 0.87),
    };

    let (cycle_label, claims, evidence, action, confidence) = trace_data;

    rsx! {
        section { class: "card",
            h3 { "Long-Horizon Trace Viewer" }
            p { class: "muted small", "Detailed breakdown of selected cycle" }
            
            div { class: "trace-controls",
                label { "Select Cycle: " }
                input {
                    r#type: "range",
                    min: "1",
                    max: "15",
                    value: "{selected_cycle.read()}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<usize>() {
                            selected_cycle.set(val);
                        }
                    }
                }
                span { class: "cycle-display", "{cycle_label}" }
            }
            
            div { class: "trace-content",
                div { class: "trace-section",
                    h4 { "Action Taken" }
                    div { class: "trace-action",
                        span { class: "action-badge", "{action}" }
                        span { class: "action-confidence",
                            "Confidence: {(confidence * 100.0) as u8}%"
                        }
                    }
                }
                
                div { class: "trace-section",
                    h4 { "Claims Active" }
                    div { class: "trace-claims",
                        if claims.is_empty() {
                            p { class: "muted", "(no claims active)" }
                        } else {
                            for claim_id in &claims {
                                span { class: "claim-badge", "{claim_id}" }
                            }
                        }
                    }
                }
                
                div { class: "trace-section",
                    h4 { "Evidence Used" }
                    div { class: "trace-evidence",
                        if evidence.is_empty() {
                            p { class: "muted", "(no evidence linked)" }
                        } else {
                            for ev_id in &evidence {
                                span { class: "evidence-badge", "{ev_id}" }
                            }
                        }
                    }
                }
                
                div { class: "trace-section",
                    h4 { "Reasoning Audit" }
                    div { class: "trace-audit",
                        p { class: "audit-text",
                            "Cycle {*selected_cycle.read()}: Evaluated {claims.len()} claim(s) with {evidence.len()} evidence link(s). \
                             Selected action: {action} with {(confidence * 100.0) as u8}% confidence."
                        }
                    }
                }
            }
            
            div { class: "trace-navigation",
                button {
                    onclick: move |_| {
                        let current = *selected_cycle.read();
                        if current > 1 {
                            selected_cycle.set(current - 1);
                        }
                    },
                    "← Previous"
                }
                button {
                    onclick: move |_| {
                        let current = *selected_cycle.read();
                        if current < 15 {
                            selected_cycle.set(current + 1);
                        }
                    },
                    "Next →"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_viewer_renders() {
        let mut vdom = VirtualDom::new_with_props(
            LongHorizonTraceViewer,
            LongHorizonTraceViewerProps { current_cycle: 1 },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("trace-content"));
        assert!(html.contains("trace-navigation"));
    }
}
