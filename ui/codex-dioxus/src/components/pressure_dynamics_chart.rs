use dioxus::prelude::*;

/// Pressure dynamics display showing action score trends across cycles
#[component]
pub fn PressureDynamicsChart(cycle_count: u8) -> Element {
    // Mock pressure and regulation data across 15 cycles
    let pressure_data = vec![
        0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.85, 0.8, 0.75, 0.7, 0.65, 0.6, 0.5,
    ];

    let regulation_data = vec![
        0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.82, 0.85, 0.88, 0.87, 0.85, 0.82, 0.8, 0.8, 0.8,
    ];

    rsx! {
        section { class: "card",
            h3 { "Pressure Dynamics" }
            p { class: "muted small",
                "Action score trends and regulation effects across {cycle_count} cycles"
            }

            div { class: "pressure-chart-container",
                // ASCII-style chart representation
                div { class: "pressure-chart",
                    div { class: "chart-title", "Pressure Trend" }
                    div { class: "chart-visualization",
                        div { class: "chart-bars",
                            for (cycle, pressure) in pressure_data.iter().enumerate() {
                                div { class: "bar-group",
                                    div { class: "bar-container",
                                        div {
                                            class: "bar pressure-bar",
                                            style: format!("--bar-height: {}%", (pressure * 100.0) as i32),
                                            title: format!("Cycle {}: {:.3}", cycle + 1, pressure)
                                        }
                                    }
                                    span { class: "bar-label", "{cycle + 1}" }
                                }
                            }
                        }
                        div { class: "chart-axis-y",
                            span { "1.0" }
                            span { "0.5" }
                            span { "0.0" }
                        }
                    }
                }

                div { class: "regulation-chart",
                    div { class: "chart-title", "Regulation Effect" }
                    div { class: "chart-visualization",
                        div { class: "chart-bars",
                            for (cycle, regulation) in regulation_data.iter().enumerate() {
                                div { class: "bar-group",
                                    div { class: "bar-container",
                                        div {
                                            class: "bar regulation-bar",
                                            style: format!("--bar-height: {}%", (regulation * 100.0) as i32),
                                            title: format!("Cycle {}: {:.3}", cycle + 1, regulation)
                                        }
                                    }
                                    span { class: "bar-label", "{cycle + 1}" }
                                }
                            }
                        }
                        div { class: "chart-axis-y",
                            span { "1.0" }
                            span { "0.5" }
                            span { "0.0" }
                        }
                    }
                }
            }

            div { class: "pressure-metrics",
                div { class: "metric-box",
                    span { class: "metric-label", "Average Pressure" }
                    span { class: "metric-value", "0.615" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Average Regulation" }
                    span { class: "metric-value", "0.823" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Peak Pressure" }
                    span { class: "metric-value bad", "0.850" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Pressure Range" }
                    span { class: "metric-value", "0.200 - 0.850" }
                }
            }

            p { class: "muted small chart-note",
                "Green bars show confident behavior (high regulation), red shows deferring behavior (high pressure)"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_dynamics_renders() {
        let mut vdom = VirtualDom::new_with_props(
            PressureDynamicsChart,
            PressureDynamicsChartProps { cycle_count: 15 },
        );
        let html = format!("{:?}", vdom.render_immediate());
        assert!(html.contains("pressure-chart"));
        assert!(html.contains("regulation"));
    }
}
