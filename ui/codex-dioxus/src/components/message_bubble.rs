use crate::bridge::types::{ChatMessage, ChatRole};
use crate::components::basis_items_table::BasisItemsTable;
use dioxus::prelude::*;

#[component]
pub fn MessageBubble(
    message: ChatMessage,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let role_class = match message.role {
        ChatRole::User => "bubble-user",
        ChatRole::Codex => "bubble-codex",
        ChatRole::System => "bubble-system",
    };
    let selected_class = if selected { "bubble-selected" } else { "" };
    let id = message.id.clone();

    rsx! {
        div {
            class: "message-wrap {role_class}",
            onclick: move |_| on_select.call(id.clone()),
            div { class: "message-bubble {selected_class}",
                p { class: "message-content",
                    "{message.content}"
                    if message.role == ChatRole::Codex && message.runtime.is_none() && !message.content.is_empty() {
                        span { class: "streaming-caret" }
                    }
                }
                div { class: "message-meta",
                    span { "{message.timestamp}" }
                    if let Some(trace) = &message.runtime {
                        span { "action: {trace.selected_action}" }
                        span { "metadata: {trace.metadata_quality.label()}" }
                        if trace.provider_executions_local > 0 {
                            span { class: "badge warning", "Non-Authoritative Provider ({trace.provider_executions_local})" }
                        }
                    }
                }
                // Phase 9: Display answer basis items for grounded answers
                if let Some(trace) = &message.runtime {
                    if !trace.answer_basis_items.is_empty() {
                        div { class: "message-basis-section",
                            BasisItemsTable {
                                basis_items: trace.answer_basis_items.clone(),
                                warnings: trace.answer_warnings.clone(),
                                show_warnings: true,
                            }
                        }
                    }
                }
            }
        }
    }
}
