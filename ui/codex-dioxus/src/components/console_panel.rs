use crate::bridge::runtime_client::{send_runtime_command, RuntimeTransport};
use crate::bridge::types::RuntimeCommand;
use dioxus::prelude::*;

#[component]
pub fn ConsolePanel() -> Element {
    let mut command = use_signal(|| RuntimeCommand::ReplayLast);
    let mut status = use_signal(|| String::from("Commands are disabled in UI v1."));

    rsx! {
        section { class: "card",
            h3 { "Runtime Console (Disabled)" }
            p { class: "muted", "Bridge transport exists for future integration, but execution is intentionally disabled." }
            div { class: "button-row",
                button {
                    class: "btn",
                    onclick: move |_| {
                        command.set(RuntimeCommand::RefreshProofState);
                    },
                    "Select: RefreshProofState"
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        command.set(RuntimeCommand::ReplayLast);
                    },
                    "Select: ReplayLast"
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        command.set(RuntimeCommand::RequestAuditSnapshot);
                    },
                    "Select: RequestAuditSnapshot"
                }
            }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let transport = RuntimeTransport::new_disabled();
                    let result = send_runtime_command(&transport, &command());
                    status.set(format!(
                        "{:?} {:?}: {}",
                        result.command, result.status, result.message
                    ));
                },
                "Send (Disabled)"
            }
            p { class: "muted", "Status: {status()}" }
        }
    }
}
