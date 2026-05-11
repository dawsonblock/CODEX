use dioxus::prelude::*;

use crate::bridge::proof_reader::load_dashboard_state;
use crate::bridge::runtime_client::{next_message_id, now_timestamp_string, RuntimeClient};
use crate::bridge::types::{
    ChatMessage, ChatRole, CodexProofState, HistoricalSummary, ProofManifest, RuntimeBridgeMode,
    TimeRange, UiSettings,
};
use crate::components::action_schema_panel::ActionSchemaPanel;
use crate::components::action_trace_panel::ActionTracePanel;
use crate::components::audit_panel::AuditPanel;
use crate::components::chat_input::ChatInput;
use crate::components::chat_view::ChatView;
use crate::components::console_panel::ConsolePanel;
use crate::components::evidence_panel::EvidencePanel;
use crate::components::pressure_panel::PressurePanel;
use crate::components::proof_dashboard::ProofDashboard;
use crate::components::runtime_status::RuntimeStatusPanel;
use crate::components::settings_panel::SettingsPanel;
use crate::{LOGO_SVG, MAIN_CSS};

pub const UI_BOUNDARY_LINES: [&str; 5] = [
    "Codex-main 32 is not sentient.",
    "Codex-main 32 is not conscious.",
    "Codex-main 32 is not AGI.",
    "Codex-main 32 is not production-ready.",
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
    let mut settings = use_signal(UiSettings::default);
    let mut selected_message_id = use_signal(|| None::<String>);
    let mut messages = use_signal(|| {
        vec![ChatMessage {
            id: next_message_id(ChatRole::System),
            role: ChatRole::System,
            content: "Codex chat shell ready. Runtime authority remains in Rust; provider/tool execution is disabled in this pass.".to_string(),
            timestamp: now_timestamp_string(),
            runtime: None,
        }]
    });

    let current: Option<CodexProofState> = dashboard_state().proof.clone();
    let history: HistoricalSummary = dashboard_state().history.clone();
    let errors = dashboard_state().errors.clone();
    let manifest: Option<ProofManifest> = current.clone().map(|s| s.manifest);
    let selected_trace = {
        let selected = selected_message_id();
        let msgs = messages();
        selected.and_then(|id| {
            msgs.iter()
                .find(|m| m.id == id)
                .and_then(|m| m.runtime.clone())
        })
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div { class: "app-shell {theme().class()}",
            aside { class: "sidebar",
                div { class: "brand",
                    img { src: LOGO_SVG }
                    h1 { "Codex" }
                }
                button {
                    class: "nav-item active",
                    onclick: move |_| {
                        messages.set(vec![ChatMessage {
                            id: next_message_id(ChatRole::System),
                            role: ChatRole::System,
                            content: "New chat started. UI session history only; not runtime claim memory.".to_string(),
                            timestamp: now_timestamp_string(),
                            runtime: None,
                        }]);
                        selected_message_id.set(None);
                    },
                    "New Chat"
                }
                button { class: "nav-item", "Proof" }
                button { class: "nav-item", "Evidence" }
                button { class: "nav-item", "Claims" }
                button { class: "nav-item", "Pressure" }
                button { class: "nav-item", "Audit" }
                button { class: "nav-item", "Settings" }
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
                        h2 { "Codex Chat" }
                        p { class: "subtitle", "Chat shell backed by a bounded CODEX runtime bridge." }
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
                            "Reload Proof/Status"
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

                div { class: "chat-shell",
                    div { class: "chat-center",
                        ChatView {
                            messages: messages(),
                            selected_id: selected_message_id(),
                            on_select: move |id| selected_message_id.set(Some(id)),
                        }
                        ChatInput {
                            on_send: move |text: String| {
                                let user_message = ChatMessage {
                                    id: next_message_id(ChatRole::User),
                                    role: ChatRole::User,
                                    content: text.clone(),
                                    timestamp: now_timestamp_string(),
                                    runtime: None,
                                };

                                messages.with_mut(|m| m.push(user_message));

                                let mode = settings().runtime_bridge_mode;
                                let mut messages = messages.clone();
                                let mut selected_message_id = selected_message_id.clone();

                                spawn(async move {
                                    let client = RuntimeClient::new(mode);
                                    let response = client.send_user_message(&text).await;
                                    let assistant_message = ChatMessage {
                                        id: next_message_id(ChatRole::Codex),
                                        role: ChatRole::Codex,
                                        content: response.message,
                                        timestamp: now_timestamp_string(),
                                        runtime: Some(response.trace),
                                    };

                                    let selected_id = assistant_message.id.clone();
                                    messages.with_mut(|m| m.push(assistant_message));
                                    selected_message_id.set(Some(selected_id));
                                });
                            }
                        }
                    }

                    if settings().show_metadata_panel {
                        div { class: "chat-inspector",
                            ActionTracePanel { trace: selected_trace }
                            RuntimeStatusPanel { manifest }
                            ProofDashboard {
                                state: current.clone(),
                                history,
                                range: time_range(),
                            }
                            if let Some(state) = current.clone() {
                                EvidencePanel { proof: state.clone() }
                                if settings().show_pressure_panel {
                                    PressurePanel { proof: state.clone() }
                                }
                                AuditPanel { proof: state }
                            }
                            ActionSchemaPanel {}
                            SettingsPanel {
                                settings: settings(),
                                on_toggle_metadata: move |_| {
                                    settings.with_mut(|s| s.show_metadata_panel = !s.show_metadata_panel);
                                },
                                on_toggle_pressure: move |_| {
                                    settings.with_mut(|s| s.show_pressure_panel = !s.show_pressure_panel);
                                },
                                on_cycle_bridge_mode: move |_| {
                                    settings.with_mut(|s| {
                                        s.runtime_bridge_mode = match s.runtime_bridge_mode {
                                            RuntimeBridgeMode::MockUiMode => RuntimeBridgeMode::LocalCodexRuntimeReadOnly,
                                            RuntimeBridgeMode::LocalCodexRuntimeReadOnly => RuntimeBridgeMode::LocalOllamaProvider,
                                            RuntimeBridgeMode::LocalOllamaProvider => RuntimeBridgeMode::ExternalProviderDisabled,
                                            RuntimeBridgeMode::ExternalProviderDisabled => RuntimeBridgeMode::MockUiMode,
                                        };
                                    });
                                }
                            }
                            ConsolePanel {}
                        }
                    }
                }
            }
        }
    }
}
