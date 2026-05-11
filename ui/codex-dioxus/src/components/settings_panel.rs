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
                    if cfg!(feature = "ui-local-providers") {
                        span {
                            class: if settings.provider_gate_enabled { "badge success" } else { "badge danger" },
                            if settings.provider_gate_enabled { "ENABLED" } else { "LOCKED" }
                        }
                    } else {
                        span { class: "badge warning", "NOT COMPILED IN (default build)" }
                    }
                }
            }
            div { class: "button-row",
                button {
                    class: "btn",
                    onclick: move |_| on_cycle_bridge_mode.call(()),
                    "Cycle Bridge Mode"
                }
                if cfg!(feature = "ui-local-providers") {
                    button {
                        class: if settings.provider_gate_enabled { "btn primary" } else { "btn" },
                        onclick: move |_| on_toggle_provider_gate.call(()),
                        if settings.provider_gate_enabled { "Lock Providers" } else { "Unlock Providers" }
                    }
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

            if cfg!(feature = "ui-local-providers") {
                div { class: "warning-banner",
                    h4 { "⚠ Experimental Local Provider Mode Active" }
                    p { class: "muted", "Local provider modes (Ollama/Turboquant) use localhost APIs only." }
                    p { class: "muted", "These are NOT authoritative CODEX runtime outputs. Use for testing only." }
                    p { class: "muted", "Real external cloud tool execution remains strictly DISABLED." }
                    p { class: "muted", "Provider output is labeled \"Local provider draft — not CODEX runtime authority\"." }
                }
            } else {
                div { class: "card",
                    h4 { "Provider Execution" }
                    p { class: "muted", "Provider execution is disabled in this build (default CODEX build)." }
                    p { class: "muted", "External and local cloud providers are not compiled in." }
                    p { class: "muted", "To enable experimental localhost providers, build with --features ui-local-providers." }
                    p { class: "muted", "CODEX runtime remains authoritative." }
                }
            }
        }
    }
}
