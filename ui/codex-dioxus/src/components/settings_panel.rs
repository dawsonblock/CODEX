use crate::bridge::types::UiSettings;
use dioxus::prelude::*;

#[component]
pub fn SettingsPanel(
    settings: UiSettings,
    on_toggle_metadata: EventHandler<()>,
    on_toggle_pressure: EventHandler<()>,
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
            }
            div { class: "button-row",
                button {
                    class: "btn",
                    onclick: move |_| on_cycle_bridge_mode.call(()),
                    "Cycle Bridge Mode"
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
            p { class: "muted", "Local CODEX runtime mode is read-only. Provider execution remains disabled in this version." }
            p { class: "muted", "No real external tool execution is enabled in this pass." }
        }
    }
}
