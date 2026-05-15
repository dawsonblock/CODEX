use crate::bridge::state_provider::use_ui_runtime_state;
use dioxus::prelude::*;

/// Pressure dynamics display showing action score trends across cycles
/// Now integrated with UIRuntimeState for live pressure metrics
#[component]
pub fn PressureDynamicsChart(#[props(default)] cycle_count: Option<u8>) -> Element {
    // Get live state
    let state = use_ui_runtime_state();
    let pressure_metrics = state.read().pressure_metrics.read().clone();
    let current_cycle = *state.read().current_cycle.read() as u8;

    // Use passed cycle_count or derive from state
    let display_cycle_count = cycle_count.unwrap_or(current_cycle);

    // Extract pressure values from live metrics
    let pressure_data: Vec<f64> = pressure_metrics.iter().map(|m| m.pressure).collect();

    let regulation_data: Vec<f64> = pressure_metrics.iter().map(|m| m.regulation).collect();

    // Calculate statistics from live data
    let avg_pressure = if !pressure_data.is_empty() {
        pressure_data.iter().sum::<f64>() / pressure_data.len() as f64
    } else {
        0.0
    };

    let avg_regulation = if !regulation_data.is_empty() {
        regulation_data.iter().sum::<f64>() / regulation_data.len() as f64
    } else {
        0.0
    };

    let peak_pressure = pressure_data.iter().copied().fold(0.0, f64::max);

    let min_pressure = pressure_data.iter().copied().fold(f64::INFINITY, f64::min);

    rsx! {
        section { class: "card",
            h3 { "Pressure Dynamics (Live)" }
            p { class: "muted small",
                "Action score trends and regulation effects across {display_cycle_count} cycles (Current: {current_cycle})"
            }

            div { class: "pressure-chart-container",
                // ASCII-style chart representation
                div { class: "pressure-chart",
                    div { class: "chart-title", "Pressure Trend" }
                    div { class: "chart-visualization",
                        div { class: "chart-bars",
                            if pressure_data.is_empty() {
                                div { class: "chart-placeholder", "Awaiting pressure metrics..." }
                            } else {
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
                            if regulation_data.is_empty() {
                                div { class: "chart-placeholder", "Awaiting regulation data..." }
                            } else {
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
                    span { class: "metric-value", "{avg_pressure:.3}" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Average Regulation" }
                    span { class: "metric-value", "{avg_regulation:.3}" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Peak Pressure" }
                    span { class: "metric-value bad", "{peak_pressure:.3}" }
                }
                div { class: "metric-box",
                    span { class: "metric-label", "Pressure Range" }
                    span { class: "metric-value", "{min_pressure:.3} - {peak_pressure:.3}" }
                }
            }

            p { class: "muted small chart-note",
                "Green bars show confident behavior (high regulation), red shows deferring behavior (high pressure)"
            }
        }
    }
}

// Test module disabled - Dioxus 0.7 rendering API has changed
// To be re-enabled once tests are migrated to new API patterns
