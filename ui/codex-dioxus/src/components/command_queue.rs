use crate::bridge::types::{CommandApprovalRecord, CommandApprovalState};
use dioxus::prelude::*;

#[component]
pub fn CommandQueue(
    records: Vec<CommandApprovalRecord>,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "command-queue",
            h3 { "Pending Command Queue" }
            if records.is_empty() {
                p { class: "muted", "No pending commands for review." }
            } else {
                div { class: "list",
                    {
                        records.into_iter().map(|record| {
                            let approve_id = record.id.clone();
                            let reject_id = record.id.clone();
                            rsx! {
                                div { class: "card command-item",
                                    key: "{record.id}",
                                    div { class: "command-header",
                                        span { class: "badge", "{record.state:?}" }
                                        span { class: "timestamp", "{record.timestamp}" }
                                    }
                                    code { class: "command-body", "{record.command}" }
                                    if record.state == CommandApprovalState::AwaitingApproval {
                                        div { class: "button-row",
                                            button {
                                                class: "btn primary",
                                                onclick: move |_| on_approve.call(approve_id.clone()),
                                                "Approve"
                                            }
                                            button {
                                                class: "btn",
                                                onclick: move |_| on_reject.call(reject_id.clone()),
                                                "Reject"
                                            }
                                        }
                                    }
                                    if let Some(res) = &record.result {
                                        p { class: "command-result", "Result: {res}" }
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}
