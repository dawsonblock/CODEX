use crate::bridge::types::UiSettings;
use dioxus::prelude::*;

#[component]
pub fn SettingsPanel(
    settings: UiSettings,
    on_toggle_metadata: EventHandler<()>,
    on_toggle_pressure: EventHandler<()>,
    on_toggle_provider_gate: EventHandler<()>,
    on_cycle_bridge_mode: EventHandler<()>,
) -> Element {
    rsx! {
        section { class: "card",
            h3 { "Settings" }
            ul { class: "list",
                li { "Theme: dark/light/system (UI shell controlled)" }
                li { "Accent: {settings.accent_color}" }
                li { "Proof artifact path: {settings.proof_artifact_path}" }
                li { "Runtime bridge mode: {settings.runtime_bridge_mode.label()}" }
                li {
                    span { "Provider Security Gate: " }
                    span {
                        class: if settings.provider_gate_enabled { "badge success" } else { "badge danger" },
                        if settings.provider_gate_enabled { "ENABLED" } else { "LOCKED" }
                    }
                }
            }
            div { class: "button-row",
                button {
                    class: "btn",
                    onclick: move |_| on_cycle_bridge_mode.call(()),
                    "Cycle Bridge Mode"
                }
                button {
                    class: if settings.provider_gate_enabled { "btn primary" } else { "btn" },
                    onclick: move |_| on_toggle_provider_gate.call(()),
                    if settings.provider_gate_enabled { "Lock Providers" } else { "Unlock Providers" }
                }
                button {
                    class: "btn",
                    onclick: move |_| on_toggle_metadata.call(()),
                    if settings.show_metadata_panel { "Hide Metadata" } else { "Show Metadata" }
                }
                button {
                    class: "btn",
                    onclick: move |_| on_toggle_pressure.call(()),
                    if settings.show_pressure_panel { "Hide Pressure" } else { "Show Pressure" }
                }
            }
            
            div { class: "warning-banner",
                h4 { "Provider Boundary Notice" }
                p { class: "muted", "Experimental local provider modes (Ollama/Turboquant) use localhost APIs." }
                p { class: "muted", "These are NOT authoritative CODEX runtime outputs and should be used for testing only." }
                p { class: "muted", "Real external cloud tool execution remains strictly DISABLED." }
            }
        }
    }
}
