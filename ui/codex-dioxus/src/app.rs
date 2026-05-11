use dioxus::prelude::*;

use crate::bridge::proof_reader::load_dashboard_state;
use crate::bridge::types::{CodexProofState, HistoricalSummary, ProofManifest, TimeRange};
use crate::components::action_schema_panel::ActionSchemaPanel;
use crate::components::audit_panel::AuditPanel;
use crate::components::console_panel::ConsolePanel;
use crate::components::evidence_panel::EvidencePanel;
use crate::components::pressure_panel::PressurePanel;
use crate::components::proof_dashboard::ProofDashboard;
use crate::components::runtime_status::RuntimeStatusPanel;
use crate::{LOGO_SVG, MAIN_CSS};

pub const UI_BOUNDARY_LINES: [&str; 5] = [
    "CODEX-main 32 is not sentient.",
    "CODEX-main 32 is not conscious.",
    "CODEX-main 32 is not AGI.",
    "CODEX-main 32 is not production-ready.",
    "UI shell is bounded to visualization and disabled command intents.",
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn warning_snapshot(errors: &[String]) -> String {
    if errors.is_empty() {
        "none".to_string()
    } else {
        errors.join(" | ")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    Dark,
    Light,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_snapshot_none() {
        let errors: Vec<String> = Vec::new();
        assert_eq!(warning_snapshot(&errors), "none");
    }

    #[test]
    fn warning_snapshot_joins_messages() {
        let errors = vec!["missing report".to_string(), "bad json".to_string()];
        assert_eq!(warning_snapshot(&errors), "missing report | bad json");
    }
}

impl Theme {
    fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Dark => "theme-dark",
            Self::Light => "theme-light",
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut theme = use_signal(|| Theme::Dark);
    let mut time_range = use_signal(|| TimeRange::Current);
    let mut dashboard_state = use_signal(|| load_dashboard_state(TimeRange::Current));

    let current: Option<CodexProofState> = dashboard_state().proof.clone();
    let history: HistoricalSummary = dashboard_state().history.clone();
    let errors = dashboard_state().errors.clone();
    let manifest: Option<ProofManifest> = current.clone().map(|s| s.manifest);

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div { class: "app-shell {theme().class()}",
            aside { class: "sidebar",
                div { class: "brand",
                    img { src: LOGO_SVG }
                    h1 { "Codex" }
                }
                button { class: "nav-item active", "Dashboard" }
                button { class: "nav-item", disabled: true, "Logs (planned)" }
                button { class: "nav-item", disabled: true, "Replay (planned)" }
                div { class: "sidebar-footer",
                    "UI shell only. Runtime authority remains in Rust workspace."
                    ul { class: "list",
                        for line in UI_BOUNDARY_LINES {
                            li { "{line}" }
                        }
                    }
                }
            }

            main { class: "main",
                div { class: "banner",
                    "Boundary: UI displays proof/runtime artifacts and does not execute external tools."
                }
                div { class: "header-row",
                    div {
                        h2 { "Codex Dashboard" }
                        p { class: "subtitle", "Evidence-grounded, contradiction-aware, replay-auditable." }
                    }
                    div { class: "toolbar",
                        button {
                            class: "btn",
                            onclick: move |_| {
                                time_range.set(TimeRange::Current);
                                dashboard_state.set(load_dashboard_state(TimeRange::Current));
                            },
                            "Current"
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                time_range.set(TimeRange::Last24Hours);
                                dashboard_state.set(load_dashboard_state(TimeRange::Last24Hours));
                            },
                            "Last 24h"
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                time_range.set(TimeRange::Last7Days);
                                dashboard_state.set(load_dashboard_state(TimeRange::Last7Days));
                            },
                            "Last 7d"
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                time_range.set(TimeRange::AllHistory);
                                dashboard_state.set(load_dashboard_state(TimeRange::AllHistory));
                            },
                            "All"
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                theme.set(theme().toggle());
                            },
                            "Toggle Theme"
                        }
                        button {
                            class: "btn primary",
                            onclick: move |_| {
                                dashboard_state.set(load_dashboard_state(time_range()));
                            },
                            "Reload Proof"
                        }
                    }
                }

                if !errors.is_empty() {
                    div { class: "error-box",
                        strong { "Proof load warnings" }
                        ul { class: "list",
                            for err in errors {
                                li { "{err}" }
                            }
                        }
                    }
                }

                div { class: "grid",
                    RuntimeStatusPanel { manifest }
                    ProofDashboard {
                        state: current.clone(),
                        history,
                        range: time_range(),
                    }

                    if let Some(state) = current.clone() {
                        EvidencePanel { proof: state.clone() }
                        PressurePanel { proof: state.clone() }
                        AuditPanel { proof: state }
                    }

                    ActionSchemaPanel {}
                    ConsolePanel {}
                }
            }
        }
    }
}
