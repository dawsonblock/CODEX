use crate::bridge::runtime_client::{
    grant_command_approval, request_command_approval, reset_command_approval, send_runtime_command,
    RuntimeTransport,
};
use crate::bridge::types::{CommandApprovalState, RuntimeCommand};
use dioxus::prelude::*;

#[component]
pub fn ConsolePanel() -> Element {
    let mut command = use_signal(|| RuntimeCommand::ReplayLast);
    let mut approval = use_signal(CommandApprovalState::default);
    let mut status = use_signal(|| String::from("Commands are disabled in UI v1."));
    let approval_label = format!("{:?}", approval());

    rsx! {
        section { class: "card",
            h3 { "Runtime Console (Gated Dry-Run)" }
            p { class: "muted", "Bridge transport exists for future integration, but execution is intentionally disabled." }
            p { class: "muted", "Approval state: {approval_label}" }
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
            div { class: "button-row",
                button {
                    class: "btn",
                    onclick: move |_| {
                        approval.set(request_command_approval(approval()));
                        status.set("Approval requested.".to_string());
                    },
                    "Request Approval"
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        approval.set(grant_command_approval(approval()));
                        status.set("Approval granted for dry-run intent.".to_string());
                    },
                    "Grant Approval"
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        approval.set(reset_command_approval());
                        status.set("Approval reset to draft.".to_string());
                    },
                    "Reset"
                }
            }
            button {
                class: "btn primary",
                onclick: move |_| {
                    let transport = RuntimeTransport::new_disabled();
                    let result = send_runtime_command(&transport, &command(), approval());
                    status.set(format!(
                        "{:?} {:?}: {}",
                        result.command, result.status, result.message
                    ));
                },
                "Send (Dry-Run Intent)"
            }
            p { class: "muted", "Status: {status()}" }
        }
    }
}
