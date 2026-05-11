use dioxus::prelude::*;

use crate::bridge::proof_reader::load_proof_state;
use crate::bridge::types::{CodexProofState, ProofManifest};
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    Dark,
    Light,
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
    let mut proof_state = use_signal(load_proof_state);

    let current: Option<CodexProofState> = proof_state().state.clone();
    let errors = proof_state().errors.clone();
    let manifest: Option<ProofManifest> = current.clone().map(|s| s.manifest);

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div { class: "app-shell {theme().class()}",
            aside { class: "sidebar",
                div { class: "brand",
                    img { src: LOGO_SVG }
                    h1 { "CODEX" }
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
                        h2 { "Runtime Dashboard" }
                        p { class: "subtitle", "Evidence-grounded, contradiction-aware, replay-auditable." }
                    }
                    div { class: "toolbar",
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
                                proof_state.set(load_proof_state());
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
                    ProofDashboard { state: current.clone() }

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
